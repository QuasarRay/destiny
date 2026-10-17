#!/usr/bin/env python3
"""Refresh the canonical queue, generate architecture views, and fail closed.

The existing Aegis case inventory and verifier validators remain authoritative
for their assurance lanes. This entry point adds latest-source scope and keeps
the incomplete product/configuration inventory visible.
"""
import argparse
import ast
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path
import platform
import re
import sys
import tempfile
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]
STATE = ROOT / ".aegis/destiny/migration"
RECORDS = ROOT / "verification/migration"
QUEUE = ROOT / ".aegis/destiny/work-queue.json"
sys.path.insert(0, str(ROOT))
sys.path.insert(0, str(ROOT / ".agents/infra"))
from agentinfra.atomic import atomic_write_json
from agentinfra.locks import FileLock


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text())


def original_region(revision, source, span):
    from tools.migrate_destiny import blob
    content = blob(revision, source)
    start, end = span
    lines = content.splitlines(keepends=True)
    if not 0 < start <= end <= len(lines):
        raise ValueError("original source span is invalid: " + source)
    return content, b"".join(lines[start - 1:end])


def validate_world_scope(scope, inventory):
    """Check reviewed source packets, not imports or old pass/fail categories.

    This is a partial semantic review input. Unreviewed cases stay UNKNOWN;
    this validator does not pretend a fixture inventory is a feature inventory.
    """
    revision = inventory["authoritative_commit"]
    if scope["original_commit"] != revision:
        raise ValueError("world scope source authority is stale")
    cases = {row["id"]: row for row in inventory["revisions"][-1]["cases"]}
    features = {row["id"]: row for row in scope["features"]}
    reviewed = {row["id"]: row for row in scope["case_classifications"]}
    if len(features) != len(scope["features"]) or len(reviewed) != len(scope["case_classifications"]):
        raise ValueError("repeated world feature or classification identity")
    for feature in features.values():
        if feature["family"] not in scope["minimum_family_checklist"]:
            raise ValueError("unknown world feature family")
        if not feature.get("source_regions"):
            raise ValueError("world feature lacks source-bound regions: " + feature["id"])
        for region in feature["source_regions"]:
            content, selected = original_region(revision, region["source"], region["span"])
            if (hashlib.sha256(content).hexdigest() != region["source_sha256"] or
                    hashlib.sha256(selected).hexdigest() != region["region_sha256"]):
                raise ValueError("world feature source digest is stale: " + feature["id"])
    for identity, row in reviewed.items():
        if identity not in cases or row["source"] != cases[identity]["source"]:
            raise ValueError("classification does not identify an original fixture")
        if row["category"] not in {"WORLD_CORE", "HOST_ADAPTER", "MIXED", "UNKNOWN"}:
            raise ValueError("unknown semantic category")
        content, selected = original_region(revision, row["source"], row["source_span"])
        if (row["source_sha256"] != cases[identity]["source_sha256"] or
                hashlib.sha256(content).hexdigest() != row["source_sha256"] or
                hashlib.sha256(selected).hexdigest() != row["region_sha256"]):
            raise ValueError("classification source digest is stale: " + identity)
        if not row["rationale"] or not set(row["feature_ids"]) <= features.keys():
            raise ValueError("classification has missing rationale or omitted core feature: " + identity)
        audits = [audit for audit in scope["classification_audit"] if audit["id"] == identity]
        if not audits or audits[-1]["to"] != row["category"] or audits[-1]["source_sha256"] != row["source_sha256"] or not audits[-1]["reason"]:
            raise ValueError("classification change lacks a current source-bound audit")
        assertions = {item["id"]: item for item in row["assertions"]}
        if len(assertions) != len(row["assertions"]) or not assertions:
            raise ValueError("reviewed fixture has ambiguous or missing assertions")
        if row["source"].endswith(".py"):
            tree = ast.parse(content)
            start, end = row["source_span"]
            actual = {(node.lineno, node.func.attr) for node in ast.walk(tree)
                      if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
                      and node.func.attr.startswith("assert") and start <= node.lineno <= end}
            if actual != {(item["line"], item["call"]) for item in assertions.values()}:
                raise ValueError("review omitted an original assertion: " + identity)
        for item in assertions.values():
            if item["id"] != f"{identity}::assert:{item['line']}":
                raise ValueError("assertion identity is not source-qualified")
        core = set(row.get("core_assertion_ids", []))
        host = set(row.get("adapter_assertion_ids", []))
        if core & host or core | host != assertions.keys():
            raise ValueError("missing or multiply classified original assertions")
        category = row["category"]
        if category in {"WORLD_CORE", "MIXED"} and (not core or not row["feature_ids"]):
            raise ValueError("core semantics have no retained core assertion/feature")
        if category == "HOST_ADAPTER" and (core or row["feature_ids"]):
            raise ValueError("core assertions incorrectly deferred as host-only")
        if category == "WORLD_CORE" and host:
            raise ValueError("core-only classification includes host assertions")
        # assertBallEqual observes world state regardless of the Blue stream
        # used to carry it. A stream import never permits wholesale deferral.
        if any(item["call"] == "assertBallEqual" and item["id"] not in core for item in assertions.values()):
            raise ValueError("mixed state-transfer fixture incorrectly deferred wholesale")
        if category in {"HOST_ADAPTER", "MIXED"}:
            contract = row.get("adapter_contract", {})
            if not contract.get("deferred_reason") or not contract.get("original_boundaries"):
                raise ValueError("deferred adapter has no recorded boundary/reason")
            if any(boundary.encode() not in selected for boundary in contract["original_boundaries"]):
                raise ValueError("adapter boundary is absent from reviewed original region")
    return features, reviewed


