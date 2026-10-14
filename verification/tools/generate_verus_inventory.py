#!/usr/bin/env python3
"""Review-map actual pinned VIR symbols and create counted formal plans.

Discovery receipts are historical inventory inputs, including count-discovery
rejections. They never become accepted evidence through generation. Plans need
fresh bind/verify/audit and the separate concurrency acceptance validator.
"""
import hashlib
import json
from pathlib import Path
import re
import sys

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'.agents/infra'))
from agentinfra.formal import unseal
from agentinfra.verus_evidence import inventory

ENTRIES=['proofs','state_contracts','lifecycle_contracts','ball_contracts',
         'visibility_contracts','geometry_contracts','ffi_lifecycle','evolution_callbacks']
PIN='ef738406b61ad06bae2f402857e464735ab106a2'
CALLBACK='python/destiny/test/ballpark/evolve/test_add.py::TestAdd::test_balls_can_not_be_added_during_evolve'
CORE=['verification/verus/symbol_inventory.json','verification/verus/trust_register.json',
      'verification/cases/concurrency_mappings.json','verification/cases/inventory.json',
      'verification/cases/reference_review.json','verification/tools/generate_verus_inventory.py',
      'verification/tools/check_concurrency_acceptance.py','verification/tools/generate_concurrency_map.py',
      'verification/spec/src/original_motion.rs','verification/spec/src/original_warp.rs',
      'verification/spec/Cargo.toml','verification/spec/Cargo.lock',
      'verification/controller/base.json','verification/controller/destiny-controller.patch',
      'Cargo.toml','Cargo.lock','rust-toolchain.toml']

def sha(path):return hashlib.sha256((ROOT/path).read_bytes()).hexdigest()

