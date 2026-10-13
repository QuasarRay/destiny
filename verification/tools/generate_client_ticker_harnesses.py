#!/usr/bin/env python3
"""Translate the ten complete pinned client ticker scenarios, one OPEN case.

The shared fixture executes all three real bodies, ten original evolution steps,
the actual stream roundtrip, and pre/evolve/post order. Every original assertion
is emitted at its source line. Unknown syntax or changed helpers fail closed.
"""
import argparse, ast, hashlib, json, subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
REV='ef738406b61ad06bae2f402857e464735ab106a2'
SOURCE='python/destiny/test/net/client/test_ticker.py'
HELPERS='python/destiny/test/net/client/helpers.py'
TARGET=ROOT/'verification/kani/src/client_ticker_contracts.rs'

def original(path):
    return subprocess.run(['git','show',REV+':'+path],cwd=ROOT,capture_output=True,text=True,check=True).stdout

def exact_method(cls,name,expected):
    actual=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==name)
    wanted=ast.parse(expected).body[0]
    if ast.dump(actual)!=ast.dump(wanted):raise ValueError('Changed reviewed helper '+name)

def review_fixture(tree):
    cls=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name=='TestTicker')
    if [ast.unparse(x) for x in cls.bases]!=['unittest.TestCase']:raise ValueError('Changed fixture inheritance')
    constants={n.targets[0].id:ast.literal_eval(n.value) for n in tree.body if isinstance(n,ast.Assign)}
    if constants!={'BALL_ID_1':1,'BALL_ID_2':2,'BALL_ID_3':3,'TICKS_ELAPSED':10}:raise ValueError('Changed fixture constants')
    exact_method(cls,'setUp','''def setUp(self):
 self._error_handler = TestErrorHandler()
 self._client_ticker = TestClientTicker()
 self._ticker = Ticker(self._error_handler, self._client_ticker)
 self._ballpark = destiny.Ballpark()
 self._ticker.set_ballpark(self._ballpark)
 self._do_setstate()
 self._tick(TICKS_ELAPSED)
''')
    exact_method(cls,'tearDown','''def tearDown(self):
 del self._ballpark
 del self._ticker
 del self._client_ticker
 del self._error_handler
''')
    exact_method(cls,'_get_park_with_state','''def _get_park_with_state(self):
 bp = destiny.Ballpark()
 ball_1 = add_ball_to_park(bp, ball_id=1, x=0, y=0, z=0)
 ball_2 = add_ball_to_park(bp, ball_id=2, x=1000, y=0, z=0)
 ball_3 = add_ball_to_park(bp, ball_id=3, x=-1000, y=0, z=0)
 bp.Orbit(ball_2.id, ball_1.id)
 bp.GotoPoint(ball_3.id, 0, 0, 0)
 bp.GotoDirection(ball_1.id, 0.0, 1.0, 0.0)
 for _ in range(TICKS_ELAPSED):
  bp.Evolve()
 return bp
''')
    exact_method(cls,'_do_setstate','''def _do_setstate(self, current_time=0):
 park_with_state = self._get_park_with_state()
 s = blue.MemStream()
 park_with_state.WriteFullStateToStream(s)
 self._ticker.update([[current_time, ("SetState", (s.Read(), BALL_ID_1))]], False)
''')
    exact_method(cls,'_tick','''def _tick(self, time_stamp):
 self._ticker.do_pre_tick()
 self._ballpark._parent_Evolve()
 self._ticker.do_post_tick(time_stamp)
''')
    helpers=ast.parse(original(HELPERS))
    add=next(n for n in helpers.body if isinstance(n,ast.FunctionDef) and n.name=='add_ball_to_park')
    names=[n.arg for n in add.args.args]
    expected=['park','ball_id','mass','radius','max_velocity','is_free','is_global','is_massive','is_interactive','is_space_junk','x','y','z','vx','vy','vz','agility','speed_fraction']
    defaults=[ast.literal_eval(n) for n in add.args.defaults]
    if names!=expected or defaults!=[-1,1,1.0,1.0,True,False,True,True,False,0,0,0,0,0,0,1.0,1.0] or add.decorator_list:
        raise ValueError('Changed actual AddBall helper defaults')
    if len(add.body)!=1 or not isinstance(add.body[0],ast.Return) or ast.unparse(add.body[0].value)!='park.AddBall('+', '.join(expected[1:])+')':
        raise ValueError('Changed full AddBall helper call')
    # Test interfaces contain the source's no-op callbacks and mutable boolean
    # error flags. Validate all their bodies, including the original typo.
    error=next(n for n in helpers.body if isinstance(n,ast.ClassDef) and n.name=='TestErrorHandler')
    exact_method(error,'__init__','''def __init__(self):
 self._fatal_desync_occured = False
 self._recoverable_desync_occurred = False
''')
    exact_method(error,'on_fatal_desync','''def on_fatal_desync(self):
 self._fatal_desync_occured = True
''')
    exact_method(error,'on_recoverable_desync','''def on_recoverable_desync(self):
 self._recoverable_desync_occurred = True
''')
    exact_method(error,'fatal_desync_occurred','''@property
def fatal_desync_occurred(self):
 return self._fatal_desync_occured
''')
    exact_method(error,'recoverable_desync_occurred','''@property
def recoverable_desync_occurred(self):
 return self._recoverable_desync_occurred
''')
    interface=next(n for n in helpers.body if isinstance(n,ast.ClassDef) and n.name=='TestClientTicker')
    for name,args in [('on_set_state','self'),('on_ballpark_local_action','self, func_name, args'),('clean_up_after_ball_removal','self, ball_id, ball, is_terminal=False'),('clean_up_after_multiple_ball_removal','self, ball_ids, is_release=False')]:
        exact_method(interface,name,'def '+name+'('+args+'):\n pass\n')
    exact_method(interface,'should_log_actions','def should_log_actions(self):\n return True\n')
    exact_method(interface,'get_ball_destruction_delay','def get_ball_destruction_delay(self, ball, is_terminal=False):\n return 0\n')
    exact_method(interface,'get_ball_destruction_delays','def get_ball_destruction_delays(self, ball_ids, is_release=False):\n return {ball_id: 0 for ball_id in ball_ids}\n')
    return cls