def core_inputs():
    from tools.migrate_destiny import generator_identity, blob, LATEST
    # Python bytecode is an observation artifact, not a Rust/compiler input.
    # Keep all source/fixture files and their names, excluding only ephemeral
    # interpreter caches that independent source-contract checks may refresh.
    paths = {path for directory in ("src", "verification/spec/src", "tests", "verification/codegen/src")
             for path in (ROOT / directory).rglob("*") if path.is_file()
             and path.suffix not in {".pyc", ".pyo"} and "__pycache__" not in path.parts}
    paths.update(ROOT / path for path in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                 "verification/spec/Cargo.toml", "verification/spec/Cargo.lock",
                 "verification/codegen/Cargo.toml", "verification/codegen/Cargo.lock",
                 "verification/kani/src/iterative_sphere_production_contracts.rs",
                 "verification/kani/src/noniterative_production_contracts.rs",
                 "verification/migration/angular_oracle_samples.json", "verification/migration/angular_rules.json",
                 "verification/migration/angular_source_evidence.json", "verification/migration/speed_source_evidence.json",
                 "verification/migration/stop_oracle_samples.json", "verification/tools/check_stop_migration.py",
                 "verification/migration/warp_alignment_source_evidence.json",
                 "verification/migration/missile_source_evidence.json", "verification/migration/missile_oracle_samples.json",
                 "verification/migration/partition_source_evidence.json", "verification/migration/partition_rules.json",
                 "verification/migration/partition_oracle_samples.json", "verification/migration/partition_dynamic_oracle_samples.json", "verification/tools/check_partition_migration.py",
                 "verification/migration/liveness_source_evidence.json", "verification/migration/liveness_rules.json",
                 "verification/migration/liveness_oracle_samples.json",
                 "verification/migration/force_rules.json", "verification/migration/force_source_evidence.json",
                 "verification/migration/force_oracle_samples.json",
                 "verification/migration/contact_rules.json", "verification/migration/contact_source_evidence.json",
                 "verification/migration/collision_dispatch_oracle_samples.json",
                 "verification/migration/kinematic_rules.json", "verification/migration/kinematic_source_evidence.json",
                 "verification/migration/kinematic_oracle_samples.json",
                 "verification/tools/check_missile_migration.py", "verification/cases/math_dependencies.json"))
    # Expected missile vectors depend on actual pinned source files, including
    # ignored dependency checkouts; retain their identities in observations.
    missile_oracle = read(ROOT / "verification/migration/missile_oracle_samples.json")
    for key in ("math_source_sha256", "directx_source_sha256"):
        for path, expected in missile_oracle["rotation"][key].items():
            source = ROOT / path
            if sha(source) != expected:
                raise ValueError("missile oracle dependency source changed: " + path)
            paths.add(source)
    liveness_oracle = read(ROOT / "verification/migration/liveness_oracle_samples.json")
    force_oracle = read(ROOT / "verification/migration/force_oracle_samples.json")
    dispatch_oracle=read(ROOT/"verification/migration/collision_dispatch_oracle_samples.json")
    kinematic_oracle=read(ROOT/"verification/migration/kinematic_oracle_samples.json")
    for oracle,stem,library,receipt in [
            (force_oracle,"force_oracle","liboriginal_force.so","world-force-oracle-build-execution.json"),
            (dispatch_oracle,"collision_dispatch_oracle","liboriginal_collision_dispatch.so","world-collision-dispatch-oracle-build-execution.json"),
            (kinematic_oracle,"kinematic_oracle","liboriginal_kinematic.so","world-kinematic-oracle-build-execution.json")]:
        if (oracle["original_commit"]!=LATEST or
                oracle["checker_sha256"]!=sha(ROOT/"verification/tools/check_stop_migration.py")):
            raise ValueError("source oracle checker changed: "+stem)
        for source,expected in oracle["source_sha256"].items():
            if hashlib.sha256(blob(LATEST,source)).hexdigest()!=expected:
                raise ValueError("source oracle authority changed: "+source)
        for path,expected in [(STATE/"stop-oracle"/(stem+".cpp"),oracle["scaffold_sha256"]),
                              (STATE/"stop-oracle"/library,oracle["library_sha256"])]:
            if not path.is_file() or sha(path)!=expected:
                raise ValueError("source oracle compiled artifact changed: "+str(path))
        build=read(STATE/receipt)
        if (build["argv"]!=oracle["command"] or build["returncode"] or build["timed_out"] or
                build["stdout_truncated"] or build["stderr_truncated"]):
            raise ValueError("source oracle compile receipt is invalid: "+stem)
    for path,expected in kinematic_oracle["external_source_sha256"].items():
        source=ROOT/path
        if not source.is_file() or sha(source)!=expected:
            raise ValueError("kinematic source constructor changed: "+path)
        paths.add(source)
    if sha(STATE/"stop-oracle/liboriginal_stop.so")!=kinematic_oracle["stop_reference_library_sha256"]:
        raise ValueError("kinematic stop reference changed")
    contact_angles=force_oracle["contact_angles"]
    if (hashlib.sha256(blob(LATEST,"src/Ball.cpp")).hexdigest()!=contact_angles["source_sha256"] or
            sha(STATE/"angular-oracle/liboriginal_angular.so")!=contact_angles["library_sha256"] or
            sha(STATE/"angular-oracle/build.json")!=contact_angles["build_receipt_sha256"]):
        raise ValueError("contact angular oracle source/compiled binding changed")
    environment = liveness_oracle["environment"]["external_sha256"]
    for path, expected in environment.items():
        source = Path(path)
        if not source.is_file() or sha(source) != expected:
            raise ValueError("liveness CPython oracle environment changed: " + path)
    return {"source_sha256": {str(path.relative_to(ROOT)): sha(path) for path in sorted(paths)},
            "oracle_environment_sha256": environment,
            "generator_sha256": generator_identity()}