def review(stem,symbol):
    suffix=symbol.rsplit('::',1)[-1]
    if stem=='evolution_callbacks':
        rules={
            'begin_tick':'Only idle phase 0 enters evolution phase 1; every other u8 is rejected without a phase change.',
            'enter_callback':'Only evolution phase 1 enters synchronous callback phase 2.',
            'leave_callback':'Only callback phase 2 returns to evolution phase 1.',
            'finish_tick':'Only evolution phase 1 returns to idle phase 0.',
            'add_allowed':'Only idle phase 0 admits AddBall; evolution and callback phases reject it.',
            'feasible_callback_rejection_trace':'Actual phase kernels compose idle -> evolution -> callback -> evolution -> idle, rejecting AddBall and nested evolution in the callback.'}
        rules['candidate_admitted']='Actual Partition flag kernel preserves the interactive-only branch that bypasses fixed/bubble filters, and otherwise admits the original master/global/bubble and cloaked-owner/free cases.'
        rules['take_notifications']='Actual generic VecDeque extraction preserves the caller prefix and returns exactly the current operation suffix, or rejects an invalid start without mutation. Std container contracts are explicit primitive trust.'
        rules['prepend']='Actual generic VecDeque append/swap consumes the newly produced continuation sequence and places it before the complete owned caller suffix, preserving every element and order for arbitrary valid container states. Container/memory primitive contracts are explicit trust; the foreign callback driver and gate remain separate.'
        rules['feasible_nested_read_preserves_the_owned_parent_batch']='Checked standard-container constructor and actual extraction kernel establish a nonempty owned parent batch and an empty nested-read batch without changing the parent sequence.'
        rules['notification_transition']='Actual notification kernel updates membership only when enabled and requests one notification precisely when its inside/outside state changes; disabled processing preserves prior state.'
        rules['take_next']='Actual generic production continuation kernel returns exactly the current owned queue head and removes that element; an empty queue remains empty. Pinned standard-container contracts remain explicit trust.'
        rules['feasible_owned_notification_continuation']='Checked nonempty original-candidate constructor, actual head extraction and an empty nested read preserve the parent continuation; continuing consumes its second candidate once and reaches an empty pass. World/cancellation/geometry correspondence remains required.'
        if suffix not in rules:raise ValueError('Unreviewed phase symbol '+symbol)
        return ['callback-timing-and-reentry'] if suffix in ('candidate_admitted','take_notifications','take_next','prepend','feasible_owned_notification_continuation','feasible_nested_read_preserves_the_owned_parent_batch','notification_transition') else ['evolution-phase-ownership'],rules[suffix],([CALLBACK] if suffix in ('add_allowed','feasible_callback_rejection_trace') else [])
    if stem=='ffi_lifecycle':
        if '::ball_lifecycle::' in symbol:
            rules={
                'transition':'Actual total retained-object transition preserves reference frames, rejects usize exhaustion/empty release/dead resurrection, sets moribund on Remove, preserves it on Clear, and clears it only for attached repair. Every bool/usize/u8 input is included; registry/object/lease/Mutex/Python correspondence remains required.',
                'feasible_retained_remove_alias':'Checked executable transition sequence constructs two leases, removes membership with moribund true, copies the retained object, releases all three leases and rejects resurrection at detached zero.',
                'feasible_clear_and_delayed_repair':'Checked executable witnesses distinguish Clear from Remove and show delayed removal remains attached while an attached repair clears moribund, with reference counts preserved.',
                'follower_action':'Actual StopAllFollowers action selection rejects absent or retargeted followers, sends missiles to nonmassive Goto and interactive orbiters to Goto, and stops every other admitted mode. Covers all bool/u8 inputs; ordered callback/registry/FFI callers remain required.',
                'admitted':'Actual MissileFollow admission includes every i64/bool input, rejects self/owner/target aliases, absent source/target and moribund targets. A missing owner is tolerated by source; registry, mode callback and finite-field adapters retain separate correspondence obligations.',
                'remote_follower':'Actual source free/follow-mode/remote-bubble selection includes missing targets and does not exclude moribund or mismatched registry entries. All bool/u8 inputs are covered. Free registry, mode/ID/bubble adapters, complete collection and query execution remain required.',
                'action':'Actual harmonic mode action for all bool/u8 values: field true requests Stop/Field, removing existing Field requests Stop, otherwise preserves mode. Actual identity update precedes this decision; complete field/generation/mode/callback/ABI/stream correspondence remains required.',
                'feasible_follower_reentry_and_special_modes':'Checked executable witnesses exercise original Follow, reentrant retarget, absent follower, missile, and interactive/noninteractive Orbit decisions without assumptions.',
            }
            if suffix not in rules: raise ValueError('Unreviewed retained-object symbol '+symbol)
            cases = ['python/destiny/test/ballpark/test_movement_controls.py::'+fixture+'::test_removing_ball_stops_followers_from_following_that_ball'
                     for fixture in ('TestFollow','TestMissileFollow')] if suffix == 'follower_action' else []
            return ['retained-object-membership-and-identity'],rules[suffix],cases
        if suffix=='allocate_collision_id':
            return ['collision-identity-counters'],('Actual production signed-i64 source policy: server negative/client -1 consume only the high counter, server zero consumes only the local counter, explicit identities preserve both counters; MIN exhaustion rejects before decrement. Object-map uniqueness, registration, reset/restore, callers and concurrent ownership remain required.'),[]
        if suffix=='retain_count':relation='Total actual usize kernel rejects zero/MAX and otherwise returns count+1 without overflow.'
        elif suffix=='release_count':relation='Total actual usize kernel rejects zero and otherwise returns count-1; only one reaches zero.'
        elif suffix=='retain_attempt':relation='Actual shared weak-CAS attempt changes the SC atomic permission only on successful increment; rejection and failure preserve it.'
        elif suffix=='release_attempt':relation='Actual shared weak-CAS attempt changes the SC atomic permission only on successful decrement; rejection and failure preserve it.'
        elif suffix=='coupled_retain_attempt':relation='The actual checked increment attempt and token retain transition preserve count-token.value == atomic-permission.value and create a matching new owner only on success.'
        elif suffix=='coupled_release_attempt':relation='The actual checked decrement attempt consumes one owner only on success and withdraws logical allocation storage exactly for the final 1 -> 0 transition; failure returns the same owner.'
        elif suffix=='initialized_live_reference_is_feasible':relation='Checked tracked constructor establishes a feasible logical initialized state with one matching reference and nonempty allocation storage.'
        elif suffix.endswith('_inductive'):relation='The named token-machine initialization/transition preserves the single-allocation invariant: no owners at zero, matching owner multiplicity at a positive machine count, and no unrelated allocation tokens.'
        elif suffix.endswith('_asserts'):relation='The named generated transition/guard assertions follow from the ownership invariant: retain needs a live reference, final release consumes storage, and live-reference guard excludes reclamation.'
        else:raise ValueError('Unreviewed lifetime symbol '+symbol)
        return ['abi-machine-counters','abi-reference-ownership'],relation,[]
    return ['legacy-sequential-model-preservation'],(
        'Preserve the existing named theorem of its local mathematical sequential specification. '
        'Its int/bool abstractions, identity clauses and legacy markers provide no complete qualified original-case, IEEE-754 or production-concurrency credit.'),[]

