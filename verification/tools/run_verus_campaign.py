#!/usr/bin/env python3
"""Bind and execute each reviewed Verus entry once; audit exact symbol evidence.

No discovery, automatic retries or input edits occur in this runner. Failures
preserve their receipts and stop the current campaign. Case acceptance remains
separate and may report incomplete even when all scoped entries are checked.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[2]

def project_path(value):
    path=Path(value)
    return path if path.is_absolute() else ROOT/path

def write_receipt(path,payload):
    if path is None:
        return
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(payload,sort_keys=True)+"\n")

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verus',required=True)
    parser.add_argument('--plan',default='verification/plans/verus-concurrency.json')
    parser.add_argument('--campaign-dir',default='.aegis/destiny/verus-final-campaign')
    parser.add_argument('--receipt')
    args=parser.parse_args()

    plan_path=project_path(args.plan)
    state=project_path(args.campaign_dir)
    receipt_path=project_path(args.receipt) if args.receipt else None

    try:
        plan=json.loads(plan_path.read_text())
        if len(plan['obligations'])!=len({o['entry'] for o in plan['obligations']}):
            raise ValueError('Duplicate entry would repeat a proof')

        prefix=[sys.executable,'-B',str(ROOT/'.agents/bin/destinyctl.py'),'--project',str(ROOT)]

        def run(argv,path):
            result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True)
            path.write_text(result.stdout)
            if result.stderr:
                path.with_suffix('.stderr').write_text(result.stderr)
            if result.returncode:
                raise ValueError('Runner rejected command; see '+str(path))
            return json.loads(result.stdout)

        state.mkdir(parents=True,exist_ok=True)
        bound=run(prefix+['bind',args.plan],state/'binding.json')
        if bound['status']!='BOUND':
            raise ValueError('Source binding failed')

        for obligation in plan['obligations']:
            if obligation['method']!='verus':
                raise ValueError('Campaign contains a different proof method')
            result=run(
                prefix+[
                    'verify',
                    obligation['id'],
                    '--budget-seconds','120',
                    '--verus',args.verus,
                ],
                state/(obligation['id']+'.json'),
            )
            print(obligation['entry'],result['status'],flush=True)
            if result['status']!='CHECKED_SCOPED':
                raise ValueError(
                    'Proof incomplete; retain receipt and diagnose before retry'
                )

        audited=run(prefix+['audit'],state/'audit.json')
        if audited['status']!='CHECKED_SCOPED':
            raise ValueError('Final audit found stale/incomplete evidence')

        # Explicit scoped acceptance of symbol integrity. The full catalog gate is
        # a separate command and retains required correspondence gaps.
        result=subprocess.run(
            [sys.executable,'-B','verification/tools/check_concurrency_acceptance.py'],
            cwd=ROOT,
        )
        summary={
            'schema':1,
            'status':'CHECKED_SCOPED',
            'engine':'verus',
            'kani_proof_credit':False,
            'verus_checked_scoped':True,
            'binding':bound['binding'],
            'deduplicated_entries':len(plan['obligations']),
            'formal_audit':'CHECKED_SCOPED',
            'case_acceptance_exit':result.returncode,
            'campaign_dir':args.campaign_dir,
            'notice':'Scoped counts/symbol evidence do not establish completed cases or release readiness.',
        }
        write_receipt(receipt_path,summary)
        print(json.dumps(summary))
        return result.returncode
    except (ValueError,OSError,KeyError,json.JSONDecodeError) as error:
        incomplete={
            'schema':1,
            'status':'INCOMPLETE',
            'engine':'verus',
            'kani_proof_credit':False,
            'verus_checked_scoped':False,
            'deduplicated_entries':0,
            'formal_audit':'INCOMPLETE',
            'case_acceptance_exit':-1,
            'campaign_dir':args.campaign_dir,
            'reason':str(error),
        }
        write_receipt(receipt_path,incomplete)
        print(json.dumps(incomplete))
        return 1

if __name__=='__main__':
    raise SystemExit(main())