def core_commands():
    return [
        ("core-debug", ["cargo", "test", "--no-default-features", "--locked", "--offline"]),
        ("angular-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_angular_contract"]),
        ("angular-oracle-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--lib", "generated_angular::tests"]),
        ("motion-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_motion_contract"]),
        ("missile-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_missile_contract"]),
        ("partition-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_partition_contract"]),
        ("liveness-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_liveness_contract"]),
        ("liveness-owner-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--lib", "generated_liveness::tests"]),
        ("force-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_force_contract"]),
        ("contact-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_contact_contract"]),
        ("kinematic-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--test", "original_kinematic_contract"]),
        ("kinematic-phase-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--lib", "source_kinematic_tests"]),
        ("concrete-kernel-release", ["cargo", "test", "--release", "--no-default-features", "--locked", "--offline", "--lib", "concrete_"]),
        ("optional-integration-tests", ["cargo", "test", "--all-features", "--lib", "--locked", "--offline"]),
        ("core-clippy", ["cargo", "clippy", "--no-default-features", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]),
        ("optional-integration-clippy", ["cargo", "clippy", "--all-features", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]),
        ("codegen-clippy", ["cargo", "clippy", "--manifest-path", "verification/codegen/Cargo.toml", "--locked", "--offline", "--", "-D", "warnings"]),
    ]


def observe_core():
    """Retain actual named test executions, with no proof or release credit."""
    from tools.migrate_destiny import checked, LATEST
    before = core_inputs()
    runs = []
    for name, command in core_commands():
        output = checked(command, "world-angular-" + name, 300)
        path = STATE / ("world-angular-" + name + "-execution.json")
        runs.append({"configuration": name, "execution": str(path.relative_to(ROOT)), "sha256": sha(path),
                     "passed_tests": re.findall(r"^test (\S+) \.\.\. ok$", output, re.MULTILINE)})
    if before != core_inputs():
        raise ValueError("core sources changed during the observed executions")
    version = checked(["rustc", "-Vv"], "world-angular-rustc")
    report = {"schema": 1, "original_commit": LATEST, **before, "runs": runs,
              "platform": platform.platform(), "rustc": version, "proof_credit": False,
              "scope": "Named executable observations of the real Rust/Bevy engine on this host. No full feature/domain, proof, release or supported-platform-matrix claim."}
    atomic_write_json(STATE / "world-core-observations.json", report)
    print(json.dumps({"report": str(STATE / "world-core-observations.json"), "passed_test_observations": sum(len(row["passed_tests"]) for row in runs), "proof_credit": False}))


def core_observations(revision):
    path = STATE / "world-core-observations.json"
    if not path.exists():
        return set(), "no current direct core execution receipt"
    document = read(path)
    if document["original_commit"] != revision or any(document.get(key) != value for key, value in core_inputs().items()):
        return set(), "direct core execution receipt is stale"
    witnesses = set()
    expected_commands = dict(core_commands())
    if len(document["runs"]) != len(expected_commands) or {run["configuration"] for run in document["runs"]} != expected_commands.keys():
        return set(), "required observed core command selection is missing or repeated"
    for run in document["runs"]:
        receipt = (ROOT / run["execution"]).resolve()
        if not receipt.is_relative_to(STATE) or not receipt.is_file() or sha(receipt) != run["sha256"]:
            return set(), "direct core process receipt is missing or changed"
        execution = read(receipt)
        if execution["argv"] != expected_commands[run["configuration"]] or execution["cwd"] != str(ROOT):
            return set(), "core receipt did not execute the required production command"
        if execution["returncode"] != 0 or execution["timed_out"] or execution["stdout_truncated"] or execution["stderr_truncated"]:
            return set(), "direct core execution failed or is truncated"
        actual = re.findall(r"^test (\S+) \.\.\. ok$", execution["stdout"], re.MULTILINE)
        if actual != run["passed_tests"]:
            return set(), "named core tests disagree with actual process output"
        witnesses.update(name.rsplit("::", 1)[-1] for name in actual)
    return witnesses, None


def validate_identities(queue, inventory):
    cases = inventory["revisions"][-1]["cases"]
    expected = {row["id"] for row in cases}
    actual = [row["id"] for row in queue["obligations"]]
    if not expected or len(actual) != len(set(actual)) or set(actual) != expected:
        raise ValueError("latest source denominator has missing, repeated or extra cases")
    if queue["original_commit"] != inventory["authoritative_commit"]:
        raise ValueError("canonical queue source authority differs")


def validate_fixture_scope(matrix, latest, inventory_digest):
    if matrix["original_commit"] != latest["commit"] or matrix["inventory_sha256"] != inventory_digest:
        raise ValueError("native fixture source authority/inventory is stale")
    expected = {row["id"]: row["source_sha256"] for row in latest["cases"] if row["source"].endswith(".py")}
    observations = matrix["cases"]
    if len(observations) != len(expected) or {row["id"] for row in observations} != set(expected):
        raise ValueError("native fixture selection is missing, repeated or extra")
    if any(row["source_sha256"] != expected[row["id"]] for row in observations):
        raise ValueError("native fixture original source hash differs")
    if not matrix.get("production_rust_sha256") or not matrix.get("build_input_sha256"):
        raise ValueError("native fixture receipt lacks observed Rust/build source identities")
    if matrix.get("matrix_runner_sha256") != sha(ROOT / "verification/tools/run_original_native_matrix.py") or matrix.get("case_runner_sha256") != sha(ROOT / "verification/tools/run_original_native_cases.py"):
        raise ValueError("native fixture runner changed")


