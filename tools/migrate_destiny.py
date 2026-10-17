#!/usr/bin/env python3
"""Source-bound migration entry points; uses the existing Aegis process runner.

No source inventory, documentation record, generation, or runtime receipt is proof.
Consumers build tracked Rust normally; only maintainers need extraction tools.
"""
from __future__ import annotations

import argparse
import ast
from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / ".aegis/destiny/migration"
RECORDS = ROOT / "verification/migration"
LATEST = "114e89fa584f0e1a81eb0a679aabf41af8870072"
LEGACY = "ef738406b61ad06bae2f402857e464735ab106a2"
REVISIONS = (LEGACY, LATEST)
RUST_BASE = "8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268"
sys.path.insert(0, str(ROOT / ".agents/infra"))
from agentinfra.atomic import atomic_write_json, atomic_write_bytes
from agentinfra.process import run_process


def sha(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(json_bytes(value))


def git(*arguments):
    return subprocess.run(["git", "--no-replace-objects", *arguments], cwd=ROOT,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True).stdout


def blob(revision, path):
    return git("show", f"{revision}:{path}")


def checked(arguments, name, timeout=120):
    """Use Aegis supervision and retain full, bounded execution diagnostics."""
    STATE.mkdir(parents=True, exist_ok=True)
    environment = {key: os.environ[key] for key in
                   ("PATH", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN") if key in os.environ}
    environment.setdefault("CARGO_HOME", str(Path.home() / ".cargo"))
    environment.setdefault("RUSTUP_HOME", str(Path.home() / ".rustup"))
    result = run_process([str(a) for a in arguments], cwd=ROOT, timeout=timeout,
                         env=environment, capture_limit=8_000_000)
    runs = STATE / "runs"
    runs.mkdir(exist_ok=True)
    number = len(list(runs.glob(f"{name}-*.json"))) + 1
    atomic_write_json(runs / f"{name}-{number}.json", asdict(result))
    atomic_write_json(STATE / f"{name}-execution.json", asdict(result))
    if result.returncode or result.timed_out or result.stdout_truncated or result.stderr_truncated:
        raise RuntimeError(f"{name} failed; see {STATE / (name + '-execution.json')}: {result.stderr[-1600:]}")
    return result.stdout


def entries(revision):
    rows = []
    for item in git("ls-tree", "-rz", "--full-tree", revision).split(b"\0"):
        if not item:
            continue
        metadata, path = item.split(b"\t", 1)
        mode, kind, identity = metadata.decode().split()
        rows.append({"path": path.decode(), "mode": mode, "kind": kind, "git_object": identity})
    return rows


def inventory():
    """Inventory whole trees and tests independently of the legacy title catalog."""
    snapshots = []
    for revision in REVISIONS:
        files, cases, declarations, holes = [], [], [], []
        for item in entries(revision):
            path = item["path"]
            row = dict(item)
            if item["kind"] != "blob":
                row["dependency_source_status"] = "PINNED_GITLINK_NOT_A_SOURCE_EXTRACTION"
                files.append(row)
                continue
            content = blob(revision, path)
            row["sha256"] = sha(content)
            row["bytes"] = len(content)
            files.append(row)
            if path.endswith(".py"):
                try:
                    tree = ast.parse(content, filename=path)
                    def walk(nodes, scope=()):
                        for node in nodes:
                            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
                                qualified = "::".join((*scope, node.name))
                                record = {"id": f"{revision}::{path}::{qualified}", "source": path,
                                          "qualified_name": qualified, "line": node.lineno,
                                          "end_line": node.end_lineno, "kind": type(node).__name__,
                                          "source_sha256": row["sha256"], "fact_kind": "extracted_python_ast"}
                                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                                    record["signature_ast"] = ast.dump(node.args, include_attributes=False)
                                    if node.name.startswith("test_") and scope:
                                        cases.append({"id": f"{path}::{'::'.join(scope)}::{node.name}",
                                                      "fixture": "::".join(scope), "test": node.name,
                                                      "line": node.lineno, "source": path,
                                                      "source_sha256": row["sha256"]})
                                declarations.append(record)
                                walk(node.body, (*scope, node.name))
                    walk(tree.body)
                except (SyntaxError, ValueError) as error:
                    holes.append({"source": path, "reason": str(error), "kind": "python_parse_failure"})
            elif path.endswith((".cpp", ".h", ".hpp", ".c", ".cc")):
                text = content.decode("utf-8", errors="replace")
                for match in re.finditer(r"\bTEST(?:_F|_P)?\s*\(\s*(\w+)\s*,\s*(\w+)\s*\)", text):
                    cases.append({"id": f"{path}::{match[1]}::{match[2]}", "fixture": match[1],
                                  "test": match[2], "source": path,
                                  "line": text.count("\n", 0, match.start()) + 1,
                                  "source_sha256": row["sha256"], "extraction": "lexical_macro_candidate"})
                holes.append({"source": path, "kind": "compiler_configuration_required",
                              "reason": "Full original build/ABI and dependencies must be configured; a lexical test scan does not extract C++ semantics."})
        ids = [case["id"] for case in cases]
        if len(ids) != len(set(ids)):
            raise ValueError("duplicate fixture/case identity; parameterization review required")
        presets = json.loads(blob(revision, "CMakePresets.json"))
        snapshots.append({"commit": revision, "git_tree": git("rev-parse", revision + "^{tree}").decode().strip(),
                          "source_tree_sha256": sha(json_bytes(files)), "files": files,
                          "cases": sorted(cases, key=lambda c: c["id"]), "python_declarations": declarations,
                          "extraction_holes": holes, "configure_presets": presets["configurePresets"],
                          "dependencies": {p: json.loads(blob(revision, p)) for p in ("vcpkg.json", "vcpkg-configuration.json")}})
    legacy_ids = {case["id"] for case in snapshots[0]["cases"]}
    latest_ids = {case["id"] for case in snapshots[1]["cases"]}
    if legacy_ids - latest_ids:
        raise ValueError("new original revision removed legacy cases; retain explicit obligations")
    document = {"schema": 1, "generator_sha256": sha(Path(__file__).read_bytes()), "repository": "https://github.com/QuasarRay/destiny.git",
                "authoritative_commit": LATEST, "authority_decision": "User requested newest Original Destiny, preserving all Rust work and earlier obligations.",
                "legacy_commit": LEGACY, "revisions": snapshots,
                "retained_legacy_cases": len(legacy_ids), "latest_cases": len(latest_ids),
                "new_cases": sorted(latest_ids - legacy_ids), "removed_cases": sorted(legacy_ids - latest_ids),
                "feature_denominator_status": "UNRESOLVED_COMPILER_AND_CONFIG_EXTRACTION",
                "notice": "Whole source/file inventory and test candidates are not a complete behavioral feature inventory, implementation, or proof."}
    save(RECORDS / "source_inventory.json", document)
    print(json.dumps({key: document[key] for key in ("authoritative_commit", "retained_legacy_cases", "latest_cases", "new_cases", "feature_denominator_status")}))


def extractor_build():
    source = ROOT / "verification/tools/extract_cpp_semantics.cpp"
    executable = STATE / "extract-cpp-semantics"
    llvm = shlex.split(checked(["llvm-config", "--ldflags", "--libs", "--system-libs"], "llvm-link-flags"))
    checked(["clang++", "-std=c++17", "-O1", source, "-lclang-cpp", *llvm, "-o", executable], "libtooling-build", 180)
    return executable


def extract():
    source = blob(LATEST, "src/Vector3d.h")
    if source != blob(LEGACY, "src/Vector3d.h"):
        raise ValueError("shared semantic source differs across revisions")
    original = STATE / "original" / LATEST
    (original / "src").mkdir(parents=True, exist_ok=True)
    (original / "src/Vector3d.h").write_bytes(source)
    # Only the external Vector3 declaration is needed to parse unused overloads.
    # Length/LengthSq use exclusively original Vector3d double fields and sqrt.
    adapter = ('#include <cmath>\n'
               'struct Vector3 { float x,y,z; Vector3(float,float,float); };\n'
               '#include "src/Vector3d.h"\n'
               'extern "C" double destiny_oracle_vector_length(double x,double y,double z) {\n'
               ' return Vector3d(x,y,z).Length();\n}\n')
    adapter_path = original / "geometry_oracle.cpp"
    adapter_path.write_text(adapter)
    resource_dir = checked(["clang++", "-print-resource-dir"], "clang-resource-dir").strip()
    flags = ["-std=c++17", "-fno-fast-math", "-ffp-contract=off", "-resource-dir=" + resource_dir]
    configuration = {"scope": "isolated_original_Vector3d_header; not full_original_build",
                     "compiler": checked(["clang++", "--version"], "clang-version"),
                     "flags": flags, "target": checked(["clang++", "-dumpmachine"], "clang-target").strip(),
                     "adapter_sha256": sha(adapter.encode()),
                     "external_Vector3": "declaration-only parsing adapter for unused overloads; no selected method reads it",
                     "numeric_policy": "IEEE binary64 expression order; no fast math or contraction; host sqrt"}
    database = [{"directory": str(original), "file": str(adapter_path),
                 "arguments": ["clang++", *flags, "-c", str(adapter_path)]}]
    save(original / "compile_commands.json", database)
    executable = extractor_build()
    compiler_records = json.loads(checked([executable, "-p", original, adapter_path], "libtooling-extract"))
    save(RECORDS / "compiler_geometry.json", {"schema": 1, "source": "src/Vector3d.h",
          "source_sha256": sha(source), "original_revisions": list(REVISIONS),
          "configuration": configuration, "records": compiler_records})
    manifest = {"schema": 1, "source_sha256": sha(source), "original_revisions": list(REVISIONS),
                "extractor_sha256": sha((ROOT / "verification/tools/extract_cpp_semantics.cpp").read_bytes()),
                "configuration_sha256": sha(json_bytes(configuration)), "rule_version": "scalar-expression-v1",
                "generator_sha256": generator_identity(),
                "functions": [{k: v for k, v in record.items() if k not in ("compiler_usr", "signature", "source_line")}
                              for record in compiler_records if record["name"] == "vector_length"],
                "reuse": {"type_path": "destiny_original_spec::Vec3", "method": "length",
                          "sources": ["verification/spec/src/geometry.rs", "verification/spec/src/model.rs"],
                          "fields": {"x": 0, "y": 1, "z": 2}}}
    save(RECORDS / "geometry_semantics.json", manifest)
    checked(["clang++", *flags, "-shared", "-fPIC", "-O2", adapter_path,
             "-o", STATE / "liboriginal_geometry.so"], "original-geometry-oracle")
    help_text = checked(["clang-doc", "--help"], "clang-doc-help")
    if "=json" not in help_text:
        raise RuntimeError("installed clang-doc lacks required machine-readable JSON mode")
    checked(["clang-doc", "-p", original, "--format=json", "--output=" + str(STATE / "clang-doc"),
             "--pretty-json", "--project-name=Original-Destiny", adapter_path], "clang-doc-extract")
    print(json.dumps({"status": "EXTRACTED_SCOPED", "functions": len(compiler_records),
                      "full_original_build": False, "clang_doc_format": "json"}))


def generator_identity():
    paths = ["tools/migrate_destiny.py", "verification/codegen/src/main.rs",
             "verification/codegen/src/kinematics.rs", "verification/migration/kinematic_rules.json", "verification/migration/kinematic_source_evidence.json",
             "verification/codegen/src/contacts.rs", "verification/migration/contact_rules.json", "verification/migration/contact_source_evidence.json",
             "verification/codegen/src/forces.rs", "verification/migration/force_rules.json", "verification/migration/force_source_evidence.json",
             "verification/codegen/src/liveness.rs", "verification/migration/liveness_rules.json", "verification/migration/liveness_source_evidence.json",
             "verification/codegen/src/partition.rs", "verification/codegen/src/state_rules.rs",
             "verification/migration/partition_rules.json", "verification/migration/partition_source_evidence.json",
             "verification/codegen/src/angular.rs", "verification/codegen/src/speed.rs", "verification/codegen/src/warp.rs", "verification/codegen/src/missile.rs", "verification/migration/missile_source_evidence.json", "verification/migration/warp_alignment_source_evidence.json", "verification/migration/speed_source_evidence.json", "verification/migration/angular_rules.json",
             "verification/migration/angular_source_evidence.json", "verification/spec/src/original_rotation.rs",
             "verification/migration/angular_oracle_samples.json", "verification/tools/check_angular_migration.py",
             "verification/codegen/Cargo.toml", "verification/codegen/Cargo.lock",
             "verification/spec/src/geometry.rs", "verification/spec/src/model.rs",
             "verification/spec/Cargo.toml", "verification/spec/Cargo.lock",
             "verification/spec/src/original_park.rs", "verification/migration/uncloak_rule.json",
             "verification/migration/uncloak_source_evidence.json",
             "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"]
    return sha(json_bytes({p: sha((ROOT / p).read_bytes()) for p in paths}))


def generate(check=False):
    unit = json.loads((RECORDS / "geometry_semantics.json").read_text())
    if unit["source_sha256"] != sha(blob(LATEST, "src/Vector3d.h")) or unit["generator_sha256"] != generator_identity() or unit["extractor_sha256"] != sha((ROOT / "verification/tools/extract_cpp_semantics.cpp").read_bytes()):
        raise ValueError("semantic source/generator drift; extract again and review the changed inputs")
    for evidence_file in ("uncloak_source_evidence.json", "angular_source_evidence.json", "speed_source_evidence.json", "warp_alignment_source_evidence.json", "missile_source_evidence.json", "partition_source_evidence.json", "liveness_source_evidence.json", "force_source_evidence.json", "contact_source_evidence.json", "kinematic_source_evidence.json"):
        state_evidence = json.loads((RECORDS / evidence_file).read_text())
        if state_evidence["original_commit"] != LATEST:
            raise ValueError("state rule source authority differs")
        for row in state_evidence["original_regions"]:
            source = blob(LATEST, row["source"])
            lines = source.splitlines(keepends=True)
            if sha(source) != row["source_sha256"] or sha(b"".join(lines[row["start"] - 1:row["end"]])) != row["region_sha256"]:
                raise ValueError("original state rule evidence changed")
        for path, digest in state_evidence["reused_sources"].items():
            if sha((ROOT / path).read_bytes()) != digest:
                raise ValueError("reused state guard source changed")
        for path, digest in state_evidence.get("reused_git_sources", {}).items():
            if sha(blob(RUST_BASE, path)) != digest:
                raise ValueError("immutable reused Rust source changed")
    checked(["cargo", "build", "--manifest-path", "verification/codegen/Cargo.toml", "--locked", "--offline"], "codegen-build", 180)
    binary = ROOT / "verification/codegen/target/debug/destiny-semantic-codegen"
    prior_generation = json.loads((STATE / "generation.json").read_text()) if (STATE / "generation.json").exists() else {}
    with tempfile.TemporaryDirectory(prefix="regeneration-", dir=STATE) as directory:
        staging = Path(directory)
        original_inputs = {}
        for target in ("src/runtime.rs", "src/lib.rs", "src/network.rs", "src/bubbles.rs", "src/iterative_collision.rs", "src/noniterative_collision.rs", "verification/kani/src/iterative_sphere_production_contracts.rs", "verification/kani/src/noniterative_production_contracts.rs", "verification/spec/src/original_partition.rs", "verification/spec/src/lib.rs"):
            original_inputs[target] = staging / "preserved-base" / target
            original_inputs[target].parent.mkdir(parents=True, exist_ok=True)
            original_inputs[target].write_bytes(blob(RUST_BASE, target))
        outputs = {"verification/kani/src/iterative_sphere_production_contracts.rs": ("liveness-fixture", original_inputs["verification/kani/src/iterative_sphere_production_contracts.rs"]),
                   "verification/kani/src/noniterative_production_contracts.rs": ("liveness-fixture", original_inputs["verification/kani/src/noniterative_production_contracts.rs"]),
                   "verification/spec/src/original_partition.rs": ("partition-spec", original_inputs["verification/spec/src/original_partition.rs"]),
                   "src/iterative_collision.rs": ("liveness-iterative", original_inputs["src/iterative_collision.rs"]),
                   "src/noniterative_collision.rs": ("liveness-noniterative", original_inputs["src/noniterative_collision.rs"]),
                   "src/generated_liveness.rs": ("liveness-emit", None),
                   "src/generated_forces.rs": ("force-emit", None),
                   "src/generated_contacts.rs": ("contact-emit", None),
                   "src/generated_kinematics.rs": ("kinematic-emit", None),
                   "verification/spec/src/lib.rs": ("partition-spec-lib", original_inputs["verification/spec/src/lib.rs"]),
                   "src/bubbles.rs": ("partition-bubbles", original_inputs["src/bubbles.rs"]),
                   "src/generated_partition.rs": ("partition-emit", None),
                   "src/generated_geometry.rs": ("emit", RECORDS / "geometry_semantics.json"),
                   "src/generated_angular.rs": ("angular-emit", RECORDS / "angular_rules.json"),
                   "src/generated_missile.rs": ("missile-emit", None),
                   "src/runtime.rs": ("transform", original_inputs["src/runtime.rs"]),
                   "src/network.rs": ("speed-network", original_inputs["src/network.rs"]),
                   "src/lib.rs": ("module", original_inputs["src/lib.rs"])}
        oracle_report = RECORDS / "geometry_oracle_samples.json"
        samples = json.loads(oracle_report.read_text())
        if samples["samples_sha256"] != sha(json_bytes(samples["rows"])) or samples["source_sha256"] != unit["source_sha256"] or samples["configuration_sha256"] != unit["configuration_sha256"]:
            raise ValueError("independent oracle sample source/configuration drift")
        outputs["verification/kani/src/generated_geometry_contracts.rs"] = ("harness", oracle_report)
        changes = []
        transformed_records = {}
        for target, (mode, input_path) in outputs.items():
            output = staging / target
            output.parent.mkdir(parents=True, exist_ok=True)
            arguments = [binary, mode] if input_path is None else [binary, mode, input_path]
            if mode == "harness":
                arguments.append(RECORDS / "geometry_semantics.json")
            checked([*arguments, output], "generate-" + Path(target).stem)
            if target == "src/runtime.rs":
                checked([binary, "uncloak", output, RECORDS / "uncloak_rule.json", output], "generate-uncloak-guard")
                checked([binary, "angular-runtime", output, output], "generate-angular-runtime")
                checked([binary, "speed-runtime", output, output], "generate-speed-runtime")
                checked([binary, "warp-runtime", output, output], "generate-warp-runtime")
                checked([binary, "missile-runtime", output, output], "generate-missile-runtime")
                checked([binary, "partition-runtime", output, output], "generate-partition-runtime")
                checked([binary, "liveness-runtime", output, output], "generate-liveness-runtime")
                checked([binary, "force-runtime", output, output], "generate-force-runtime")
                checked([binary,"contact-runtime",output,output],"generate-contact-runtime")
                checked([binary,"kinematic-runtime",output,output],"generate-kinematic-runtime")
            if target=="src/noniterative_collision.rs":
                checked([binary,"force-gradient",output,output],"generate-force-gradient")
                checked([binary,"contact-gradient",output,output],"generate-contact-gradient")
            if target=="verification/kani/src/noniterative_production_contracts.rs":
                checked([binary,"force-fixture",output,output],"generate-force-fixture")
                checked([binary,"contact-gradient-fixture",output,output],"generate-contact-gradient-fixture")
            if target=="src/iterative_collision.rs":
                checked([binary,"contact-iterative",output,output],"generate-contact-iterative")
            if target=="verification/kani/src/iterative_sphere_production_contracts.rs":
                checked([binary,"contact-iterative-fixture",output,output],"generate-contact-iterative-fixture")
            if target=="src/generated_liveness.rs":
                checked([binary,"contact-frame",output,output],"generate-contact-frame")
            if target=="src/bubbles.rs":
                checked([binary,"liveness-bubbles",output,output],"generate-liveness-bubbles")
            if target == "src/lib.rs":
                with output.open("a") as stream:
                    stream.write("\nmod generated_angular;\npub use generated_angular::{Angles as DestinyAngularState, RollParameters as DestinyRollParameters};\n")
                    stream.write("\npub use runtime::DestinyMaxVelocity;\n")
                    stream.write("\nmod generated_missile;\n")
                    stream.write("\nmod generated_partition;\n")
                    stream.write("\nmod generated_liveness;\npub use generated_liveness::LivenessState as DestinyLivenessState;\n")
                    stream.write("\nmod generated_forces;\npub use generated_forces::Forces as DestinyForceState;\n")
                    stream.write("\nmod generated_contacts;\npub use generated_contacts::{CollisionEvent as DestinyCollisionEvent, CollisionNotice as DestinyCollisionNotice, CollisionPass as DestinyCollisionPass};\n")
                    stream.write("\nmod generated_kinematics;\npub use generated_kinematics::Pose as DestinyOldPose;\n")
                    stream.write('\n#[cfg(test)]\n#[path = "../verification/kani/src/iterative_sphere_production_contracts.rs"]\nmod concrete_iterative_kernel_fixtures;\n#[cfg(test)]\n#[path = "../verification/kani/src/noniterative_production_contracts.rs"]\nmod concrete_noniterative_kernel_fixtures;\n')
            provenance = (f"// Migration source: {LATEST}; preserved Rust base: {RUST_BASE}.\n"
                          f"// Generator SHA-256: {generator_identity()}.\n"
                          f"// Semantic bundle SHA-256: {sha((RECORDS / 'geometry_semantics.json').read_bytes())}.\n"
                          "// Source/rule/configuration mappings: verification/migration/rust_transformations.json.\n"
                          "// Regenerate: python tools/migrate_destiny.py generate\n")
            output.write_text(provenance + output.read_text())
            checked(["rustfmt", "--edition=2024", "--config", "skip_children=true", output], "format-" + Path(target).stem)
            live = ROOT / target
            transformed_records[target] = {"output_sha256": sha(output.read_bytes()), "mode": mode}
            if target in original_inputs:
                transformed_records[target]["preserved_base_sha256"] = sha(original_inputs[target].read_bytes())
            if not live.exists() or live.read_bytes() != output.read_bytes():
                changes.append(target)
        if check and changes:
            raise ValueError("generated drift: " + ", ".join(changes))
        if not check:
            # Validate the complete staged batch before changing any live file.
            # Keep receipt-bound pending hashes for recovery from an I/O failure.
            for target in changes:
                live = ROOT / target
                permitted = {prior_generation.get("outputs", {}).get(target),
                             prior_generation.get("pending_outputs", {}).get(target)}
                if target in original_inputs:
                    permitted.add(sha(original_inputs[target].read_bytes()))
                if live.exists() and sha(live.read_bytes()) not in permitted:
                    raise ValueError("independent user edits detected; preserve and incorporate before regeneration: " + target)
            save(STATE / "generation.json", {**prior_generation,
                 "pending_outputs": {target: transformed_records[target]["output_sha256"] for target in changes},
                 "pending_generator_sha256": generator_identity()})
            for target in changes:
                atomic_write_bytes(ROOT / target, (staging / target).read_bytes())
    save(STATE / "generation.json", {"schema": 1, "status": "DRIFT_CHECK_PASSED" if check else "GENERATED",
         "generator_sha256": generator_identity(), "semantic_input_sha256": sha((RECORDS / "geometry_semantics.json").read_bytes()),
         "outputs": {path: sha((ROOT / path).read_bytes()) for path in outputs}, "changed": changes})
    save(RECORDS / "rust_transformations.json", {"schema": 1, "preserved_rust_base": RUST_BASE,
         "generator_sha256": generator_identity(), "semantic_input_sha256": sha((RECORDS / "geometry_semantics.json").read_bytes()),
         "state_rule_evidence_sha256": sha((RECORDS / "uncloak_source_evidence.json").read_bytes()),
         "source_evidence_sha256": {name: sha((RECORDS / name).read_bytes()) for name in
             ("uncloak_source_evidence.json", "angular_source_evidence.json", "speed_source_evidence.json", "warp_alignment_source_evidence.json", "missile_source_evidence.json", "partition_source_evidence.json", "liveness_source_evidence.json", "force_source_evidence.json", "contact_source_evidence.json", "kinematic_source_evidence.json")},
         "targets": transformed_records, "regenerate": "python tools/migrate_destiny.py generate",
         "check": "python tools/migrate_destiny.py generate --check",
         "scope": "Clean staging starts from preserved immutable Rust inputs. Drift comparison never deletes user files."})
    print(json.dumps({"status": "DRIFT_CHECK_PASSED" if check else "GENERATED", "changed": changes}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("inventory", "extract", "generate"))
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.command == "inventory":
        inventory()
    elif args.command == "extract":
        extract()
    else:
        generate(args.check)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(json.dumps({"status": "INCOMPLETE", "reason": str(error)}), file=sys.stderr)
        sys.exit(1)
