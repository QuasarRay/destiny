#!/usr/bin/env python3
"""Fail closed on incomplete original-case mappings and fresh formal evidence.

Always regenerate the queue with Aegis's existing receipt/source/tool validators;
never trust a previously saved queue or marker count as evidence. Default mode
requires both proof lanes for all 531 exact identities. --structure-only validates
the denominator and mapping provenance, and explicitly grants no proof credit.
Detailed failed identities are saved under .aegis, with compact console output.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys
from types import SimpleNamespace

ROOT=Path(__file__).resolve().parents[2]

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--controller",type=Path,default=ROOT/".agents")
    parser.add_argument("--structure-only",action="store_true")
    parser.add_argument("--lane",choices=("spec_proved","production_rust_refinement_proved","both"),default="both")
    parser.add_argument("--differential")
    parser.add_argument("--output",type=Path,default=ROOT/".aegis/destiny/case-acceptance.json")
    args=parser.parse_args()
    sys.path.insert(0,str(args.controller.resolve()/"infra"))
    from agentinfra.catalog import qualified_index,work_queue
    from agentinfra.coverage import mappings
    from agentinfra.destiny import load_profile,validate_project
    from agentinfra.security import confined_path
    profile=load_profile();validate_project(ROOT,profile)
    rows=qualified_index(ROOT)
    identities={r["id"] for r in rows}
    if (len(rows),len(identities),len({(r["source"],r["test"]) for r in rows}),len({r["source"] for r in rows}))!=(531,531,509,34):
        raise ValueError("Authoritative original denominator changed")
    inventory=json.loads((ROOT/"verification/cases/inventory.json").read_text())
    document=json.loads((ROOT/"verification/cases/mappings.json").read_text())
    if inventory.get("reader_sha256")!=sha(args.controller/"infra/agentinfra/catalog.py"):
        raise ValueError("Assertion inventory reader changed; regenerate and review it")
    if inventory.get("generator_sha256")!=sha(ROOT/"verification/tools/generate_case_inventory.py"):
        raise ValueError("Assertion inventory generator changed")
    if document.get("inventory_sha256")!=sha(ROOT/"verification/cases/inventory.json"):
        raise ValueError("Assertion mapping inventory digest differs")
    if document.get("generator_sha256")!=sha(ROOT/"verification/tools/generate_case_mappings.py"):
        raise ValueError("Assertion mapping generator changed")
    records=mappings(ROOT,rows)
    if set(records)!=identities:raise ValueError("Omitted or ambiguous assertion mapping identity")
    for record in records.values():
        for candidate in record.get("candidate_harnesses",[]):
            if candidate.get("proof_source_sha256")!=sha(confined_path(ROOT,candidate["file"],must_exist=True)):
                raise ValueError("Stale candidate source: "+record["id"])
    queue_path=ROOT/".aegis/destiny/acceptance-work-queue.json"
    summary=work_queue(ROOT,profile,SimpleNamespace(original=None,output=str(queue_path.relative_to(ROOT)),differential=args.differential))
    queue=json.loads(queue_path.read_text())
    if len(queue["obligations"])!=531 or {r["id"] for r in queue["obligations"]}!=identities:
        raise ValueError("Fresh queue omitted or merged an original")
    lanes=("spec_proved","production_rust_refinement_proved") if args.lane=="both" else (args.lane,)
    failures=[]
    for row in queue["obligations"]:
        record=records[row["id"]]
        reasons=[]
        if record.get("rust_translation_reviewed") is not True:reasons.append("Original assertion/setup/helper correspondence not reviewed")
        for lane in lanes:
            if not row["assurance"][lane]:reasons.append("Missing current complete "+lane)
        if reasons:failures.append({"id":row["id"],"status":record["status"],"reasons":reasons,
                                  "residual_correspondence":record["residual_correspondence"]})
    report={"schema":1,"status":"STRUCTURE_VALIDATED_NO_PROOF_CREDIT" if args.structure_only else "INCOMPLETE" if failures else "ACCEPTED_SCOPED",
            "original_commit":queue["original_commit"],"source_identity":queue["source_identity"],
            "qualified_cases":531,"assurance_counts":summary["assurance_counts"],"requested_lanes":list(lanes),
            "failed_cases":failures,"queue_blockers":queue["blockers"],
            "notice":"Only current classified evidence through the existing Aegis validators can discharge a lane. Structure mode and markers confer no proof credit; release and non-Rust correspondence gates remain separate."}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+"\n")
    print(json.dumps({"status":report["status"],"qualified_cases":531,"failed_cases":len(failures),
                      "assurance_counts":summary["assurance_counts"],"report":str(args.output)}))
    return 0 if args.structure_only or not failures else 1

if __name__=="__main__":
    try:sys.exit(main())
    except (OSError,ValueError,KeyError,TypeError,RuntimeError) as error:
        print(json.dumps({"status":"INVALID_OR_UNAVAILABLE","reason":str(error)}));sys.exit(1)