def queue():
    from agentinfra.catalog import work_queue
    from agentinfra.destiny import load_profile
    sys.path.insert(0, str(ROOT))
    from tools.migrate_destiny import inventory as refresh_source_inventory
    refresh_source_inventory()
    inventory = read(RECORDS / "source_inventory.json")
    scope = read(RECORDS / "world_scope.json")
    features, reviewed = validate_world_scope(scope, inventory)
    latest = inventory["revisions"][-1]
    legacy_cases = {case["id"] for case in inventory["revisions"][0]["cases"]}
    previous = read(QUEUE) if QUEUE.exists() else {"obligations": []}
    framework_error = None
    try:
        with tempfile.TemporaryDirectory(prefix="framework-input-", dir=STATE) as directory:
            transient = Path(directory) / "case-input.json"
            work_queue(ROOT, load_profile(), SimpleNamespace(original=None, output=str(transient.relative_to(ROOT)), differential=None))
            prior = read(transient)
    except (ValueError, RuntimeError, OSError) as error:
        framework_error = str(error)
        prior = previous
    old = {row["id"]: row for row in prior["obligations"]}
    matrix_path = STATE / "original-native-final/summary.json"
    pointer = STATE / "latest-original-native-matrix.json"
    if pointer.exists():
        reference = read(pointer)
        matrix_path = (ROOT / reference["report"]).resolve()
        if not matrix_path.is_relative_to(ROOT / ".aegis") or sha(matrix_path) != reference["sha256"]:
            raise ValueError("latest native matrix pointer is not a retained in-project receipt")
    matrix = read(matrix_path) if matrix_path.exists() else None
    fixture_error = None
    if matrix:
        try:
            validate_fixture_scope(matrix, latest, sha(RECORDS / "source_inventory.json"))
        except ValueError as error:
            # Preserve stale observations as history; refreshing the canonical
            # inventory must not silently credit them or abort all new work.
            fixture_error = str(error)
    observed = {row["id"]: row for row in matrix["cases"]} if matrix else {}
    library = Path(matrix["library"]) if matrix and matrix.get("library") else ROOT / "target/debug/libdestiny_bevy_compat.so"
    if not library.resolve().is_relative_to(ROOT):
        raise ValueError("current migration matrix library is outside the preserved project")
    runtime_fresh = matrix and not fixture_error and library.exists() and sha(library) == matrix["library_sha256"]
    if runtime_fresh:
        current_files = {
            "production_python_sha256": {str(path.relative_to(ROOT)) for path in (ROOT / "python").rglob("*.py")},
            "production_rust_sha256": {str(path.relative_to(ROOT)) for directory in (ROOT / "src", ROOT / "verification/spec/src") for path in directory.rglob("*.rs")},
            "build_input_sha256": {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "verification/spec/Cargo.toml", "verification/spec/Cargo.lock"},
        }
        runtime_fresh = all(set(matrix[group]) == paths for group, paths in current_files.items())
    if runtime_fresh:
        runtime_fresh = all((ROOT / path).is_file() and sha(ROOT / path) == digest
                            for group in ("production_python_sha256", "production_rust_sha256", "build_input_sha256")
                            for path, digest in matrix[group].items())
    presets = [preset["name"] for preset in latest["configure_presets"] if not preset.get("hidden")]
    witnesses, observation_error = core_observations(latest["commit"])
    rows, world_obligations = [], []
    for case in latest["cases"]:
        row = copy.deepcopy(old.get(case["id"], {}))
        row.update(case)
        row["original_commit"] = latest["commit"]
        row["previous_catalog_commit"] = inventory["legacy_commit"] if case["id"] in legacy_cases else None
        if framework_error or case["id"] not in old:
            historical = row.get("historical_assurance_not_current_credit", row.get("assurance", {}))
            row["historical_assurance_not_current_credit"] = historical
            row["assurance"] = {"mapped_or_marked": historical.get("mapped_or_marked", False),
                                "spec_proved": False, "production_rust_refinement_proved": False,
                                "runtime_differential_evidence": False}
        observation = observed.get(case["id"])
        row["original_fixture_observation"] = observation
        row["original_fixture_observation_current"] = bool(runtime_fresh and observation and observation["status"] == "PASSED")
        native = bool(row["original_fixture_observation_current"] and observation["native_calls"] > 0)
        row["implementation_state"] = "integrated" if native else "discovered"
        row["implementation_scope"] = "Exact original fixture only; complete feature/domain and supported configurations remain unresolved."
        row["required_configurations"] = presets
        row["validated_configurations"] = []
        row["domain"] = {"fixture": case["fixture"], "test": case["test"], "full_api_domain": "UNRESOLVED"}
        row["initial_state_inputs_and_order"] = "Original fixture/setup/helpers and ordered body; existing assertion_mapping retained where available."
        row["numeric_rules"] = "Original source types and expression order; complete operation relation unresolved."
        row["status_reason"] = "Required proofs/configurations and whole-feature correspondence are still open."
        classification = reviewed.get(case["id"])
        category = classification["category"] if classification else "UNKNOWN"
        row["semantic_domain"] = category
        row["feature_ids"] = classification["feature_ids"] if classification else []
        row["classification_rationale"] = classification["rationale"] if classification else scope["unknown_default"]
        row["core_obligation_ids"] = []
        row["adapter_obligation_ids"] = []
        for domain in ({"WORLD_CORE": ["WORLD_CORE"], "HOST_ADAPTER": ["HOST_ADAPTER"],
                        "MIXED": ["WORLD_CORE", "HOST_ADAPTER"], "UNKNOWN": ["UNKNOWN"]}[category]):
            identity = case["id"] + "::" + domain
            host_only = domain == "HOST_ADAPTER"
            obligation = {"id": identity, "original_fixture_id": case["id"],
                          "source_sha256": case["source_sha256"], "semantic_domain": domain,
                          "feature_ids": row["feature_ids"] if not host_only else [],
                          "assertion_ids": classification.get("adapter_assertion_ids" if host_only else "core_assertion_ids", []) if classification else [],
                          "required_core_configurations": [] if host_only else scope["configurations"],
                          "current_result": "DEFERRED_BLUE_INTEGRATION" if host_only else "UNRESOLVED" if domain == "UNKNOWN" else "REQUIRED_EVIDENCE_INCOMPLETE",
                          "deferred_reason": classification["adapter_contract"]["deferred_reason"] if host_only else None}
            world_obligations.append(obligation)
            if domain == "WORLD_CORE": row["core_obligation_ids"].append(identity)
            if host_only: row["adapter_obligation_ids"].append(identity)
        rows.append(row)
    source_holes = [{"id": f"{latest['commit']}::{hole['source']}::{preset}::compiler-extraction",
                     "source": hole["source"], "configuration": preset, "status": "blocked", "reason": hole["reason"]}
                    for hole in latest["extraction_holes"] for preset in presets]
    result = {**prior, "schema": 2, "status": "INCOMPLETE_MIGRATION", "original_commit": latest["commit"],
              "source_inventory_sha256": sha(RECORDS / "source_inventory.json"), "obligations": rows,
              "source_configuration_obligations": source_holes,
              "full_feature_denominator": inventory["feature_denominator_status"],
              "configuration_matrix": presets,
              "framework_evidence_status": "BLOCKED" if framework_error else "VALIDATED_BY_EXISTING_AEGIS_READERS",
              "last_successful_framework_source_identity": prior.get("source_identity") or prior.get("last_successful_framework_source_identity"),
              "source_identity": None if framework_error else prior.get("source_identity"),
              "framework_blocker": framework_error,
              "world_scope_sha256": sha(RECORDS / "world_scope.json"),
              "world_milestone": scope["milestone"], "world_obligations": world_obligations,
              "world_features": [{**feature, "observed_runtime_witnesses": sorted(set(feature["required_runtime_witnesses"]) & witnesses),
                                  "current_result": "REQUIRED_EVIDENCE_INCOMPLETE", "required_core_configurations": scope["configurations"],
                                  "generation_provenance": {"record": "verification/migration/rust_transformations.json", "sha256": sha(RECORDS / "rust_transformations.json")}}
                                 for feature in features.values()],
              "world_feature_denominator": scope["feature_denominator_status"],
              "world_classification_counts": dict(Counter(row["semantic_domain"] for row in rows)),
              "core_observation_blocker": observation_error,
              "original_fixture_receipt_current": bool(runtime_fresh),
              "original_fixture_receipt_blocker": fixture_error or (None if runtime_fresh else "library or production/build input hashes differ"),
              "original_fixture_receipt": str(matrix_path.relative_to(ROOT)),
              "notice": "One canonical queue. Native fixture integration is not proof, full-domain equivalence, or acceptance across supported configurations."}
    validate_identities(result, inventory)
    atomic_write_json(QUEUE, result)
    print(json.dumps({"status": result["status"], "original_cases": len(rows), "source_configuration_holes": len(source_holes),
                      "current_native_fixture_passes": sum(row["implementation_state"] == "integrated" for row in rows),
                      "semantic_classifications": result["world_classification_counts"],
                      "framework_blocker": framework_error, "queue": str(QUEUE)}))


