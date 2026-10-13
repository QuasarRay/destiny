#!/usr/bin/env python3
"""Prepare bounded plans from real property discovery, without claiming proof.

Each entry runs one exact harness once. Plans require fresh bind/verify/audit
after every source/report/controller input is ready. Missing property discovery
is retained in mappings.json rather than filled with invented property IDs.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main():
    mappings = json.loads((ROOT / "verification/cases/mappings.json").read_text())
    selected = [c for c in mappings["cases"] if c["rust_translation_reviewed"]
                and c["property_discovery"] is not None]
    proof_inputs = sorted({
        p.relative_to(ROOT).as_posix()
        for directory, pattern in (("verification/kani/src", "*.rs"),
                                   ("verification/meta/src", "*.rs"),
                                   ("verification/meta-support/src", "*.rs"))
        for p in (ROOT / directory).glob(pattern)
    } | {"verification/kani/Cargo.toml", "verification/kani/Cargo.lock",
         "verification/meta/Cargo.toml", "verification/meta/Cargo.lock",
         "verification/meta-support/Cargo.toml", "verification/meta-support/Cargo.lock",
         "verification/cases/inventory.json", "verification/cases/mappings.json",
         "verification/cases/reference_review.json", "src/formation.rs",
         "rust-toolchain.toml", "verification/tools/generate_formal_plans.py",
         "verification/tools/generate_case_mappings.py"})
    specification = sorted({p.relative_to(ROOT).as_posix()
                            for p in (ROOT / "verification/spec/src").glob("*.rs")}
                           | {"verification/spec/Cargo.toml", "verification/spec/Cargo.lock"})
    if len(proof_inputs) > 64 or len(specification) > 64:
        raise ValueError("Consumed compiler inputs exceed formal plan schema bound")
    obligations = []
    for case in selected:
        requirement, = case["required_obligations"]["spec_proved"]
        candidate, = case["candidate_harnesses"]
        obligations.append({
            "id": "cpp-" + hashlib.sha256(case["id"].encode()).hexdigest()[:16],
            "invariant": "The concrete pinned scenario " + case["id"] +
                         " satisfies the entire recorded original assertion conjunction after its exact construction and operation sequence.",
            "method": "kani", "entry": requirement["entry"], "manifest": requirement["manifest"],
            "expected_checks": 1, "specification": specification,
            "implementation": ["verification/spec/src/model.rs", "verification/spec/src/geometry.rs",
                               "verification/spec/src/collision.rs"],
            "proof_inputs": proof_inputs,
            "assumptions": "Concrete original literals, reviewed C++ fixture initialization and macro semantics; f64 IEEE arithmetic and the pinned Kani builtin model for supported operations. No kani::assume in these concrete C++ harnesses. Preserve all source-declared unwind limits; no safety or reachability checks disabled.",
            "limits": "Specification scenario only. It does not establish arbitrary input-domain parity, native C++ compiler/GTest execution, platform ABI, Python loading, engine wrapper behavior or production refinement. Every production correspondence requirement remains in mappings.json.",
            "evidence": {
                "kind": "spec_proved", "original_cases": [case["id"]],
                "assertion_properties": requirement["assertion_properties"],
                "reachability": case["property_discovery"]["reachability"],
                "relation": "All original assertion sites are mapped to the complete Rust assertion conjunction in " + candidate["file"] + "; vector equality expands to all three exact coordinates and floating comparisons preserve the original four-ULP rule where used.",
                "review": "Original fixture/test definitions and consumed geometry/collision definitions were read at the behavioral pin. Their per-file hashes and revision differences are in reference_review.json. Assertion/dependency locations, exact manifest/harness and observed regular-output properties are in mappings.json. Discovery is historical; fresh classified verification and source-location matching are required for credit.",
            },
        })
    directory = ROOT / "verification/plans"
    directory.mkdir(parents=True, exist_ok=True)
    files = []
    for start in range(0, len(obligations), 32):
        path = directory / f"cpp-concrete-{start//32+1:03}.json"
        plan = {
            "schema": 1, "objective": "Verify the complete reviewed C++ concrete assertion scenarios represented by this bounded plan.",
            "authority_sha256": hashlib.sha256((ROOT / ".agents/contracts/destiny.json").read_bytes()).hexdigest(),
            "reference_review": "Behavioral authority ef738406b61ad06bae2f402857e464735ab106a2; formation CI authority 114e89fa584f0e1a81eb0a679aabf41af8870072. These test/geometry/collision definition hashes match across pins; exact reviewed paths and any other differences are retained in verification/cases/reference_review.json.",
            "reuse": {"strategy": "reuse", "sources": [
                "Original C++ test and geometry/collision definitions at ef738406b61ad06bae2f402857e464735ab106a2",
                "Reviewed existing engine-independent Rust specification and concrete Kani counterparts",
                "Real complete Kani 0.68.0 property discovery recorded in mappings.json; no status imported as fresh proof"],
                "license": "MIT: original Destiny notices retained in source; existing verification and controller notices preserved.",
                "reason": "Reuse the reviewed concrete counterparts and existing strict controller; no new prover or unproved replacement operation.",
                "generator": ["python3", "verification/tools/generate_formal_plans.py"]},
            "obligations": obligations[start:start+32],
        }
        path.write_text(json.dumps(plan, indent=2) + "\n")
        files.append(str(path.relative_to(ROOT)))
    print(json.dumps({"plans": files, "declared_harnesses": len(obligations), "status": "PREPARED_NOT_VERIFIED"}))


if __name__ == "__main__":
    main()