def main():
    rows=[];plans=[]
    for stem in ENTRIES:
        name=(('verus-retained-ball-lifecycle-development' if (ROOT/'.aegis/destiny/verus-retained-ball-lifecycle-development.json').exists() else 'verus-collision-identity-counter-development' if (ROOT/'.aegis/destiny/verus-collision-identity-counter-development.json').exists() else 'verus-ffi-inventory-development') if stem=='ffi_lifecycle' else
              ('verus-live-continuation-discovery' if (ROOT/'.aegis/destiny/verus-live-continuation-discovery.json').exists() else 'verus-evolution-inventory-development') if stem=='evolution_callbacks' else
              'verus-'+stem+'-symbol-discovery')
        if stem == 'ffi_lifecycle' and (ROOT/'.aegis/destiny/verus-follower-removal-development.json').exists():
            name = 'verus-follower-removal-development'
        if stem == 'ffi_lifecycle' and (ROOT/'.aegis/destiny/verus-missile-admission-development.json').exists():
            name = 'verus-missile-admission-development'
        if stem == 'ffi_lifecycle' and (ROOT/'.aegis/destiny/verus-remote-follower-development.json').exists():
            name = 'verus-remote-follower-development'
        if stem == 'ffi_lifecycle' and (ROOT/'.aegis/destiny/verus-harmonic-development.json').exists():
            name = 'verus-harmonic-development'
        if stem == 'evolution_callbacks' and (ROOT/'.aegis/destiny/verus-continuation-prepend-development.json').exists():
            name = 'verus-continuation-prepend-development'
        summary=json.loads((ROOT/'.aegis/destiny'/f'{name}.json').read_text())
        receipt=unseal(ROOT/summary['receipt']);command=receipt['commands'][0]
        if command['returncode']!=0 or command['timed_out'] or command['stdout_truncated'] or command['stderr_truncated']:
            raise ValueError('Failed/incomplete discovery '+stem)
        counts=re.findall(r'verification results:: (\d+) verified, (\d+) errors',command['stdout'])
        if len(counts)!=1 or int(counts[0][0])==0 or int(counts[0][1])!=0:raise ValueError('Invalid counted discovery '+stem)
        value=receipt['verus_inventory'];actual=inventory((ROOT/value['artifact']).read_text(),value['entry'])
        if len(actual['checked'])!=int(counts[0][0]):raise ValueError('Actual symbol count differs '+stem)
        symbols=[]
        for observed in actual['checked']:
            obligations,relation,cases=review(stem,observed['symbol'])
            symbols.append({'symbol':observed['symbol'],'mode':observed['mode'],
                'obligations':obligations,'original_cases':cases,'relation':relation,
                'review':'Reviewed against the entry contracts and actual production body/transition where declared. Complete original assertions, callers and residual boundaries remain required.',
                'production_body':observed['body'] if stem in ('ffi_lifecycle','evolution_callbacks') else None})
        if stem=='ffi_lifecycle':
            impl=['src/ffi_lifecycle.rs','src/ball_lifecycle.rs','src/missile.rs','src/harmonic.rs','src/collision_identity.rs','src/runtime.rs','src/ffi.rs','src/operation_gate.rs','python/destiny/_backend.py','python/destiny/_facade.py','include/destiny_bevy_compat.h']
            inputs=[actual['entry'],'verification/verus/reference_protocol.rs','verification/verus/ball_lifecycle.rs','src/ffi_lifecycle.rs','src/ball_lifecycle.rs','src/missile.rs','src/harmonic.rs','src/collision_identity.rs',*CORE]
            scope='production_counter_and_atomic_attempt_kernels_plus_logical_token_relation'
            limits='No complete raw-pointer/Box allocation identity, global atomic invariant across the retry driver, reference-transfer caller, operation gate/Mutex, poison, ECS, OS or foreign-callback refinement. No lock-free, fairness or termination claim.'
        elif stem=='evolution_callbacks':
            impl=['src/evolution_phase.rs','src/proximity.rs','src/notification_batch.rs','src/operation_gate.rs','src/ffi.rs','src/runtime.rs','python/destiny/_backend.py','python/destiny/_facade.py']
            inputs=[actual['entry'],'src/evolution_phase.rs','src/proximity.rs','src/notification_batch.rs',*CORE]
            scope='production_phase_kernels_and_feasible_callback_trace'
            limits='Phase kernels alone do not establish callback invocation, ECS membership/repair frame conditions, every command ordering, foreign exception handling, Warp IEEE-754/libm reachability, all worker/engine configurations, snapshots, allocation/Mutex permissions or progress.'
        else:
            impl=[actual['entry']];inputs=[actual['entry'],*CORE]
            scope='legacy_mathematical_sequential_model_only'
            limits='Existing int/bool mathematical and identity theorems are preserved. This entry does not establish production linkage, floating-point equivalence, complete qualified-case assertions, concurrency or interpreter/native/engine correspondence.'
        obligation_id=stem.replace('_','-')+'-reviewed-inventory'
        rows.append({'schema':1,'entry':actual['entry'],'expected_checks':len(symbols),'plan_obligation':obligation_id,
            'scope':scope,'symbols':symbols,'generated_trusted_symbols':[r['symbol'] for r in actual['generated_trusted']],
            'discovery':{'receipt':summary['receipt'],'artifact':value['artifact'],'artifact_sha256':value['artifact_sha256'],
                         'status':'HISTORICAL_DISCOVERY_NOT_ACCEPTANCE'},'residual_obligations':[limits]})
        plans.append({'id':obligation_id,'method':'verus','entry':actual['entry'],'expected_checks':len(symbols),
            'invariant':'Conjoin the exact reviewed theorem and executable contracts in verification/verus/symbol_inventory.json for '+actual['entry']+'. Scope: '+scope+'.',
            'specification':['verification/cases/reference_review.json','verification/cases/concurrency_mappings.json',actual['entry']],
            'implementation':impl,'proof_inputs':sorted(set(inputs)),
            'assumptions':'No new assume/admit/unchecked behavior contracts. Pin Verus 0.2026.09.20.aef82ed, compatible vstd/macros and Z3 as identified by runner receipts; review unavoidable primitives and token-sharding trust in verification/verus/trust_register.json. Machine usize/u8 contracts retain their whole domains; mathematical legacy lemmas retain their original declared preconditions.',
            'limits':limits})
    document={'schema':1,'original_commit':PIN,'verus_pin':'0.2026.09.20.aef82ed',
        'generator_sha256':sha('verification/tools/generate_verus_inventory.py'),'entries':rows,
        'notice':'Actual discovery symbol identities/counts plus semantic scope review. Only current bound regular verifier execution, exact matching inventories and the separate conjunction validator provide accepted scoped evidence. No case or production-readiness boolean is generated.'}
    (ROOT/'verification/verus/symbol_inventory.json').write_text(json.dumps(document,indent=2)+'\n')
    plan={'schema':1,'objective':'Execute each preserved/new Verus entry once against its exact reviewed symbol inventory and production/logical scope; retain all qualified-case and boundary conjuncts separately.',
        'authority_sha256':hashlib.sha256((ROOT/'.agents/contracts/destiny.json').read_bytes()).hexdigest(),
        'reference_review':'Behavioral authority '+PIN+'; separately recorded formation CI pin 114e89fa584f0e1a81eb0a679aabf41af8870072. verification/cases/reference_review.json records reviewed definitions and hashes. Reviewed Ballpark.cpp and Thunkers.cpp differ only in stdafx.h/StdAfx.h include spelling; Ball.cpp, callback case and helper definitions match. No authority switch; external/native numerical and Blue/interpreter correspondence remain pending.',
        'reuse':{'strategy':'generate','sources':['Existing six Verus entries preserved at Destiny HEAD 6e0725313d58ace04bdf13ed096925638c3336d8','Actual production ffi_lifecycle/evolution_phase shared bodies and reviewed pinned Verus token-machine APIs'],
            'license':'Destiny MIT; retained CCP notices in source-derived numeric specifications. Verus tool/framework licenses remain separately identified; no license is invented for the supplied Aegis base.',
            'reason':'Deduplicate exact entry runs and bind actual macro-generated symbol inventories without changing Kani classification or promoting a partial kernel to a completed original case.',
            'generator':['python3','-B','verification/tools/generate_verus_inventory.py']},'obligations':plans}
    path=ROOT/'verification/plans/verus-concurrency.json';path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(plan,indent=2)+'\n')
    print(json.dumps({'entries':len(rows),'checked_symbols':sum(r['expected_checks'] for r in rows),'plan':str(path),'status':'PLANS_AND_REVIEWED_INVENTORY_NO_CURRENT_PROOF_CREDIT'}))

if __name__=='__main__':main()