def validate_architecture(document):
    concerns = {row["id"] for row in document["concerns"]}
    components = {row["id"] for row in document["components"]}
    covered = set()
    for row in document["components"]:
        if not (ROOT / row["path"]).is_file():
            raise ValueError("missing component source path: " + row["path"])
    for view in document["views"]:
        if not set(view["concerns"]) <= concerns or not set(view["components"]) <= components:
            raise ValueError("unresolved architecture view reference")
        covered.update(view["concerns"])
    if covered != concerns:
        raise ValueError("architecture view leaves a declared concern uncovered")
    for row in document["relationships"]:
        if row["from"] not in components or row["to"] not in components:
            raise ValueError("unresolved architecture relationship")
    for row in [*document["decisions"], *document["stakeholders"]]:
        if not set(row["concerns"]) <= concerns:
            raise ValueError("unknown decision/stakeholder concern")
    scope = read(RECORDS / "world_scope.json")
    features = {feature["id"]: feature for feature in scope["features"]}
    decisions = {decision["id"] for decision in document["decisions"]}
    links = {row["id"]: row for row in document.get("world_feature_links", [])}
    if links.keys() != features.keys():
        raise ValueError("architecture omits a discovered world feature")
    for identity, link in links.items():
        feature = features[identity]
        if (link["production_paths"] != feature["production_paths"] or
                link["source_regions"] != feature["original_regions"] or
                link["witnesses"] != feature["required_runtime_witnesses"] or
                link["decision"] not in decisions or not (ROOT / link["record"]).is_file()):
            raise ValueError("contradictory world feature architecture link: " + identity)