PREAMBLE='''// Copyright © 2023 CCP ehf. Original tests' MIT license retained.
// Generated by generate_client_ticker_harnesses.py from pinned Git blobs.
// Three real bodies, ten fixture ticks, actual serialized snapshots and full
// history processing. Native Python/Blue, method replacement and object alias
// correspondence remain separate requirements; no production credit is claimed.
#![allow(non_snake_case)]
use destiny_original_spec::{ClientHistoryAction,ClientHistoryEvent,ClientHistoryState,
    OriginalBallInit,OriginalClientTicker,OriginalMode,OriginalParkProjection,Vec3};

fn state(events:&[(i64,ClientHistoryAction)])->ClientHistoryState {
    let mut state=ClientHistoryState::new();let mut i=0;
    while i<events.len() {
        if let Some((timestamp,action))=events.get(i) {
            assert!(state.push(ClientHistoryEvent{timestamp:*timestamp,action:*action}));
        }i+=1;
    }state
}
fn tick(ticker:&mut OriginalClientTicker,time_stamp:i64) {
    assert!(ticker.do_pre_tick()==Ok(()));
    assert!(ticker.parent_evolve()==Ok(()));
    assert!(ticker.do_post_tick(time_stamp)==Ok(()));
}
fn ticker_fixture(ticker:&mut OriginalClientTicker) {
    // Default constructs the fresh park in its final storage. set_ballpark's
    // three reference assignments all point to this same initialized value.
    // Avoid copying the large park through a fixture return on the test stack.
    ticker.has_ballpark=true;
    let mut bp=OriginalParkProjection::default();
    let mut ball_ids=[0_i64;3];let positions=[0.0,1000.0,-1000.0];let mut i=0;
    while i<3 {
        let id=i64::try_from(i+1);assert!(id.is_ok());
        let ball=OriginalBallInit{id:id.unwrap_or(0),mass:1.0,radius:1.0,max_velocity:1.0,
            is_free:true,is_global:false,is_massive:true,is_interactive:true,is_space_junk:false,
            position:Vec3{x:positions[i],y:0.0,z:0.0},velocity:Vec3::ZERO,agility:1.0,speed_fraction:1.0};
        ball_ids[i]=bp.add_ball(ball);assert!(!bp.scope_error);i+=1;
    }
    bp.follow(ball_ids[1],ball_ids[0],1.0,true);
    bp.goto_point(ball_ids[2],Vec3::ZERO);
    bp.goto_direction(ball_ids[0],Vec3{x:0.0,y:1.0,z:0.0});
    for _ in 0..10 { assert!(bp.evolve_two_spherical_bodies()==Ok(())); }
    let stream=bp.write_original_full_state(None);assert!(stream.is_ok());
    let index=ticker.retain_received_stream(stream.unwrap_or_default());assert!(index.is_ok());
    assert!(ticker.update(&state(&[(0,ClientHistoryAction::SetState{stream_index:index.unwrap_or(usize::MAX),ego_id:1})]),false)==Ok(()));
    tick(ticker,10);
}
'''

