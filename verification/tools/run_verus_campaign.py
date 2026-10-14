#!/usr/bin/env python3
"""Validate optional discovery, then execute each reviewed Verus entry once.

The runner does not generate or mutate discovery inputs, retry proofs, or edit
proof sources. When Kani-assisted discovery is supplied it is validated before
the existing source-bound campaign. Failures preserve receipts and stop the
current campaign; case acceptance remains separate.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--verus',required=True)
    parser.add_argument('--plan',default='verification/plans/verus-concurrency.json')
    parser.add_argument('--automation-discovery')
    args=parser.parse_args()
    plan=json.loads((ROOT/args.plan).read_text())
    if len(plan['obligations'])!=len({o['entry'] for o in plan['obligations']}):
        raise ValueError('Duplicate entry would repeat a proof')
    prefix=[sys.executable,'-B',str(ROOT/'.agents/bin/destinyctl.py'),'--project',str(ROOT)]
    def run(argv,path):
        result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True)
        path.write_text(result.stdout)
        if result.stderr:path.with_suffix('.stderr').write_text(result.stderr)
        if result.returncode:raise ValueError('Runner rejected command; see '+str(path))
        return json.loads(result.stdout)
    state=ROOT/'.aegis/destiny/verus-final-campaign';state.mkdir(parents=True,exist_ok=True)
    automation=None
    if args.automation_discovery:
        discovery=Path(args.automation_discovery)
        if not discovery.is_absolute(): discovery=ROOT/discovery
        automation=run([
            sys.executable,'-B',str(ROOT/'verification/tools/verus_kani_bridge.py'),
            'check-discovery','--discovery',str(discovery),'--plan',args.plan
        ],state/'automation-discovery.json')
        if automation['status']!='DISCOVERY_BOUND':
            raise ValueError('Kani-assisted automation discovery is incomplete')
    bound=run(prefix+['bind',args.plan],state/'binding.json')
    if bound['status']!='BOUND':raise ValueError('Source binding failed')
    for obligation in plan['obligations']:
        if obligation['method']!='verus':raise ValueError('Campaign contains a different proof method')
        result=run(prefix+['verify',obligation['id'],'--budget-seconds','120','--verus',args.verus],state/(obligation['id']+'.json'))
        print(obligation['entry'],result['status'],flush=True)
        if result['status']!='CHECKED_SCOPED':raise ValueError('Proof incomplete; retain receipt and diagnose before retry')
    audited=run(prefix+['audit'],state/'audit.json')
    if audited['status']!='CHECKED_SCOPED':raise ValueError('Final audit found stale/incomplete evidence')
    # Explicit scoped acceptance of symbol integrity. The full catalog gate is
    # a separate command and retains required correspondence gaps.
    result=subprocess.run([sys.executable,'-B','verification/tools/check_concurrency_acceptance.py'],cwd=ROOT)
    print(json.dumps({'binding':bound['binding'],'deduplicated_entries':len(plan['obligations']),
        'formal_audit':'CHECKED_SCOPED','case_acceptance_exit':result.returncode,
        'automation_selected_candidates':None if automation is None else automation['selected_candidates'],
        'automation_obligations':[] if automation is None else automation['controller_obligations'],
        'notice':'Scoped counts/symbol evidence do not establish completed cases, translation soundness or release readiness.'}))
    return result.returncode

if __name__=='__main__':
    try:raise SystemExit(main())
    except (ValueError,OSError,KeyError) as error:
        print(json.dumps({'status':'INCOMPLETE','reason':str(error)}));raise SystemExit(1)