def cpg_mapping():
    compiler = read(RECORDS / "compiler_geometry.json")
    methods = []
    for row in compiler["records"]:
        if not row["compiler_usr"] or row["return_type"] != "double":
            raise ValueError("unresolved compiler method identity")
        methods.append({"source_identity": f"{compiler['original_revisions'][-1]}::src/Vector3d.h::{row['compiler_usr']}::{row['signature']}",
                        "compiler_usr": row["compiler_usr"], "cpg_node_kind": "METHOD",
                        "property_mapping": {"FULL_NAME": row["original_symbol"], "SIGNATURE": row["signature"],
                                             "FILENAME": "src/Vector3d.h", "LINE_NUMBER": row["source_line"],
                                             "IS_EXTERNAL": False, "AST_PARENT_TYPE": "TYPE_DECL", "AST_PARENT_FULL_NAME": "Vector3d"}})
    document = {"schema": 1, "artifact_kind": "CPG mapping sidecar; not a conformant Code Property Graph",
                "specification": "https://cpg.joern.io/", "specification_version": "1.1",
                "schema_source_repository": "https://github.com/ShiftLeftSecurity/codepropertygraph.git",
                "schema_source_commit": "1735c289d7258d4ef02f6c39ce635581cca4f330",
                "joern_repository": "https://github.com/joernio/joern.git",
                "joern_declared_commit": "3218f990c258a29e5715198a94d776b7aeabfdff",
                "implementation_schema_match": "UNVERIFIED; check this before executing a frontend",
                "compiler_input_sha256": sha(RECORDS / "compiler_geometry.json"),
                "mapping_structure_checked": True, "joern_executed": False, "schema_conformance_assessed": False,
                "method_mapping_candidates": methods,
                "layers": [{"nodes": ["METHOD", "METHOD_PARAMETER_IN", "TYPE_DECL", "MEMBER", "CALL", "IDENTIFIER", "LITERAL", "RETURN"],
                            "edges": ["AST", "CFG", "CALL", "REF", "CDG", "REACHING_DEF"],
                            "generated_by_joern": False, "validated_graph_layers": [],
                            "reason": "No installed Joern frontend or complete original compile configuration. Scalar compiler IR exists separately."}],
                "extension_boundary": "Destiny contracts, Bevy mappings and evidence stay in sidecars. No graph-local IDs are used as stable source identities.",
                "unresolved": ["Matching implementation/schema validation", "Full compiler configuration and macro provenance", "All graph layer/cardinality checks", "78 ambiguous Doxygen identity joins"],
                "regenerate": "python verification/tools/update_migration.py cpg"}
    (RECORDS / "cpg_mapping.json").write_text(json.dumps(document, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"status": "CPG_MAPPING_ONLY", "method_candidates": len(methods), "validated_graph_layers": 0, "joern_executed": False}))


def architecture(check=False):
    source = RECORDS / "architecture.json"
    document = read(source)
    validate_architecture(document)
    components = {row["id"]: row for row in document["components"]}
    prefix = f"<!-- Generated by verification/tools/update_migration.py architecture; source SHA-256 {sha(source)}; generator SHA-256 {sha(Path(__file__))}. -->\n\n"
    outputs = {}
    for view in document["views"]:
        body = [prefix, "# " + view["name"], "", document["profile"], "", "Scope: " + document["scope"], "",
                f"View `{view['id']}`; model kind: {view['model_kind']}. Concerns: " + ", ".join(view["concerns"]) + ".", "", view["description"], "",
                "| Component | Source | Responsibility |", "| --- | --- | --- |"]
        for identity in view["components"]:
            component = components[identity]
            body.append(f"| {identity} | [{component['path']}](../../{component['path']}) | {component['role']} ({component['fact']}) |")
        body.extend(["", "Relationships:", ""])
        for relationship in document["relationships"]:
            if relationship["from"] in view["components"] and relationship["to"] in view["components"]:
                body.append(f"- `{relationship['from']}` → `{relationship['to']}`: {relationship['relation']}.")
        if view["id"] == "V-state":
            body.extend(["", "Source angular state ownership (partial world-field inventory):", "", "| Field | Owner | Mutation | Snapshot |", "| --- | --- | --- | --- |"])
            for row in document["world_state_records"]:
                body.append(f"| {row['field']} | {row['owner']} | {row['mutation']} | {row['snapshot']} |")
        if view["id"] in {"V-overview", "V-verification"}:
            body.extend(["", "Discovered core feature links (not the complete feature denominator):", ""])
            for row in document["world_feature_links"]:
                body.append(f"- `{row['id']}`: original {', '.join(row['source_regions'])}; production {', '.join(row['production_paths']) or 'unresolved'}; named checks {', '.join(row['witnesses']) or 'unresolved'}; {row['decision']}. Source-bound record: [{row['record']}](../../{row['record']}).")
        body.extend(["", "Open obligations:", "", *["- " + issue for issue in document["unresolved"]], ""])
        outputs[ROOT / "docs/architecture" / view["file"]] = "\n".join(body)
    for decision in document["decisions"]:
        text = [prefix, f"# {decision['id']}: {decision['title']}", "", "Status: " + decision["status"], "",
                "Concerns: " + ", ".join(decision["concerns"]), "", "## Decision", "", decision["decision"], "",
                "## Rationale", "", decision["rationale"], "", "## Alternatives", "", decision["alternatives"], "",
                "## Consequences", "", decision["consequences"], ""]
        outputs[ROOT / "docs/adr" / (decision["id"].lower() + ".md")] = "\n".join(text)
    for path, text in outputs.items():
        if check:
            if not path.is_file() or path.read_text() != text:
                raise ValueError("generated architecture drift: " + str(path))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    print(json.dumps({"status": "ARCHITECTURE_DRIFT_CHECK_PASSED" if check else "ARCHITECTURE_GENERATED", "views": len(document["views"]), "decisions": len(document["decisions"]), "conformance_claim": False}))


def milestone_obligations(current, scope):
    return [row for row in current["world_obligations"]
            if scope == "full" or row["semantic_domain"] != "HOST_ADAPTER"]