def expr(n):
    if isinstance(n,ast.Constant) and isinstance(n.value,(int,bool)):
        return str(n.value).lower() if isinstance(n.value,bool) else str(n.value)+'_i64'
    if isinstance(n,ast.Name) and n.id in {'BALL_ID_1','BALL_ID_2','BALL_ID_3','TICKS_ELAPSED'}:
        return str({'BALL_ID_1':1,'BALL_ID_2':2,'BALL_ID_3':3,'TICKS_ELAPSED':10}[n.id])+'_i64'
    if isinstance(n,ast.Name) and n.id=='success':return 'success'
    if isinstance(n,ast.BinOp) and isinstance(n.op,(ast.Add,ast.Sub)):
        return '('+expr(n.left)+('+' if isinstance(n.op,ast.Add) else '-')+expr(n.right)+')'
    known={'self._ballpark.currentTime':'ticker.park.current_time','ball_two.mode':'ticker.park.retained(ball_two).mode',
        'destiny.DSTBALL_ORBIT':'OriginalMode::Orbit','destiny.DSTBALL_STOP':'OriginalMode::Stop',
        'self._error_handler.recoverable_desync_occurred':'ticker.recoverable_desync_occurred',
        'self._error_handler.fatal_desync_occurred':'ticker.fatal_desync_occurred'}
    if ast.unparse(n) in known:return known[ast.unparse(n)]
    raise ValueError('Unreviewed expression '+ast.unparse(n))

def incoming(n):
    if not isinstance(n,ast.List):raise ValueError('Expected original state list')
    entries=[]
    for item in n.elts:
        if not isinstance(item,ast.List) or len(item.elts)!=2:raise ValueError('Unreviewed event structure')
        stamp,action=item.elts
        if not isinstance(action,ast.Tuple) or len(action.elts)!=2 or ast.literal_eval(action.elts[0])!='Stop':raise ValueError('Unreviewed selected action')
        args=action.elts[1]
        if not isinstance(args,ast.Tuple) or len(args.elts)!=1:raise ValueError('Unreviewed Stop parameters')
        entries.append('('+expr(stamp)+',ClientHistoryAction::Stop{id:'+expr(args.elts[0])+'})')
    return 'state(&['+','.join(entries)+'])'

