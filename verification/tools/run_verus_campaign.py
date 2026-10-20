#!/usr/bin/env python3
"""Bind and execute each reviewed Verus entry once; audit exact symbol evidence.

No discovery, automatic retries or input edits occur in this runner. Failures
preserve their receipts and stop the current campaign. Case acceptance remains
separate and may report incomplete even when all scoped entries are checked.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys

try:
    import resource
except ImportError:
    resource = None

ROOT=Path(__file__).resolve().parents[2]


class CampaignTimeout(RuntimeError):
    pass


def select_plan(plan, requested):
    obligations=plan.get('obligations')
    if not isinstance(obligations,list) or not obligations:
        raise ValueError('Verus campaign plan has no obligations')
    by_id={}
    for obligation in obligations:
        identity=obligation.get('id')
        if not isinstance(identity,str) or not identity:
            raise ValueError('Verus campaign plan contains an obligation without an id')
        if identity in by_id:
            raise ValueError('Duplicate obligation id in Verus campaign plan: '+identity)
        by_id[identity]=obligation

    if requested:
        if len(requested)!=len(set(requested)):
            raise ValueError('Configured Verus obligation inventory contains duplicates')
        unknown=sorted(set(requested)-set(by_id))
        if unknown:
            raise ValueError('Configured Verus obligations are absent from the plan: '+', '.join(unknown))
        selected=[by_id[identity] for identity in requested]
    else:
        selected=list(obligations)
    if not selected:
        raise ValueError('Empty Verus obligation inventory cannot be executed')

    filtered=dict(plan)
    filtered['obligations']=selected
    return filtered


def subprocess_with_limits(argv,cwd,timeout_seconds,memory_mib=None):
    if timeout_seconds<=0:
        raise ValueError('timeout-seconds must be positive')
    if memory_mib is not None and memory_mib<=0:
        raise ValueError('memory-mib must be positive')
    if memory_mib is not None and resource is None:
        raise ValueError('memory-mib is unsupported on this platform')

    preexec_fn=None
    if memory_mib is not None:
        limit=memory_mib*1024*1024
        def apply_memory_limit():
            resource.setrlimit(resource.RLIMIT_AS,(limit,limit))
        preexec_fn=apply_memory_limit

    process=subprocess.Popen(
        argv,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        start_new_session=(os.name=='posix'),
        preexec_fn=preexec_fn,
    )
    try:
        stdout,stderr=process.communicate(timeout=timeout_seconds)
    except subprocess.TimeoutExpired as error:
        if os.name=='posix':
            os.killpg(process.pid,signal.SIGKILL)
        else:
            process.kill()
        stdout,stderr=process.communicate()
        raise CampaignTimeout(
            f'command exceeded configured {timeout_seconds}s limit: {argv!r}; '
            f'stdout={stdout[-1000:]!r} stderr={stderr[-1000:]!r}'
        ) from error
    return subprocess.CompletedProcess(argv,process.returncode,stdout,stderr)


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
    parser.add_argument('--obligation',action='append',default=[])
    parser.add_argument('--timeout-seconds',type=int,default=120)
    parser.add_argument('--memory-mib',type=int)
    parser.add_argument('--max-concurrency',type=int,default=1)
    args=parser.parse_args()

    if not 1<=args.timeout_seconds<=120:
        parser.error('--timeout-seconds must be between 1 and the maintained 120-second scoped Verus limit')
    if args.memory_mib is not None and args.memory_mib<=0:
        parser.error('--memory-mib must be positive')
    if args.max_concurrency!=1:
        parser.error('--max-concurrency must be 1 because the maintained Destiny controller serializes proof execution')

    plan_path=project_path(args.plan)
    state=project_path(args.campaign_dir)
    receipt_path=project_path(args.receipt) if args.receipt else None

    try:
        plan=json.loads(plan_path.read_text())
        if len(plan['obligations'])!=len({o['entry'] for o in plan['obligations']}):
            raise ValueError('Duplicate entry would repeat a proof')
        selected_plan=select_plan(plan,args.obligation)
        selected_obligations=[row['id'] for row in selected_plan['obligations']]

        prefix=[sys.executable,'-B',str(ROOT/'.agents/bin/destinyctl.py'),'--project',str(ROOT)]

        def run(argv,path):
            result=subprocess_with_limits(
                argv,
                ROOT,
                args.timeout_seconds,
                args.memory_mib,
            )
            path.write_text(result.stdout)
            if result.stderr:
                path.with_suffix('.stderr').write_text(result.stderr)
            if result.returncode:
                raise ValueError('Runner rejected command; see '+str(path))
            return json.loads(result.stdout)

        state.mkdir(parents=True,exist_ok=True)
        try:
            state.resolve().relative_to(ROOT.resolve())
        except ValueError as error:
            raise ValueError('campaign directory must remain inside the project root') from error
        selected_plan_bytes=(json.dumps(selected_plan,indent=2,sort_keys=True)+'\n').encode()
        selected_plan_digest=hashlib.sha256(selected_plan_bytes).hexdigest()
        selected_plan_path=ROOT/'target'/'kverus-plans'/f'{selected_plan_digest}.json'
        selected_plan_path.parent.mkdir(parents=True,exist_ok=True)
        if selected_plan_path.exists() and selected_plan_path.read_bytes()!=selected_plan_bytes:
            raise ValueError('deterministic selected-plan digest collision')
        selected_plan_path.write_bytes(selected_plan_bytes)
        selected_plan_arg=str(selected_plan_path.relative_to(ROOT))
        bound=run(prefix+['bind',selected_plan_arg],state/'binding.json')
        if bound['status']!='BOUND':
            raise ValueError('Source binding failed')

        for obligation in selected_plan['obligations']:
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
        acceptance_command=[
            sys.executable,
            '-B',
            'verification/tools/check_concurrency_acceptance.py',
        ]
        full_campaign=len(selected_plan['obligations'])==len(plan['obligations'])
        if not full_campaign:
            acceptance_command.append('--structure-only')
        result=subprocess.run(acceptance_command,cwd=ROOT)
        summary={
            'schema':1,
            'status':'CHECKED_SCOPED',
            'engine':'verus',
            'kani_proof_credit':False,
            'verus_checked_scoped':True,
            'binding':bound['binding'],
            'deduplicated_entries':len(selected_plan['obligations']),
            'formal_audit':'CHECKED_SCOPED',
            'case_acceptance_exit':result.returncode,
            'case_acceptance_scope':'full_catalog' if full_campaign else 'structure_only_for_selected_subset',
            'campaign_dir':args.campaign_dir,
            'obligations':selected_obligations,
            'timeout_seconds':args.timeout_seconds,
            'memory_mib':args.memory_mib,
            'max_concurrency':args.max_concurrency,
            'notice':'Scoped counts/symbol evidence do not establish completed cases or release readiness.',
        }
        write_receipt(receipt_path,summary)
        print(json.dumps(summary))
        return result.returncode
    except CampaignTimeout as error:
        incomplete={
            'schema':1,
            'status':'TIMEOUT',
            'engine':'verus',
            'kani_proof_credit':False,
            'verus_checked_scoped':False,
            'deduplicated_entries':0,
            'formal_audit':'INCOMPLETE',
            'case_acceptance_exit':-1,
            'campaign_dir':args.campaign_dir,
            'obligations':list(args.obligation),
            'timeout_seconds':args.timeout_seconds,
            'memory_mib':args.memory_mib,
            'max_concurrency':args.max_concurrency,
            'reason':str(error),
        }
        write_receipt(receipt_path,incomplete)
        print(json.dumps(incomplete))
        return 1
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
            'obligations':list(args.obligation),
            'timeout_seconds':args.timeout_seconds,
            'memory_mib':args.memory_mib,
            'max_concurrency':args.max_concurrency,
            'reason':str(error),
        }
        write_receipt(receipt_path,incomplete)
        print(json.dumps(incomplete))
        return 1

if __name__=='__main__':
    raise SystemExit(main())
