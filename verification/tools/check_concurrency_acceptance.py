#!/usr/bin/env python3
"""Join every qualified case to reviewed symbols and current counted observations.

This separate Verus lane preserves Kani classification. A checked kernel or
logical model cannot discharge a group's unresolved wrapper/boundary conjuncts.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'.agents/infra'))
from agentinfra.catalog import qualified_index
from agentinfra.destiny import load_profile, validate_project
from agentinfra.evidence import current_receipt
from agentinfra.formal import audit, current_identity, read, unseal
from agentinfra.verus_evidence import current_inventory, validate_case_map, validate_symbol_mapping

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--structure-only',action='store_true')
    parser.add_argument('--binding',help='Exact current formal binding; defaults to the active one')
    parser.add_argument('--output',type=Path,default=ROOT/'.aegis/destiny/concurrency-acceptance.json')
    args=parser.parse_args()
    profile=load_profile();validate_project(ROOT,profile)
    catalog=qualified_index(ROOT)
    inventory=read(ROOT/'verification/cases/inventory.json')
    value=read(ROOT/'verification/cases/concurrency_mappings.json')
    symbols=read(ROOT/'verification/verus/symbol_inventory.json')
    if value['inventory_sha256']!=sha(ROOT/'verification/cases/inventory.json'):
        raise ValueError('Stale assertion/dependency inventory')
    if value['generator_sha256']!=sha(ROOT/'verification/tools/generate_concurrency_map.py'):
        raise ValueError('Concurrency map generator changed')
    if symbols['generator_sha256']!=sha(ROOT/'verification/tools/generate_verus_inventory.py'):
        raise ValueError('Symbol mapping generator changed')
    records=validate_case_map(catalog,inventory,value)
    for row in records.values():
        for path,expected in row['production_hashes'].items():
            if sha(ROOT/path)!=expected:raise ValueError('Stale production map: '+row['id']+' -> '+path)
    entries=symbols['entries']
    registered={command[-1] for command in profile['checks']['verus']['commands']}
    if len(entries)!=len({entry['entry'] for entry in entries}) or {entry['entry'] for entry in entries}!=registered:
        raise ValueError('Omitted/duplicate/unregistered Verus entry')
    checked=[];errors=[]
    if not args.structure_only:
        audited=audit(ROOT,profile,args.binding)
        key=audited['binding']
        binding=unseal(ROOT/f'.aegis/destiny/bindings/{key}.json')
        obligations={o['id']:o for o in binding['contract']['obligations']}
        results={r['id']:r for r in audited['obligations']}
        for declared in entries:
            oid=declared['plan_obligation']
            try:
                obligation=obligations[oid];observed=results[oid]
                if observed['status']!='CHECKED_SCOPED':raise ValueError('Missing/invalid current observation')
                proof=unseal(ROOT/f'.aegis/destiny/proofs/{key}/{oid}.json')
                receipt=current_receipt(ROOT,profile,proof['result'])
                actual=current_inventory(ROOT,receipt,obligation)
                names=validate_symbol_mapping(actual,declared)
                checked.append({'entry':declared['entry'],'scope':declared['scope'],
                    'symbols':len(names),'receipt':proof['result']['receipt'],
                    'residual_obligations':declared['residual_obligations']})
            except (ValueError,OSError,KeyError) as error:
                errors.append({'entry':declared['entry'],'reason':str(error)})
    # Required groups explicitly retain their missing conjunction members. A
    # successful entry is partial until every declared caller/original/numeric
    # and execution boundary is discharged by its corresponding evidence lane.
    gaps=[]
    for row in records.values():
        required=row['required_obligations']
        residual=[{'id':group,'remaining':value['obligations'][group]['residual_obligations']}
                  for group in required if value['obligations'][group]['residual_obligations']]
        # Every currently declared entry is explicitly a kernel or a local
        # mathematical model. Even clearing prose residuals cannot turn these
        # scopes into complete public operations. A complete scope needs a
        # separately reviewed, executed wrapper/caller/observation relation.
        incomplete_scopes=[group for group in required if not any(
            entry['entry']==value['obligations'][group]['entry']
            and entry['scope']=='complete_production_concurrency_refinement'
            and not entry['residual_obligations'] for entry in checked)]
        if required and (residual or errors or incomplete_scopes or row['review_status'].endswith('PENDING')):
            gaps.append({'id':row['id'],'relevance':row['concurrency_relevance'],
                'required_obligations':required,'incomplete_production_scopes':incomplete_scopes,
                'residual_conjuncts':residual,'remaining_gaps':row['remaining_gaps']})
    status='STRUCTURE_VALIDATED_NO_PROOF_CREDIT' if args.structure_only else 'INCOMPLETE' if gaps or errors else 'ACCEPTED_CONCURRENCY_SCOPE'
    report={'schema':1,'status':status,'original_commit':value['original_commit'],
        'qualified_cases':531,'relevance_counts':dict(Counter(r['concurrency_relevance'] for r in records.values())),
        'required_derived_groups':len(value['obligations']),'current_checked_entries':checked,
        'entry_errors':errors,'incomplete_required_cases':gaps,
        'completed_required_cases':sum(bool(r['required_obligations']) for r in records.values())-len(gaps) if not args.structure_only else 0,
        'notice':'Exact symbol/input/execution integrity and declared conjunction accounting only. Semantic adequacy needs review. Kani assurance lanes, runtime observations, non-Rust boundaries and release readiness remain independent.'}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:report[k] for k in ('status','qualified_cases','relevance_counts','required_derived_groups','completed_required_cases')},sort_keys=True))
    return 0 if args.structure_only or status=='ACCEPTED_CONCURRENCY_SCOPE' else 1

if __name__=='__main__':
    try:raise SystemExit(main())
    except (ValueError,OSError,KeyError,TypeError) as error:
        print(json.dumps({'status':'INVALID_OR_UNAVAILABLE','reason':str(error)}));raise SystemExit(1)
