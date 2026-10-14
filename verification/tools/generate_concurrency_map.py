#!/usr/bin/env python3
"""Preserve the complete qualified catalog in a separate concurrency review map.

Original expressions/dependencies are referenced exactly; no Kani assurance
field is changed. Applicability is separate from proved obligations. Initial
production/correspondence reviews remain explicit until their complete paths
and observed Verus inventories have been reviewed and bound.
"""
import ast
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2]
PIN='ef738406b61ad06bae2f402857e464735ab106a2'
CALLBACK_CASE='python/destiny/test/ballpark/evolve/test_add.py::TestAdd::test_balls_can_not_be_added_during_evolve'

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def original(path):
    return subprocess.run(['git','--no-replace-objects','show',PIN+':'+path],cwd=ROOT,capture_output=True,check=True).stdout

def main():
    inventory=json.loads((ROOT/'verification/cases/inventory.json').read_text())
    if inventory['original_commit']!=PIN:raise ValueError('Original pin changed')
    cases=inventory['cases']
    if (len(cases),len({c['id'] for c in cases}),len({(c['source'],c['test']) for c in cases}),len({c['source'] for c in cases}))!=(531,531,509,34):
        raise ValueError('Qualified denominator changed')
    blobs={source:original(source) for source in {c['source'] for c in cases}}
    trees={p:ast.parse(b.decode()) for p,b in blobs.items() if p.endswith('.py')}
    records=[]
    for c in cases:
        source=c['source'];data=blobs[source]
        if hashlib.sha256(data).hexdigest()!=c['original_source_sha256']:raise ValueError('Case original hash differs')
        calls=[];body_hash=None
        if source.endswith('.py'):
            fixture=next(n for n in trees[source].body if isinstance(n,ast.ClassDef) and n.name==c['fixture'])
            method=next(n for n in fixture.body if isinstance(n,ast.FunctionDef) and n.name==c['test'])
            calls=sorted({ast.unparse(n.func) for n in ast.walk(method) if isinstance(n,ast.Call)})
            body_hash=hashlib.sha256('\n'.join(data.decode().splitlines()[method.lineno-1:method.end_lineno]).encode()).hexdigest()
        if source=='python/destiny/test/test_using_correct_module.py':
            relevance='justified_non_applicable';requirements=[]
            rationale='Exact body checks module file location or sys.path membership. It performs no concurrent state transition or threading operation. Original interpreter/module-path execution remains required and is not a Verus theorem.'
            production=['python/destiny/__init__.py'];status='Specified'
        elif source.endswith('.cpp'):
            relevance='supporting_sequential_contract';requirements=[]
            rationale='Original C++ case exercises geometry/collision value operations and GTest assertions. Parallel vector terminology is numerical, not threading. Numeric/IEEE and adapter parity remain Kani/native obligations; no threading theorem is invented from the title.'
            production=['python/destiny/_geometry.py','src/runtime.rs'];status='ProofMissing'
        elif source=='python/destiny/test/test_configure.py':
            relevance='required_concurrency_obligation';requirements=['configuration-snapshot-coherence']
            rationale='Source Settings Apply/Get/GetDefault/Reset copies all five size_t/bool fields. Actual Python RLock publishes a private validated snapshot/revision; native parks observe it before calls through one protected Configure operation, including orientation flags on existing parks. Configuration histories, caller snapshots and numerical/engine/ABI correspondence remain required; six specification scenarios and Python executions do not discharge them.'
            production=['python/destiny/_settings.py','python/destiny/_backend.py','src/runtime.rs'];status='ProofMissing'
        else:
            relevance='required_concurrency_obligation'
            requirements=['abi-reference-ownership','abi-machine-counters','abi-serialized-dispatch','abi-poison-and-unwind','abi-buffer-ownership']
            requirements+=['collision-identity-counters']
            production=['src/ffi.rs','src/ffi_lifecycle.rs','src/operation_gate.rs','src/runtime.rs','python/destiny/_backend.py','python/destiny/_facade.py']
            rationale='The supported C ABI permits independently retained owners/threads and serializes runtime calls. Preserve this exact original sequential scenario at admitted call/phase boundaries; ownership, errors and ordered effects need corresponding concurrency obligations.'
            status='ProofMissing'
            if '/evolve/' in source or source.endswith('test_callbacks.py'):
                requirements+=['evolution-phase-ownership','callback-timing-and-reentry','ordered-numerical-effects']
                production+=['src/evolution_phase.rs','src/proximity.rs','src/notification_batch.rs']
            if 'test_stream_read_write.py' in source or '/net/client/' in source:
                requirements+=['snapshot-publish-rollback-and-identity']
            if (source.endswith(('test_lifecycle_management.py','test_live_counts.py','test_stream_read_write.py','test_callbacks.py','evolve/test_minis.py'))
                    or source=='python/destiny/test/net/client/test_ticker.py' or source.endswith('test_movement_controls.py')):
                requirements+=['retained-object-membership-and-identity']
                production+=['src/ball_lifecycle.rs']
            if '/net/' in source:
                requirements+=['ordered-network-history-and-delivery-boundary']
                production += ['src/network.rs']
                side='client' if '/net/client/' in source else 'server'
                production += sorted(p.relative_to(ROOT).as_posix() for p in (ROOT/'python/destiny/net'/side).glob('*.py'))
            if source.endswith('evolve/test_add.py'):
                status='ProofMissing'
                production+=['verification/spec/src/original_warp.rs','verification/spec/src/original_motion.rs']
                rationale+=' The native Warp-to-Stop callback now executes before mode storage inside the evolution phase. The production guard rejects both creation and repair before membership/state mutation. Complete numeric, gate, pointer, world/phase and foreign error correspondence is still required; phase kernels alone do not discharge this case.'
        records.append({
            'id':c['id'],'original_source_sha256':c['original_source_sha256'],'case_body_sha256':body_hash,
            'assertion_ids':c['assertion_ids'],'dependency_ids':c['dependency_ids'],
            'original_calls':calls,'concurrency_relevance':relevance,'applicability_rationale':rationale,
            'required_obligations':requirements,'status':status,'production_paths':production,
            'production_hashes':{p:sha(ROOT/p) for p in production},
            'reference_review':'verification/cases/reference_review.json',
            'assertion_observation_relation':'Exact expressions and helper/setup IDs remain in inventory.json. All required original postconditions, ordering, errors, tolerances and unaffected-state relations must be conjoined with the linked concurrency obligations; this applicability map does not certify their translation.',
            'execution_obligation':{'required':True,'relation':'Execute the exact original setup and ordered case body through its supported production boundary without unexpected failure; preserve source-declared expected errors and callback error handling.','status':'CORRESPONDENCE_PENDING'},
            'review_status':'APPLICABILITY_REVIEWED_PRODUCTION_AND_CORRESPONDENCE_PENDING',
            'remaining_gaps':['Complete production-path/adapters and caller/trusted-boundary correspondence review; obtain current symbol-bound Verus and applicable concrete/native evidence.'] if requirements else ['Supporting numerical or interpreter boundary validation remains separate; no concurrency proof credit.'],
            'assertion_mappings':[{'id':a,'original_expression':inventory['assertions'][a]['expression'],
                'source_sha256':inventory['assertions'][a]['source_sha256'],
                'behavioral_rule':'Preserve this exact original assertion, setup and helper dependencies in the admitted history.',
                'concurrency_obligations':requirements,'verus_symbols':[],
                'review':'ORIGINAL_EXPRESSION_RETAINED_FULL_RELATION_PENDING'} for a in c['assertion_ids']],
            'dependency_mappings':[{'id':d,'snippet_sha256':inventory['dependencies'][d]['snippet_sha256'],
                'concurrency_obligations':requirements,'review':'SETUP_SITE_RETAINED_CALLER_CORRESPONDENCE_PENDING'} for d in c['dependency_ids']],
        })
        if c['source']=='python/destiny/test/ballpark/test_lifecycle_management.py' and c['fixture']=='TestRemoveBall':
            record=records[-1]
            record['production_call_path']=['Original helper constructs the default client park and fixed ball ID1',
                'Facade Ball construction -> backend binding session -> actual generation and RetainBall lease',
                'RemoveBall -> native schedule/remove -> shared retained lifecycle transition; membership and object retention are distinct',
                'Retained generation routes isMoribund to the original object rather than a replacement with the same public ID; wrapper finalization releases that lease']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ballpark.cpp',[[5471,5591],[5896,5961]]),('python/destiny/test/helpers.py',[[41,50],[159,201]])]]
            for assertion in record['assertion_mappings']:
                assertion['behavioral_rule']='Exact retained Ball.isMoribund assertion after immediate or delay100 RemoveBall; source sets the flag before either removal branch.'
                assertion['verus_symbols']=['ffi_lifecycle::ball_lifecycle::transition','ffi_lifecycle::ball_lifecycle::feasible_retained_remove_alias','ffi_lifecycle::ball_lifecycle::feasible_clear_and_delayed_repair']
                assertion['review']='SOURCE_AND_UNCHANGED_NATIVE_ASSERTION_REVIEWED_KERNEL_ONLY_FULL_CALLER_CONJUNCTION_PENDING'
            record['runtime_scope']='Both unchanged original RemoveBall cases pass on the real native library, zero skips: .aegis/destiny/lifecycle-two-remove-after.json. Development observation, not classified proof or final artifact attestation.'
            record['remaining_gaps']=['Shared bool/usize/u8 kernel accepted by pinned Verus20entry symbols and Kani69SUCCESS11SAT; complete registry/generation/map/lease and snapshot adapter proof remains required.',
                'Original Stop/StopAllFollowers/final-cleanup callback and full detached-object mutation/rotation/descriptor lifetime correspondence remain incomplete.',
                'Python weakref/destructor/locks, live native component/default representation, generation allocation/exhaustion, failed/cancelled restore and all permitted reentrant caller histories need current full correspondence.',
                'Driver lock is released before waiting for the backend and close releases backend outside the driver lock; a barrier-controlled native regression passes. Whole scheduler exclusivity/progress is not inferred from this regression.']
        if c['source']=='python/destiny/test/ballpark/test_lifecycle_management.py' and c['fixture']=='TestAddMushroom':
            record=records[-1]
            record['production_paths']=sorted(set(record['production_paths']+['src/mushroom.rs']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['production_call_path']=['AddMushroom validates source signed owner/f32 range/f64 duration -> actual source initial_ball parameters -> native AddBall and source role ID allocation',
                'Original owner/position, duration/gotoX, followRange, effect stamp and mode6 are stored in actual components/controls',
                'A free nonmoribund mushroom uses source pre-publication current time and f32 fraction/powf/radius; native radius publication precedes notification/time publication',
                'At fraction>=1 source-style delayed removal marks moribund and the retained object preserves the previous radius; fixed mushrooms do not expand']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ballpark.cpp',[[562,578],[3716,3788],[5471,5591]]),('src/Thunkers.cpp',[[304,324]]),('python/destiny/test/helpers.py',[[41,50],[159,201]])]]
            record['runtime_scope']='All5unchanged original TestAddMushroom cases pass native, zero skips: .aegis/destiny/mushroom-five-original-native-published.json. Full7radius assertions retained; development observation only.'
            record['remaining_gaps']=['Source default/time/owner/creation, every original assertion and exact helper setup retained in existing park_contracts specification candidates; production caller/map/Mutex/Python/engine/callback/cleanup/restore relations remain required.',
                'Complete actual radius kernel harness passes ordinarily but Kani failed4literal radius comparisons plus1unreachable: receipt5abe208b2fe74b1986cad29c010132f0, 213SUCCESS3SAT. Actual installed CBMC powf uses a nondeterministic exponent-bit approximation with exp_c[-722019,1]; it does not discharge original precision. No math stub, tolerance or domain change is accepted.',
                'Integer timestamp overflow/Win long widths, fractional/native tick interval representation, nonfinite API domains, source dynamical orientation, every collision/owner/mini combination and full engine publication/rollback remain required.']
            if c['test']=='test_mushroom_expands_correctly':
                record['production_kernel_candidates']=[{'harness':'mushroom_production_contracts::original_mushroom_all_seven_radius_assertions',
                    'proof_source':'verification/kani/src/mushroom_production_contracts.rs','production_source':'src/mushroom.rs',
                    'assertion_relation':'All seven original zero and decimal-place7 radius assertions, six original ticks and last-radius preservation on expiration; actual creation/fraction/radius path.',
                    'bounds':'Exactly original current0..5, start0, interval1000ms, range10f32 and duration5f64; no assumes. The full kernel/operation input domain is separate and unproved.',
                    'status':'FAILED_NUMERIC_MODEL_CORRESPONDENCE_NO_PROOF_CREDIT'}]
        if c['source']=='python/destiny/test/test_ball.py' or (c['source']=='python/destiny/test/ballpark/test_lifecycle_management.py' and c['fixture']=='TestBallpark' and c['test']=='test_add_ball_creates_a_client_ball'):
            record=records[-1]
            if 'retained-object-membership-and-identity' not in record['required_obligations']:
                record['required_obligations'].append('retained-object-membership-and-identity')
            record['production_paths']=sorted(set(record['production_paths']+['src/ball_defaults.rs','src/ball_lifecycle.rs']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['remaining_gaps']+=['Actual detached default field helper feeds native owned snapshots under a generation/client-class tag with one initial lease, outside park membership. Native constructor, identity/default, no-rotation and detached formation cases execute; all12Ball and27lifecycle originals pass0skips at the development library.',
                'Kani actual constructor helper accepted69SUCCESS2SAT, receipt0cc70f94d68f4344a8277ee334271e3b; exact38default assertions retained. This proves helper fields, not arena/native allocation, snapshot/class/interpreter/lease/caller or full object operations.',
                'Detached property mutation, complete descriptor/client/physical/class-count lifetime, snapshot extra identity fields, private arena collection, CPP/Blue class identity and all counter/gate/Mutex/foreign/OS correspondence remain required. Source capsule float conversion<=0 now produces actual value_error and Python ValueError; all broader nonfinite/API/caller domains remain separately required.']
            if c['test']=='test_default_values':
                record['production_kernel_candidates']=[{'harness':'ball_default_production_contracts::original_detached_ball_constructor_literals',
                    'proof_source':'verification/kani/src/ball_default_production_contracts.rs','production_source':'src/ball_defaults.rs',
                    'assertion_relation':'All38original inherited scalar/flag/position/velocity/mode/sentinel/formation/empty-mini/detachment assertions; actual native field helper consumed by constructor snapshot adapter.',
                    'bounds':'Exact default Ball constructor, no assumptions or loops. Every native/Python/arena/class/lease/default fixture and output adapter remains separate.',
                    'status':'ACCEPTED_DEVELOPMENT_KERNEL_ONLY_NOT_FULL_CASE'}]
        if c['source']=='python/destiny/test/ballpark/test_lifecycle_management.py' and c['fixture'] in ('TestAdditionsAndDeletions','TestBubbleTransitions','TestClearAll'):
            record=records[-1]
            record['production_paths']=sorted(set(record['production_paths']+['src/bubbles.rs','python/destiny/_util/decometaclass.py']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['remaining_gaps']+=['Actual spherical source footprints/connected components use unbounded native collections; insertion-ordered action and interactive dictionaries preserve lists versus bubble ID values, duplicate users, aliased output maps, invalid-user prefixes and pre-callback output publication. Source park callback14 uses the actual owned call batch/FFI gate; wrapped base allocation precedes decorator initialization.',
                'All27lifecycle original cases passnative0skips; callbacks/reentry/aliases have21focused native passing checks at development stage. Runtime observations are not proof.',
                'Full bubble Kani65timeouta029b1be44264247afb41d0454b46671; split update65timeouts6efc6dd3fe1c4dc1b746871f24ea9076 and72cc9aed10fc427cad0c5350844368e8 after no-copy optimization. Separate initialization candidate unverified; every original assertion and their union retained.',
                'Master/static/nonzero-source partition insertion, live movement/insertion and snapshot/rollback/extra identity state, pointer/component/geometry/candidate/filter/IEEE adapters, arbitrary Python index/dict subclass/GIL/frame/cancellation and gate/progress correspondence remain required.']
        if c['source'].endswith('test_movement_controls.py'):
            record=records[-1]
            record['production_paths']=sorted(set(record['production_paths']+['src/missile.rs','src/warp_entry.rs','src/ball_lifecycle.rs','src/notification_batch.rs']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['remaining_gaps'] += [
                'All44qualified original movement cases pass native0skip with actual ABI calls in movement-complete-native-part1/2.json,32+12cases. This is historical development execution, not current classified proof or universal native correspondence.',
                'Native Stop/Goto/Follow/Orbit/MissileFollow/FormationFollow/WarpTo/EntityWarpIn/LaunchMissile use owned callback continuations. Explicit generation-keyed source follower sets preserve reentrant stale entries and remove invalid/retargeted entries only at source cleanup. GetFollowers omits absent IDs but retains live mismatched/moribund entries. The source registry is unordered: no source total follower order is claimed.',
                'Source fields/decisions have scoped Kani and Verus kernel observations. Exact original Follow removal specification was rejected46UNREACHABLE (48da2f56a8b74cf2861b27d38523e759); launcher velocity kernel rejected3UNREACHABLE rounding checks (47c26c052a034faf8a88d9893d7440f6). No predicate/tolerance/check was discarded.',
                'Verus actual continuation prepend preserves arbitrary valid container sequences, receipt87d6042e7f7347438db20282716a446e. Bounded concrete queue Kani was interrupted twice without completed receipts and remains unverified; charged submissions recorded in kani-continuation-prepend-interruption.json.',
                'Full registry/collection/Stop/mode/final-cleanup/reentrant trace, pointer/generation adapters, every source command/evolution/IEEE domain, weak-memory/caller/gate/Mutex/Python/engine and progress correspondence remain required. Native snapshots rebuild normal follower relations; exact stale registry/stream/restore and mini cleanup correspondence remain required. Warp master PostEvent, dynamical SnapOrientation and complete sensor correspondence remain pending.']
            record['production_call_path']=['Exact ordered original calls and fixture/helper operations retained in inventory.json',
                'Python facade -> NativeBackend.invoke -> protected_runtime_operation (call lease and reentrant gate)',
                'CompatRuntime.dispatch_park -> source command fields/decisions -> operation-owned callbacks/continuations',
                'Callbacks execute outside data Mutex, complete the mode store, then resume before the caller suffix']
            if c['fixture']=='TestMissileFollow' and c['test']=='test_sets_ids_correctly':
                record['production_kernel_candidates']=[{'harness':'missile_production_contracts::original_missile_follow_ids_fields',
                    'proof_source':'verification/kani/src/missile_production_contracts.rs','production_source':'src/missile.rs',
                    'assertion_relation':'Original41followId and42ownerId constrain actual fields consumed by native follow assignment. Source defaults radii/velocity0 and IDs1/2/3; effect0, signed-zero range, destination0 and speed1 also checked.',
                    'status':'HISTORICAL_KERNEL_ONLY_62SUCCESS3SAT_9e09092c59db48898ae2e32cebda3421',
                    'remaining':'Complete fixture/membership/default/teardown/callback/registration/adapter/ABI operation relation still required.'}]
            if c['fixture']=='TestLaunchMissile' and c['test'] in ('test_direction_of_non_aimed_missiles','test_additional_velocity_of_missile_is_the_same_as_the_max_velocity_of_its_owner_if_above_limit','test_initial_velocity_of_owner_has_impact_of_non_aimed_missile_direction_and_speed','test_direction_of_aimed_missiles'):
                record['production_kernel_candidates']=[{'harness':'missile_production_contracts::original_launcher_velocity_examples',
                    'proof_source':'verification/kani/src/missile_production_contracts.rs','production_source':'src/missile.rs',
                    'assertion_relation':'All three component predicates and original places4/3 comparisons, plus two exact tuple predicates, preserve original313..315/324..326/335/343. Owner0 or X100, max0 or1234, target deltaX10000, default stationary direction-Z; actual rotation/position adapters remain separate.',
                    'status':'REJECTED_118SUCCESS2SAT3UNREACHABLE_47c26c052a034faf8a88d9893d7440f6'}]
            if c['fixture']=='TestEntityWarpIn':
                record['production_kernel_candidates']=[{'harness':'warp_entry_contracts::total_warp_command_action',
                    'proof_source':'verification/kani/src/warp_entry_contracts.rs','production_source':'src/warp_entry.rs',
                    'assertion_relation':'Entire IEEE-f64 derived distance and bool decision domain, including threshold/zero/NaN/infinite/redundant witnesses,3SUCCESS6SAT f56202b1896c43909d5b0e7ebf1e6265. This decision kernel alone is not any complete original mode/velocity assertion.',
                    'status':'HISTORICAL_DECISION_KERNEL_ONLY'},
                    {'harness':'warp_entry_contracts::original_entity_immediate_warp_decision_and_fields',
                    'proof_source':'verification/kani/src/warp_entry_contracts.rs','production_source':'src/warp_entry.rs',
                    'assertion_relation':'Immediate default source position1e10^3, target0, time0/range0: source command selects Warp and actual arrival fields are feasible;81SUCCESS3SAT f31bca57ed4541f5b50d0511c521a51d. Actual mode publication, approx-AU velocity scenario and short-Goto scenario still require their complete separate relations.',
                    'status':'HISTORICAL_FIELDS_KERNEL_ONLY'}]
        if c['source'].endswith('test_getters_and_setters.py'):
            record=records[-1]
            record['production_paths']=sorted(set(record['production_paths']+['src/accuracy.rs','src/harmonic.rs','src/formation.rs','src/ball_lifecycle.rs','src/notification_batch.rs']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['remaining_gaps'] += [
                'Original50case baseline27native passed; separate7accuracy/6remote-Ego/8harmonic-mode/1formation cases passed with actual ABI calls across development artifacts. Their49distinct union is not a final current family run. Velocity source YPR representation is still wrong; current getter computes physical quaternion XYZ angles.',
                'Accuracy actual double formula preserves original expression order, f32 radius conversions and in/out parameters. Rejection fields Kani whole IEEE inputs9SUCCESS4SAT239fb67492e44925b9ddfe1e61cdca50 and original fields9SUCCESS2SATbbe6efb9764f4f5e93499658c76abb14 are kernel observations. Admission, complete pow/sqrt/geometry and nonfinite JSON/API/Python/ABI correspondence remain required.',
                'Remote source free/follow/bubble/missing-target selector passed Kani1SUCCESS8SATa53613a02bdf4cc9a370cd818e9ebbf7 and Verus24checks4ad141a1be054d03b528d6c59068df40. Complete original query rejected source mass0 inflated-radius NaN division plus58UNREACHABLE5b7195217c2f424ca1ef1b0de2ac5b91. Native finite shortcut/compiler/NaN/partition relation remains unresolved.',
                'Harmonic source identity/action kernel passed12SUCCESS6SAT247779c07ecf405aad0cc4fd0bc53a5a and concrete fields10SUCCESS3SATe3725069037d42edb3ed4b19ce75a1c7. Original mode publication, snapshots/stream/identity/generation/FFI/Python and full state frame remain required. Boid/Troll/Rigid commands execute; Troll petrification and full Missile/Formation evolution are pending.',
                'Formation shrink source callback ordering has a native original/derived regression and actual size/slot decision48SUCCESS5SAT0056a96bfd8740d98c2b0f1b39001787. Full source specification rejected35UNREACHABLEe121100e6f334b5ba97d72f149cb33d7. Complete captured pointer/copied-ID/stale registry/slot masks/unset/mode/generation/ABI relation remains required.',
                'No whole case or complete production concurrency credit is inferred from these kernel/native observations. Every assertion and setup dependency still requires its complete mapping and current conjunction evidence.']
        if c['id']==CALLBACK_CASE:
            record=records[-1]
            record['production_call_path']=['Ballpark.Evolve -> NativeBackend.invoke',
                'dbc_runtime_call_with_mode_callback -> protected_runtime_operation (live call reference + operation gate)',
                'CompatRuntime.dispatch_yielding -> begin_callback_tick -> prepare_motion_tick (original numeric projection)',
                'drain_callbacks drops data MutexGuard -> Python handler before complete_mode_callback stores mode',
                'handler -> Ballpark.AddBall -> nested native call -> CompatRuntime.add_ball -> add_allowed(phase)==false',
                'Python dispatcher reports RuntimeError subclass; handler assignment does not complete; stored field remains None',
                'advance_callback_tick -> operation-owned notification continuation -> finish_callback_tick publishes time and restores idle phase']
            for assertion in record['assertion_mappings']:
                assertion['behavioral_rule']='The Warp-to-Stop handler sets warp_done, its attempted AddBall raises during active evolution, and the failed assignment leaves ball_added_during_evolve at its initialized None value. Mode notification precedes mode storage; membership and rejected repair fields are preserved.'
                assertion['verus_symbols']=['evolution_callbacks::add_allowed','evolution_callbacks::feasible_callback_rejection_trace']
                assertion['review']='SOURCE_AND_NATIVE_SCENARIO_REVIEWED_PHASE_KERNEL_PARTIAL_FULL_CONJUNCTION_PENDING'
            record['additional_reviewed_dependencies']=[
                {'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Thunkers.cpp',[[66,187],[358,383]]),
                    ('src/Ball.cpp',[[905,941],[1885,1923]]),
                    ('src/Ballpark.cpp',[[421,658],[913,948],[4072,4366],[4556,4650]]),
                    ('python/destiny/test/net/client/helpers.py',[[6,47]])]]
            record['remaining_gaps']=['Whole operation gate/Mutex and tracked permission correspondence, caller pointer validity and Box ownership.',
                'IEEE-754/libm and Warp adapter relation plus original collision/update/command event ordering outside this single-ball scenario.',
                'Moving-state snapshot/rollback and reentrant command/cancellation/error scenarios.',
                'Current classified Kani value-domain evidence and final source-bound Verus/native observations.']
        if c['source'].endswith('test_callbacks.py') and c['fixture']=='TestProximitySensors':
            record=records[-1]
            record['production_call_path']=['BallEventSpy wraps the native ball and registers weak event handler',
                'Ball.AddProximitySensor -> native descriptor validation and clock state',
                'Ballpark.Evolve -> advance_callback_tick -> operation-owned NotificationPass with source Range/Tracking/Sensor stage order',
                'next_tick_notification reads live geometry/membership -> unlocked synchronous DoProximity -> resumes only this pass; nested reads own no remaining candidates',
                'finish_callback_tick publishes time after callbacks and releases the evolution phase']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ball.cpp',[[244,296],[494,781]]),
                    ('src/Partition.cpp',[[283,300],[415,445]]),
                    ('src/Ballpark.cpp',[[5822,5864]]),
                    ('python/destiny/test/helpers.py',[[41,46],[159,204],[248,266]])]]
            for assertion in record['assertion_mappings']:
                assertion['behavioral_rule']='Preserve the exact ordered callback argument list at this source assertion: source elapsed addition/reset and flag filtering, sensor removal, range entry/exit and callback phase determine the observed sequence.'
                assertion['verus_symbols']=['evolution_callbacks::candidate_admitted','evolution_callbacks::take_next','evolution_callbacks::feasible_owned_notification_continuation']
                assertion['review']='SOURCE_AND_NATIVE_SCENARIO_REVIEWED_PARTIAL_KERNELS_FULL_CONJUNCTION_PENDING'
            record['remaining_gaps']=['Full caller/operation-gate/Mutex/Box correspondence and current frozen evidence.',
                'Source partition/candidate enumeration, owner-moribund continuation, cloak/singular sensor relation, inactive sensor clock and every replacement/cancellation caller/epoch semantics.',
                'Every numeric input/adapter/libm/engine boundary and the original broader nonfinite API correspondence.']
        if c['source']=='python/destiny/test/net/client/test_ticker.py':
            record=records[-1]
            record['required_obligations']+=['client-scheduler-ownership']
            record['production_call_path']=['Ticker.set_ballpark -> facade manual scheduler ownership (background driver stopped without holding its join lock)',
                'Ticker SetState -> atomic native snapshot restore -> ego/start -> consistent CaptureFullState',
                'Ticker.store_state -> native CaptureFullState; paused states skipped; original first-and-last-five pruning',
                'Ticker.synchronize_to_simulation_time -> last eligible snapshot in stored order -> native restore -> ordered parent Evolve replay']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('python/destiny/net/client/_ticker.py',[[33,340]]),
                    ('python/destiny/net/client/_baseticker.py',[[21,69]]),
                    ('python/destiny/test/net/client/helpers.py',[[1,47]])]]
            record['remaining_gaps']=['Original Blue MemStream/interpreter/native stream format is unavailable locally; adapted native JSON replay is separate runtime evidence.',
                'Manual scheduler publication, join, callback reentry, retained-handle and snapshot/restore correspondence require current formal obligations.',
                'Portable ticker changes SetState callback/clear timing, same-timestamp replacement, failure continuation, and catch-up bound relative to the original. Existing hardened behavior is not counted as original parity.',
                'Original full motion/collision/rotation state and all assertion-level replay postconditions require independent numeric/adapter proofs and exact runtime scenarios.',
                'Actual Python claim reserves the weak owner under the driver RLock before joining; driver start checks that owner under the same lock. Failed claim conditionally restores its own reservation. Native two-thread regression passes and the prior publish-after-join body fails; Python/RLock/weakref/join, generation/cancellation and progress remain unproved.',
                'Parent aliases bind the concrete facade base method and bypass recursive subclass overrides. Native regression passes; Blue decorator/object identity correspondence remains required.']
            if c['test']=='test_late_arrival':
                record['runtime_scope']='Adapted native fixture retains3source balls, ten setup ticks, SetState stamp0, ticks10/11/12/13 and Orbit/Stop assertions, using native JSON/BytesIO. Unchanged original Blue import is UNAVAILABLE.'
                record['concrete_scope']='Complete specification candidate client_ticker_contracts::TestTicker__test_late_arrival retains all setup/assertions/unwind129; Kani120 TIMEOUT5ffbf9643ca4489da9e989185d01b8c1. No proof credit.'
        if c['id']=='python/destiny/test/ballpark/evolve/test_orbit.py::TestOldOrbit::test_orbit_ball':
            record=records[-1]
            record['production_call_path']=['Original helper setters -> native mass/radius/max-speed/agility/speed-fraction components',
                'Orbit -> source follow guards and mode control -> ordered prepare_motion_tick',
                'original_motion_input transfers both radii and source-grouped time factor -> original_old_orbit_thrust_with_friction',
                'original_integrate_duration -> production position/velocity commit -> original ten-point rounding helper']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ballpark.cpp',[[735,796],[1116,1397],[4951,4983]]),
                    ('python/destiny/test/helpers.py',[[129,150],[208,231]]),
                    ('python/destiny/_util/settings.py',[[1,31]])]]
            record['remaining_gaps']=['Exact10-step Kani scenario timed out at120seconds; no omission of points, source rounding or unwinding assertions.',
                'Complete adapter signedness/f32 conversions, libm/engine/collision/orientation and gate/Mutex/ABI ownership proofs remain required.',
                'Unchanged native original scenario passes after the missing-radius fix; final exact source/artifact receipts and classified mapping remain pending.']
        if c['source']=='python/destiny/test/ballpark/evolve/test_orbit.py' and c['fixture']=='TestNewOrbit':
            record=records[-1]
            record['production_paths']+=['python/destiny/_settings.py','python/destiny/_util/settings.py',
                'verification/spec/src/original_motion.rs']
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['production_call_path']=['Original enable_new_orbit -> real settings.Apply -> NativeBackend construction option snapshot',
                'Orbit -> mode/follow controls -> ordered production prepare_motion_tick',
                'Actual f32 radii/agility/speed input and source cached factor -> original_new_orbit_thrust_with_environment(friction,dt)',
                'Production integration/position/velocity commit -> every original distance and dot-product/velocity assertion; original iteration bounds unchanged']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ballpark.cpp',[[666,753],[1116,1261],[1398,1513],[4951,4983]]),
                    ('python/destiny/test/helpers.py',[[207,232]]),('python/destiny/_util/settings.py',[[1,31]])]]
            record['remaining_gaps']=['Exact400-step right-distance Kani candidate timed out at120seconds; other original full numerical trace obligations remain unverified, with all iterations/tolerances preserved.',
                'Runtime observations on the built native library cover all four unchanged new Orbit cases (400/2000/2000/200ticks); they do not prove arbitrary FP inputs, schedules, adapters, engine/collision/orientation or ABI ownership.',
                'Original global setting changes affect existing parks; portable/native per-park option snapshots retain a separately pending dynamic-configuration correspondence obligation.',
                'Final exact source/artifact receipts and classified assertion/dependency mappings remain required.']
        if c['id']=='python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_correct_collision_order':
            record=records[-1]
            record['production_paths']+=['python/destiny/_settings.py']
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['production_call_path']=['Original fixture creates park then Apply(useIterativeCollision=True) -> validated settings revision snapshot',
                'NativeBackend selects runtime -> one native Runtime.Configure operation before the requested call',
                'Evolve -> first compatibility bubble assignment -> prepare_destiny_swept_bounds before broad-phase collection',
                'Real Avian detection/solver -> original assertions: farther target stationary and moving ball remains before fixed sphere']
            for assertion in record['assertion_mappings']:
                assertion['behavioral_rule']='Preserve the exact original assertion after the ordered single tick: target x/y/z remain 0/0/10 with the original default decimal-place comparison, and fixed ball z is strictly greater than moving ball z.'
                assertion['review']='UNCHANGED_NATIVE_CASE_REVIEWED_FULL_FORMAL_CONJUNCTION_PENDING'
            record['remaining_gaps']=['Exact original Kani candidate with unwind177 timed out at120seconds; all assertions/setup retained.',
                'Source elastic impact/invalidation/overlap/candidate order and every production numeric/adapter/engine/worker/synchronization boundary remain required;10neighboring sphere cases still fail native assertions.',
                'Pre-solver bounds cover native linear root-sphere motion; accelerated control overrides, arbitrary host shapes/forces, f32 BVH conversion and source-original collision correspondence remain unproved.',
                'Current frozen classified receipts and full dependency/assertion mappings remain pending.']
        if c['source']=='python/destiny/test/ballpark/evolve/test_iterative_collision.py':
            record=records[-1]
            record['production_paths']=sorted(set(record['production_paths']+['src/iterative_collision.rs','python/destiny/_settings.py']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['production_call_path']=[
                'Original fixture park construction and Apply(iterative=True) -> native configuration revision before operation',
                'Evolve -> exclusive operation gate and callback-aware phase -> ordered source control accelerations',
                'Old-orientation sphere branch -> original-impact histories and ID-ordered configured collision passes in src/iterative_collision.rs',
                'Prepare position/velocity publication before Avian update -> publish under data Mutex -> source-ordered synchronous notifications before timestamp publication']
            record['additional_reviewed_dependencies']=[
                {'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ballpark.cpp',[[332,403],[1988,2713]]),
                                  ('src/CollisionBallProperties.cpp',[[1,260]]),
                                  ('src/Ball.cpp',[[1663,1875]]),
                                  ('src/Partition.cpp',[[293,537]])]]
            record['implementation_scope']='Actual old-orientation parent-sphere path, dynamically sized collections, configured dt/iteration count. Capsule/box source checks and fixed-parent translation are implemented; source allocated identities/enumeration, mini-ball entities and dynamical orientation remain incomplete.'
            record['remaining_gaps']=[
                'All original source assertions and setup identities retained; current assertion-level formal receipts, input/output adapter and caller proofs remain required.',
                'Original partition/harmonic/dead-bubble filters, retained collision histories across configuration/membership changes, mini-shape/static and dynamical-orientation correspondence remain unproved.',
                'Production candidate scan currently has quadratic worst-case work/storage; supported maximum-park performance/resource and spatial partition correspondence remain pending.',
                'Avian/Bevy worker schedule, host-force and f32 broad-phase integration, rollback caches, IEEE/libm, ABI/gate/Mutex/allocation/progress relations remain required.',
                'All32unchanged fixture-qualified native scenarios passed, zero skips, ordered-all-thirty-two-original-native.json. Actual capsule-left Kani20ticks/20passes timed out120seconds (10139bee27ec45839e6eb0b34c78551f). Preliminary substep run32263e40cfa74a288e8e675f460eb83f used prototype mass/agility and is invalid for original correspondence; corrected harness is unverified. Ordinary/native observations remain separate.']
            if c['test']=='test_substep_collision':
                record['additional_concrete_proofs']=[{
                    'manifest':'verification/kani/Cargo.toml',
                    'harness':'iterative_sphere_production_contracts::original_three_sphere_substeps_use_actual_ordered_kernel',
                    'proof_source':'verification/kani/src/iterative_sphere_production_contracts.rs',
                    'production_source':'src/iterative_collision.rs',
                    'assertion_relation':'Original a/b/c velocity and position, all six unittest decimal-place0 assertions, creation IDs a,c,b; actual resolve output constrained.',
                    'bounds':'3actual source bodies; full20passes, dt1, original friction1e-10; unwind257; no assumptions. Full collection/runtime/caller/native/partition domain remains separate.',
                    'status':'CORRECTED_SOURCE_FIXTURE_UNVERIFIED_NO_PROOF_CREDIT'}]
        if c['source']=='python/destiny/test/ballpark/evolve/test_simple_collision.py':
            record=records[-1]
            record['production_paths']=sorted(set(record['production_paths']+['src/iterative_collision.rs','src/noniterative_collision.rs','verification/spec/src/original_collision.rs']))
            record['production_hashes']={p:sha(ROOT/p) for p in record['production_paths']}
            record['production_call_path']=['Evolve -> ID-ordered controls, including source Stop Y damping -> native operation ownership',
                'Actual non-iterative gradient -> shared source sphere/capsule/box potentials using actual friction/dt -> simultaneous output publication',
                'Native notification continuation -> time publication; source partition/allocation/native/host/caller boundaries remain required']
            record['additional_reviewed_dependencies']=[{'source':p,'original_sha256':hashlib.sha256(original(p)).hexdigest(),'regions':regions}
                for p,regions in [('src/Ballpark.cpp',[[793,856],[1339,1345],[2745,3005]]),
                                  ('src/Capsule.cpp',[[197,446]]),('src/OrientedBox.cpp',[[44,128]]),
                                  ('python/destiny/test/helpers.py',[[129,150],[207,236]])]]
            record['remaining_gaps']=['All14unchanged source cases pass native, zero skips (stop-y-source-noniterative-fourteen-native.json); all32iterative cases still pass. These are development observations.',
                'Actual source stopped-overlap10tick production harness ordinarypasses; Kani120 TIMEOUT696ff47898c74bb2ac0bfa4941ccf546. No formal credit.',
                'Source allocation/mini-ball registration and retained identity, partition/harmonic/dead-bubble filters, per-pair scratch bound proof, collision records/missile dispatch and complete caller/domain/numeric/synchronization/worker/host relations remain required.',
                'Native zero/extreme-friction and host-assigned positive-bubble partitions retain the existing Avian path; original correspondence is explicitly incomplete. No source domain or release claim is inferred.']
    obligations={
        'abi-machine-counters':{'entry':'verification/verus/ffi_lifecycle.rs','scope':'Actual production retain_count/release_count bodies over every usize; logical ownership/feasible initialization are separate symbols. Atomic RMW/pointer/reclamation correspondence remains required.','production':['src/ffi_lifecycle.rs','src/ffi.rs']},
        'abi-reference-ownership':{'entry':'verification/verus/ffi_lifecycle.rs','scope':'Logical tokenized single-allocation reference ownership for arbitrary finite permitted schedules up to usize::MAX. Tokens require a concrete pointer/Box/SC atomic and caller relation.','production':['src/ffi.rs','include/destiny_bevy_compat.h']},
    }
    obligations['evolution-phase-ownership']={'entry':'verification/verus/evolution_callbacks.rs',
        'scope':'Actual total production u8 phase transitions and feasible callback/rejection trace. Gate, ECS mutation and foreign callback correspondence are separate required conjuncts.',
        'production':['src/evolution_phase.rs','src/operation_gate.rs','src/ffi.rs','src/runtime.rs']}
    obligations['callback-timing-and-reentry']={'entry':'verification/verus/evolution_callbacks.rs',
        'scope':'Actual source flag and operation-batch extraction kernels, with feasible parent/nested-read trace. Numeric timing is separately executed with Kani; full callback invocation/caller/world/cancellation relation remains required.',
        'production':['src/proximity.rs','src/notification_batch.rs','src/runtime.rs','src/ffi.rs','python/destiny/_backend.py']}
    obligations['client-scheduler-ownership']={'entry':None,
        'scope':'Derived from the original client manual pre/Evolve/post sequence and the added facade driver: reserve manual ownership before join, exclude automatic starts under the same lock, preserve generation/cancellation and conditional rollback, and establish permitted callback/parent-alias histories. Regression observations do not prove every schedule or progress.',
        'production':['python/destiny/_facade.py','python/destiny/net/client/__init__.py']}
    obligations['collision-identity-counters']={'entry':'verification/verus/ffi_lifecycle.rs',
        'scope':'Actual source signed-i64 local/high policy and rejection over both actor-role partitions. Allocation-map uniqueness, object registration, reset/restore, callers and operation ownership are separate required conjuncts.',
        'production':['src/collision_identity.rs','src/runtime.rs','python/destiny/_facade.py']}
    obligations['retained-object-membership-and-identity']={'entry':'verification/verus/ffi_lifecycle.rs',
        'scope':'Actual retained-object kernel over all bool/usize/u8 inputs plus checked executable remove/alias/release/clear/delayed-repair witnesses. Park membership is distinct from object leases. Registry, non-reused generation, snapshot restoration, source caller/cleanup, Python and locking correspondence remain required.',
        'production':['src/ball_lifecycle.rs','src/runtime.rs','python/destiny/_backend.py','python/destiny/_facade.py']}
    for name in {r for c in records for r in c['required_obligations']} - set(obligations):
        obligations[name]={'entry':None,'scope':'Required production-derived obligation; source-specific contracts, executable link, theorem inventory and current evidence remain pending.','production':[]}
    for name,obligation in obligations.items():
        obligation['required_conjuncts']=['reviewed_source_and_caller_contract','current_logical_theorems',
            'actual_production_relation','complete_residual_correspondence','applicable_concrete_and_runtime_evidence']
        obligation['residual_obligations']=['Complete production wrapper, original-case and boundary correspondence for '+name]
    document={'schema':1,'original_commit':PIN,'formation_ci_commit':'114e89fa584f0e1a81eb0a679aabf41af8870072',
        'inventory_sha256':sha(ROOT/'verification/cases/inventory.json'),'generator_sha256':sha(Path(__file__)),
        'denominator':{'occurrences':531,'qualified_identities':531,'unqualified_pairs':509,'source_files':34},
        'build_features':['default','carbon-network'],'target_scope':'root Linux x86_64 build; other supported target/host configurations require their own validated topology and gates',
        'notice':'Applicability/accountability only. No assurance booleans or completed case are assigned by generation. Supporting/non-applicable cases remain in the denominator and are never counted as proved.',
        'obligations':obligations,'cases':records}
    target=ROOT/'verification/cases/concurrency_mappings.json';target.write_text(json.dumps(document,indent=2)+'\n')
    print(json.dumps({'qualified_cases':531,'relevance_counts':dict(Counter(r['concurrency_relevance'] for r in records)),'derived_obligations':len(obligations),'status':'MAPPED_NO_PROOF_CREDIT','path':str(target)}))

if __name__=='__main__':main()