def core_readiness(current, generation_error):
    rows = milestone_obligations(current, "core")
    reasons = []
    unknown = sum(row["semantic_domain"] == "UNKNOWN" for row in rows)
    if unknown:
        reasons.append(f"{unknown} original fixtures have unresolved world/adapter semantics")
    # Neither a case list nor an edited COMPLETE flag establishes closure.
    # A compiler/configuration/state-effect inventory reader is still missing.
    reasons.append("complete source-derived world feature/state/effect/configuration inventory and its closure validation remain unimplemented")
    if current["source_configuration_obligations"]:
        reasons.append("source/configuration extraction holes have not been mapped to and resolved for the core")
    if generation_error:
        reasons.append("production generation drift: " + generation_error)
    if current["core_observation_blocker"]:
        reasons.append(current["core_observation_blocker"])
    missing_paths, missing_witnesses, open_features = [], [], []
    for feature in current["world_features"]:
        if not feature["production_paths"] or any(not (ROOT / path).is_file() for path in feature["production_paths"]):
            missing_paths.append(feature["id"])
        required = set(feature["required_runtime_witnesses"])
        if not required or not required <= set(feature["observed_runtime_witnesses"]):
            missing_witnesses.append(feature["id"])
        if feature["open_semantics"]:
            open_features.append(feature["id"])
    if missing_paths:
        reasons.append("missing production implementation paths: " + ", ".join(missing_paths))
    if missing_witnesses:
        reasons.append("missing current production execution witnesses: " + ", ".join(missing_witnesses))
    if open_features:
        reasons.append("known incomplete feature semantics: " + ", ".join(open_features))
    if current["framework_evidence_status"] == "BLOCKED":
        reasons.append("required core formal evidence reader blocked: " + current["framework_blocker"])
    reasons.extend([
        "feature-to-production refinement, safety, non-vacuity and supported core configuration evidence are not complete",
        "core API/lifecycle, interaction/long-run fidelity, robustness, resource/performance and engine release receipt validation remain incomplete",
    ])
    return {"schema": 2, "scope": "core", "status": "INCOMPLETE" if reasons else "ACCEPTED", "complete": not reasons,
            "original_commit": current["original_commit"], "counts": {
                "original_test_definitions": len(current["obligations"]),
                "semantic_classifications": current["world_classification_counts"],
                "required_world_obligations_including_unknown": len(rows),
                "deferred_adapter_obligations": len(current["world_obligations"]) - len(rows),
                "discovered_core_features_not_total_denominator": len(current["world_features"]),
                "observed_core_features_with_all_named_witnesses": sum(bool(feature["required_runtime_witnesses"]) and set(feature["required_runtime_witnesses"]) <= set(feature["observed_runtime_witnesses"]) for feature in current["world_features"]),
                "unresolved_source_configuration_pairs": len(current["source_configuration_obligations"]),
            }, "reasons": reasons, "full_feature_denominator": current["world_feature_denominator"],
            "core_configuration_matrix_status": "UNRESOLVED_SEMANTIC_MAPPING",
            "core_proof_denominator_status": "UNRECONCILED_FEATURE_TO_PRODUCTION_BINDINGS",
            "deferred_integration_contract": read(RECORDS / "world_scope.json")["deferred_host_contract"],
            "notice": "Core milestone only. Unknowns cannot be deferred; original Blue hosting/release failures are excluded from core routing, while their world state/transition obligations and all mandatory core proofs remain required. This checker currently rejects completion because full inventory and production-quality evidence readers are incomplete."}


def acceptance(scope="full"):
    # Refresh through the existing validators instead of trusting edited/cached
    # queue flags or an old source identity at acceptance time.
    queue()
    inventory, current = read(RECORDS / "source_inventory.json"), read(QUEUE)
    validate_identities(current, inventory)
    rows = current["obligations"]
    reasons = []
    generation_error = None
    try:
        from tools.migrate_destiny import generate
        generate(check=True)
    except (ValueError, RuntimeError, OSError) as error:
        generation_error = str(error)
        reasons.append("production generation drift: " + generation_error)
    if current["source_inventory_sha256"] != sha(RECORDS / "source_inventory.json"):
        reasons.append("source inventory digest changed")
    if current["full_feature_denominator"] != "COMPLETE_COMPILER_AND_CONFIGURATION_INVENTORY":
        reasons.append("complete feature/configuration denominator is unresolved")
    if current["source_configuration_obligations"]:
        reasons.append("compiler extraction is incomplete for supported source/configuration pairs")
    if current["framework_evidence_status"] == "BLOCKED":
        reasons.append("existing formal evidence validator blocked: " + current["framework_blocker"])
    counts = {"inventoried_test_cases": len(rows), "required_configurations": len(current["configuration_matrix"]),
              "test_configuration_pairs": sum(len(row["required_configurations"]) for row in rows),
              "native_fixture_integrated": sum(row["implementation_state"] == "integrated" for row in rows),
              "fixture_assertions_passed": sum(row["original_fixture_observation_current"] for row in rows),
              "spec_proof_credited": sum(row["assurance"].get("spec_proved", False) for row in rows),
              "production_refinement_credited": sum(row["assurance"].get("production_rust_refinement_proved", False) for row in rows),
              "all_required_configurations_validated": sum(row["validated_configurations"] == row["required_configurations"] for row in rows)}
    for key in ("spec_proof_credited", "production_refinement_credited", "all_required_configurations_validated"):
        if counts[key] != len(rows):
            reasons.append(f"{key}: {counts[key]}/{len(rows)}")
    if counts["fixture_assertions_passed"] != len(rows):
        reasons.append("not every original fixture executed and passed on the current implementation")
    if not all(row["assurance"].get("runtime_differential_evidence", False) for row in rows):
        reasons.append("full original-engine differential and ABI/private serialization correspondence evidence is incomplete")
    if not current.get("validated_release_matrix_receipts"):
        reasons.append("required Windows/macOS, wheel, supply-chain/archive and authenticated transport release matrix has no validated receipts")
    full_world = milestone_obligations(current, "full")
    if any(row["current_result"] == "DEFERRED_BLUE_INTEGRATION" for row in full_world):
        reasons.append("preserved original-host adapter obligations are deferred until after core acceptance")
    report = {"schema": 2, "scope": "full", "status": "INCOMPLETE" if reasons else "ACCEPTED", "complete": not reasons,
              "original_commit": current["original_commit"], "counts": counts, "reasons": reasons,
              "full_feature_denominator": current["full_feature_denominator"], "explicit_exclusions": [],
              "queue_sha256": sha(QUEUE), "source_inventory_sha256": sha(RECORDS / "source_inventory.json"),
              "checker_sha256": sha(Path(__file__)), "world_scope_sha256": sha(RECORDS / "world_scope.json"),
              "notice": "Fixture, kernel, documentation, formal and release categories remain separate. No subset is labeled a complete migration."}
    atomic_write_json(STATE / "acceptance.json", report)
    atomic_write_json(STATE / "full-acceptance.json", report)
    core = core_readiness(current, generation_error)
    for key in ("queue_sha256", "source_inventory_sha256", "checker_sha256"):
        core[key] = report[key]
    core["world_scope_sha256"] = sha(RECORDS / "world_scope.json")
    atomic_write_json(STATE / "core-acceptance.json", core)
    print(json.dumps({"core": core, "full": report} if scope == "both" else core if scope == "core" else report))
    return 1 if (core["reasons"] or report["reasons"] if scope == "both" else core["reasons"] if scope == "core" else report["reasons"]) else 0


