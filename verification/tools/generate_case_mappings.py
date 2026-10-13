#!/usr/bin/env python3
"""Join every qualified original to candidacy and observed assertion properties.

Only explicitly reviewed concrete translations with complete regular
property-discovery output receive an assertion/dependency review here. That
review is not proof credit: fresh classified bindings and all correspondence
obligations are still required. Python fixture/helper mappings remain pending.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = "verification/kani/Cargo.toml"
REVIEWED_PYTHON_RULES={
    "python/destiny/test/ballpark/test_stream_read_write.py::TestWriteBallsToStream::test_write_no_ball": {
        "rule":"Exact original empty [] argument, zero timestamp, packet byte 1 and little-endian int32 timestamp produce the original five literal bytes. The source-backed empty-list specialization executes the common actual header codec and checks the zero-iteration caller condition. All original construction/byte equality/teardown conjunction assertions are retained.",
        "setup":"Original helpers.BallparkTestCase constructs the default client park at currentTime zero and ClearAll on teardown. The unexecuted list body contains no ball operation; byte header semantics are reviewed from Thunkers.cpp:2202..2258. Native Python/Blue constructor, stream dispatch, allocation and ABI correspondence remain separately required.",
    },
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def harness_index():
    found = {}
    for path in sorted((ROOT / "verification/kani/src").glob("*.rs")):
        text = path.read_text()
        markers = list(re.finditer(r'^// original-case: (\S+)\s*$', text, re.M))
        for i, marker in enumerate(markers):
            end = markers[i+1].start() if i+1 < len(markers) else len(text)
            function = re.search(r'^fn (\w+)\s*\(', text[marker.end():end], re.M)
            # paired_regression!(ordinary_test, kani_harness, body) defines
            # distinct names. Select the macro's actual Kani name, confirmed
            # against compiled Kani metadata; the first name is test-only.
            paired = re.search(r'paired_regression!\(\s*(?:#\[[^\n]+\]\s*)*\w+\s*,\s*(\w+)\s*,', text[marker.end():end])
            if paired is not None and (function is None or paired.start()<function.start()):
                function=paired
            if function is None:
                raise ValueError("Qualified marker has no following concrete function: " + marker[1])
            function_start = marker.end() + function.start()
            next_function=re.search(r'^fn \w+\s*\(', text[function_start+len(function[0]):end], re.M)
            if next_function:
                end=function_start+len(function[0])+next_function.start()
            next_paired=re.search(r'paired_regression!\(',text[function_start+len(function[0]):end])
            if next_paired:
                end=function_start+len(function[0])+next_paired.start()
            entry = path.stem + "::" + function[1]
            start_line = text.count("\n", 0, function_start) + 1
            code = text[function_start:end]
            assertions = [start_line + code.count("\n", 0, match.start())
                          for match in re.finditer(r'\bassert(?:_eq|_ne)?!\s*\(', code)]
            found.setdefault(marker[1], []).append({
                "entry": entry, "manifest": MANIFEST,
                "file": path.relative_to(ROOT).as_posix(), "line": start_line,
                "proof_source_sha256": sha(path), "candidate_assertion_lines": assertions,
                "scope": "concrete candidate; setup/native/production correspondence must be reviewed",
            })
    return found


def discovery(candidate):
    sys.path.insert(0, str(ROOT / ".agents/infra"))
    from agentinfra.evidence import kani_properties, property_location, proof_output
    from agentinfra.formal import unseal
    from types import SimpleNamespace
    entry = candidate["entry"]
    path = ROOT / ".aegis/destiny/discovery" / (entry.replace("::", "--") + ".json")
    if not path.exists():
        return None
    result = json.loads(path.read_text())
    if result.get("status") != "OBSERVED_SUCCESS" or not result.get("proof_output_checked"):
        return None
    receipt = unseal(ROOT / result["receipt"])
    if receipt["identity"].get("proof_inputs", {}).get(candidate["file"]) != candidate["proof_source_sha256"]:
        return None
    command, = receipt["commands"]
    obligation = {"method": "kani", "entry": entry, "expected_checks": 1, "manifest": MANIFEST}
    if not proof_output(obligation, SimpleNamespace(**command)):
        return None
    properties = kani_properties(command["stdout"] + "\n" + command["stderr"])
    selected = []
    for name, row in properties.items():
        location = property_location(ROOT, MANIFEST, row)
        if name.startswith(entry + ".assertion.") and row["status"] == "SUCCESS" and location:
            if location["file"] != candidate["file"] or location["line"] not in candidate["candidate_assertion_lines"]:
                raise ValueError("Discovered property no longer names the candidate's real assertion")
            selected.append({**location, "entry": entry, "manifest": MANIFEST, "property": name})
    if not selected:
        return None
    witnesses = [{"property": name, **location} for name, row in properties.items()
                 if name.endswith(".cover.1") and row["status"] == "SATISFIED"
                 and (location := property_location(ROOT, MANIFEST, row)) is not None]
    if not witnesses:
        return None
    return {"assertions": selected, "reachability": witnesses,
            "receipt": result["receipt"], "notice": "historical discovery only; fresh formal verification required"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "verification/cases/mappings.json")
    args = parser.parse_args()
    inventory = json.loads((ROOT / "verification/cases/inventory.json").read_text())
    index = harness_index()
    exact_ids = {case["id"] for case in inventory["cases"]}
    if len(exact_ids) != 531 or set(index) - exact_ids:
        raise ValueError("Omitted, unknown or ambiguous original case identity")
    cases = []
    for original in inventory["cases"]:
        identity = original["id"]
        candidates = index.get(identity, [])
        observed = discovery(candidates[0]) if (original["source"].endswith(".cpp") or identity in REVIEWED_PYTHON_RULES) and len(candidates) == 1 else None
        # The 73 concrete C++ scenarios and common comparison semantics were
        # reviewed directly against the pinned source before translation. The
        # map conservatively requires the entire Rust conjunction for every
        # original site, including vector assertions grouped into Vec3 ==.
        reviewed = observed is not None and original["extraction_status"] != "BLOCKED"
        assertions = []
        for site in original["assertion_ids"]:
            assertions.append({
                "original_id": site,
                "rule": (REVIEWED_PYTHON_RULES[identity]["rule"] if identity in REVIEWED_PYTHON_RULES else "Preserve the complete pinned C++ assertion conjunction and literal setup; exact component/vector equality, booleans and original four-ULP FLOAT/DOUBLE_EQ semantics as applicable. " + site) if reviewed else "PENDING_ASSERTION_AND_HELPER_REVIEW",
                "rust_assertions": observed["assertions"] if reviewed else [],
            })
        dependencies = [{"original_id": site,
                         "relation": (REVIEWED_PYTHON_RULES[identity]["setup"] if identity in REVIEWED_PYTHON_RULES else "Pinned C++ includes/fixture definitions and assertion macro semantics reviewed against engine-independent f64/vector/geometry representation; see reference_review.json.") if reviewed else "PENDING_SETUP_AND_ADAPTER_REVIEW"}
                        for site in original["dependency_ids"]]
        requirements = {"spec_proved": [], "production_rust_refinement_proved": []}
        if reviewed:
            requirements["spec_proved"] = [{"entry": candidates[0]["entry"], "manifest": MANIFEST,
                                           "assertion_properties": [r["property"] for r in observed["assertions"]]}]
        residual = [{"lane": "production_rust_refinement_proved",
                     "relation": "Compile the actual production operation, review both representation adapters and prove all corresponding results/side effects with a production-path cover; wrapper/native/FFI correspondence is still required."}]
        if not reviewed:
            residual.append({"lane": "spec_proved", "relation": "Complete assertion/helper, setup/teardown and original-definition correspondence review and obtain supported, non-vacuous, complete Kani properties for every required operation."})
        if identity in REVIEWED_PYTHON_RULES:
            # This formal validator's residual schema names the two proof
            # lanes. Keep executable native correspondence in the production
            # relation; the independent queue runtime field remains separate.
            residual[0]["relation"]+=" Execute the original native Python/Blue constructor, stream call/Read, byte equality and teardown against a real built library; simulated Rust data alone cannot establish interpreter/ABI/environment correspondence."
        # One pre-existing ADD_DURING_EVOLVE marker only proves a guard
        # predicate. It does not execute the callback or its entire scenario.
        partial = identity.endswith("test_add.py::TestAdd::test_balls_can_not_be_added_during_evolve") and not any(
            c["file"]=="verification/kani/src/add_callback_contracts.rs" for c in candidates)
        cases.append({"id": identity, "original_source_sha256": original["original_source_sha256"],
                      "status": "ImplementationMissing" if not candidates or partial else "ProofMissing",
                      "rust_translation_reviewed": reviewed, "assertions": assertions,
                      "dependency_mappings": dependencies, "candidate_harnesses": candidates,
                      "candidate_scope": "partial guard predicate; original callback scenario missing" if partial else "concrete assertion candidate with pending correspondence" if candidates else "implementation missing",
                      "execution_review": ("Reviewed original empty-list specialization, actual header codec, default timestamp, every asserted literal byte and teardown; real regular Kani property inventory has no unreachable obligations and a satisfied terminal witness. Classified current proof and native correspondence remain pending." if identity in REVIEWED_PYTHON_RULES else "Concrete C++ construction, operation order, complete assertions and no-unexpected-exception semantics reviewed; selected harness includes a terminal satisfied scenario cover.") if reviewed else "PENDING_EXECUTION_AND_TEARDOWN_REVIEW",
                      "reference_review": "verification/cases/reference_review.json",
                      "required_obligations": requirements, "residual_correspondence": residual,
                      "property_discovery": observed})
    document = {"schema": 1, "original_commit": inventory["original_commit"],
                "inventory_sha256": sha(ROOT / "verification/cases/inventory.json"),
                "generator_sha256": sha(Path(__file__)),
                "notice": "Candidacy and reviewed mappings are not current proof credit. Every missing case, helper/dependency and correspondence obligation is retained.",
                "cases": cases}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(document, indent=2) + "\n")
    print(json.dumps({"cases": len(cases), "marked_candidates": len(index),
                      "reviewed_assertion_mappings": sum(c["rust_translation_reviewed"] for c in cases),
                      "bytes": args.output.stat().st_size, "path": str(args.output)}))


if __name__ == "__main__":
    main()