def translate(case):
    source,fixture,test=case.split('::')
    if source!=SOURCE or fixture!='TestTicker':raise ValueError('Unreviewed source fixture')
    text=original(SOURCE);cls=review_fixture(ast.parse(text))
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test and n.name.startswith('test_'))
    if fn.decorator_list:raise ValueError('Unreviewed decorators')
    lines=['    let mut ticker=OriginalClientTicker::default();','    ticker_fixture(&mut ticker);']
    for node in fn.body:
        if isinstance(node,ast.Assign) and len(node.targets)==1:
            target=ast.unparse(node.targets[0]);value=ast.unparse(node.value)
            if target=='ball_two' and value=='self._ballpark.balls[BALL_ID_2]':
                lines+=['    assert!(ticker.park.contains(2)); // original dictionary lookup','    let ball_two=2_i64; // retained object alias, checked against the live ID'];continue
            if target=='success' and isinstance(node.value,ast.Call) and ast.unparse(node.value.func)=='self._ticker.synchronize_to_simulation_time' and len(node.value.args)==1 and not node.value.keywords:
                lines+=['    let result=ticker.synchronize_to_simulation_time('+expr(node.value.args[0])+');assert!(result.is_ok());','    let success=result.unwrap_or(false);'];continue
            if target=='self._ticker.synchronize_to_simulation_time' and value=='mock.Mock(return_value=False)':
                lines.append('    ticker.synchronization_mock_return=Some(false); // original explicit mock replacement');continue
        if isinstance(node,ast.Expr) and isinstance(node.value,ast.Call):
            call=node.value;name=ast.unparse(call.func)
            if name=='self._ticker.update' and len(call.args)==2 and not call.keywords:
                lines.append('    assert!(ticker.update(&'+incoming(call.args[0])+','+expr(call.args[1])+')==Ok(()));');continue
            if name in {'self._ticker.do_pre_tick','self._ticker.do_post_tick','self._tick','self._ballpark.RemoveBall'} and not call.keywords:
                if name=='self._ticker.do_pre_tick' and not call.args:code='assert!(ticker.do_pre_tick()==Ok(()));'
                elif len(call.args)==1:
                    value=expr(call.args[0])
                    code={'self._ticker.do_post_tick':'assert!(ticker.do_post_tick('+value+')==Ok(()));',
                        'self._tick':'tick(&mut ticker,'+value+');',
                        'self._ballpark.RemoveBall':'ticker.park.remove_ball('+value+',0);assert!(!ticker.park.scope_error);'}.get(name)
                else:code=None
                if code:lines.append('    '+code);continue
            if name in {'self.assertEqual','self.assertTrue','self.assertFalse','self.assertIn','self.assertNotIn'} and not call.keywords:
                lines.append(f'    // original-assertion: {SOURCE}:{node.lineno}')
                if name=='self.assertEqual' and len(call.args)==2:code=expr(call.args[0])+'=='+expr(call.args[1])
                elif name in {'self.assertTrue','self.assertFalse'} and len(call.args)==1:code=('!' if name=='self.assertFalse' else '')+expr(call.args[0])
                elif name in {'self.assertIn','self.assertNotIn'} and len(call.args)==2 and ast.unparse(call.args[1])=='self._ballpark.balls':code=('!' if name=='self.assertNotIn' else '')+'ticker.park.contains('+expr(call.args[0])+')'
                else:raise ValueError('Unreviewed assertion '+ast.unparse(call))
                lines.append('    assert!('+code+');');continue
        raise ValueError('Unreviewed original statement '+ast.unparse(node))
    lines+=['    assert!(!ticker.park.scope_error);',
        '    // tearDown deletes the four native references; Rust lexical drop is',
        '    // represented here. Blue/native destructor correspondence is pending.',
        '    #[cfg(kani)] kani::cover!(true,"complete original client ticker scenario and fixture reached");']
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+\
        '\n// original-helpers-sha256: '+hashlib.sha256(original(HELPERS).encode()).hexdigest()+\
        '\n#[cfg_attr(kani,kani::proof)]\n#[cfg_attr(kani,kani::unwind(129))]\n#[cfg_attr(test,test)]\nfn '+fixture+'__'+test+'() {\n'+'\n'.join(lines)+'\n}\n'

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);parser.add_argument('--refresh',action='store_true');args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN case required')
    proof=translate(args.case);old=TARGET.read_text() if TARGET.exists() else PREAMBLE
    marker='// original-case: '+args.case+'\n'
    if marker in old:
        if not args.refresh:raise ValueError('Case already translated')
        start=old.index(marker);end=old.find('\n// original-case: ',start+len(marker));end=len(old) if end<0 else end
        old=old[:start]+proof.lstrip('\n')+old[end:];proof=''
    elif args.refresh:raise ValueError('Refresh needs an existing exact case')
    first=old.find('\n// original-case: ')
    if first>=0:old=PREAMBLE+old[first:]
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