def self_test():
    inventory, current = read(RECORDS / "source_inventory.json"), read(QUEUE)
    incomplete = copy.deepcopy(current)
    incomplete["obligations"].pop()
    try:
        validate_identities(incomplete, inventory)
    except ValueError:
        pass
    else:
        raise AssertionError("incomplete manifest accepted")
    matrix = read(ROOT / current["original_fixture_receipt"])
    stale = copy.deepcopy(matrix)
    stale["original_commit"] = inventory["legacy_commit"]
    try:
        validate_fixture_scope(stale, inventory["revisions"][-1], sha(RECORDS / "source_inventory.json"))
    except ValueError:
        pass
    else:
        raise AssertionError("stale fixture source authority accepted")
    architecture_data = read(RECORDS / "architecture.json")
    invalid = copy.deepcopy(architecture_data)
    invalid["views"][0]["components"].append("UNDECLARED")
    try:
        validate_architecture(invalid)
    except ValueError:
        pass
    else:
        raise AssertionError("unknown architecture component accepted")
    scope = read(RECORDS / "world_scope.json")
    validate_world_scope(scope, inventory)
    mixed = next(row for row in scope["case_classifications"] if row["category"] == "MIXED")
    bad_scopes = []
    omitted = copy.deepcopy(scope)
    omitted["features"] = [feature for feature in omitted["features"] if feature["id"] != mixed["feature_ids"][0]]
    bad_scopes.append(("omitted_core_mechanic", omitted))
    stale_source = copy.deepcopy(scope)
    stale_source["case_classifications"][0]["source_sha256"] = "0" * 64
    bad_scopes.append(("stale_core_source", stale_source))
    deferred = copy.deepcopy(scope)
    wrong = next(row for row in deferred["case_classifications"] if row["id"] == mixed["id"])
    wrong.update(category="HOST_ADAPTER", feature_ids=[], core_assertion_ids=[], adapter_assertion_ids=[item["id"] for item in wrong["assertions"]])
    # Update audit as an adversarial edited classification; assertion semantics
    # must still reject the attempt, even with a matching superficial audit.
    next(audit for audit in deferred["classification_audit"] if audit["id"] == mixed["id"])["to"] = "HOST_ADAPTER"
    bad_scopes.append(("mixed_fixture_wholesale_deferred", deferred))
    for label, candidate in bad_scopes:
        try:
            validate_world_scope(candidate, inventory)
        except ValueError:
            continue
        raise AssertionError(label + " accepted")
    missing = copy.deepcopy(current)
    feature = next(row for row in missing["world_features"] if row["required_runtime_witnesses"])
    feature["observed_runtime_witnesses"] = []
    if not any("missing current production execution witnesses" in reason and feature["id"] in reason for reason in core_readiness(missing, None)["reasons"]):
        raise AssertionError("missing production witness accepted")
    # Exercise routing on a hypothetical discharged core and unmet host row.
    # This does not fabricate a complete engine or bypass any acceptance gate.
    routing = {"world_obligations": [
        {"semantic_domain": "WORLD_CORE", "current_result": "VALIDATED"},
        {"semantic_domain": "HOST_ADAPTER", "current_result": "DEFERRED_BLUE_INTEGRATION"},
    ]}
    assert all(row["current_result"] == "VALIDATED" for row in milestone_obligations(routing, "core"))
    assert not all(row["current_result"] == "VALIDATED" for row in milestone_obligations(routing, "full"))
    report = {"status": "NEGATIVE_CHECKS_PASSED", "incomplete_manifest_rejected": True, "stale_fixture_authority_rejected": True, "unknown_architecture_reference_rejected": True,
              **{label + "_rejected": True for label, _ in bad_scopes}, "missing_production_witness_rejected": True,
              "blue_only_failure_routing_checked": True, "proof_credit": False,
              "source_inventory_sha256": sha(RECORDS / "source_inventory.json"), "world_scope_sha256": sha(RECORDS / "world_scope.json"),
              "canonical_queue_sha256": sha(QUEUE), "checker_sha256": sha(Path(__file__)),
              "scope": "Adversarial rejection of the current source-bound partial inventory, plus milestone-routing isolation. This is not complete source closure or an engine acceptance result."}
    atomic_write_json(STATE / "world-scope-negative-checks.json", report)
    print(json.dumps(report))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("queue", "architecture", "acceptance", "self-test", "cpg", "observe-core"))
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--scope", choices=("core", "full", "both"), default="full", help="Acceptance milestone; both reports are always retained from the same queue")
    args = parser.parse_args()
    STATE.mkdir(parents=True, exist_ok=True)
    lock = FileLock(STATE / "canonical-writer.lock", purpose="sole canonical Destiny migration queue and architecture writer")
    lock.acquire(timeout=0)
    try:
        if args.command == "queue": queue()
        elif args.command == "architecture": architecture(args.check)
        elif args.command == "self-test": self_test()
        elif args.command == "cpg": cpg_mapping()
        elif args.command == "observe-core": observe_core()
        else: sys.exit(acceptance(args.scope))
    finally:
        lock.release()
