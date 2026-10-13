#!/usr/bin/env python3
"""Translate park-side server actions and full original history/stream helpers.

History construction reuses the reviewed source queue; bytes pass through the
actual original-backed codec. Native/Blue/weakref correspondence stays pending.
"""
import argparse,ast,copy,hashlib,importlib.util,json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2];REV='ef738406b61ad06bae2f402857e464735ab106a2'
SOURCE='python/destiny/test/net/server/test_actions.py';HELPERS='python/destiny/test/net/server/helpers.py'
TARGET=ROOT/'verification/kani/src/server_park_action_contracts.rs'

def original(path):return subprocess.run(['git','show',REV+':'+path],cwd=ROOT,capture_output=True,text=True,check=True).stdout
spec=importlib.util.spec_from_file_location('action_generator',ROOT/'verification/tools/generate_server_action_harnesses.py')
sg=importlib.util.module_from_spec(spec);spec.loader.exec_module(sg)

FIELDS={'id':'id','mass':'mass','radius':'radius','maxVelocity':'max_velocity','isFree':'is_free','isGlobal':'is_global',
    'isMassive':'is_massive','isInteractive':'is_interactive','isCloaked':'is_cloaked','isMoribund':'is_moribund',
    'harmonic':'harmonic','corporationID':'corporation_id','allianceID':'alliance_id','Agility':'agility',
    'speedFraction':'speed_fraction','mode':'mode','followId':'follow_id','followRange':'follow_range',
    'ownerId':'owner_id','effectStamp':'effect_stamp','formationID':'formation_id','newBubbleId':'new_bubble','oldBubbleId':'old_bubble',
    'yaw':'yaw','pitch':'pitch','roll':'roll','miniBalls':'mini_balls','miniCapsules':'mini_capsules','miniBoxes':'mini_boxes',
    **{c:'position.'+c for c in 'xyz'},**{'v'+c:'velocity.'+c for c in 'xyz'}}
FIELDS.update({'goto'+c.upper():'goto.'+c for c in 'xyz'})
def equality_expr(n):
    if isinstance(n,ast.BoolOp) and isinstance(n.op,ast.And):return '('+' && '.join(equality_expr(x) for x in n.values)+')'
    if isinstance(n,ast.Compare) and len(n.ops)==1 and isinstance(n.ops[0],ast.Eq) and len(n.comparators)==1:
        return equality_expr(n.left)+'=='+equality_expr(n.comparators[0])
    if isinstance(n,ast.Attribute) and isinstance(n.value,ast.Name) and n.value.id in {'first','second'} and n.attr in FIELDS:
        code=n.value.id+'.'+FIELDS[n.attr]
        return 'f64::from('+code+')' if n.attr in {'radius','maxVelocity','Agility','speedFraction','followRange'} else code
    if isinstance(n,ast.Call) and ast.unparse(n.func)=='list' and len(n.args)==1 and not n.keywords:
        return equality_expr(n.args[0])
    raise ValueError('Unreviewed full native equality helper '+ast.unparse(n))

def preamble():
    tree=ast.parse(original(HELPERS));base=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name=='DestinyTestCase')
    fn=next(n for n in base.body if isinstance(n,ast.FunctionDef) and n.name=='ballsAreEqual')
    if len(fn.body)!=2 or not isinstance(fn.body[0],ast.If) or not isinstance(fn.body[1],ast.Return):raise ValueError('Helper control flow changed')
    expected=ast.parse('if not ignore_bubble_id and (first.newBubbleId!=second.newBubbleId or first.oldBubbleId!=second.oldBubbleId):\n return False\n').body[0]
    if ast.dump(fn.body[0])!=ast.dump(expected):raise ValueError('Bubble-ignore helper guard changed')
    expression=equality_expr(fn.body[1].value)
    return sg.PREAMBLE+'''\nuse destiny_original_spec::OriginalServerParkActions;
fn balls_are_equal_ignoring_bubbles(first:destiny_original_spec::OriginalParkBall,second:destiny_original_spec::OriginalParkBall)->bool {
    // Every original conjunct is preserved, including its maxVelocity/radius
    // and speedFraction/mass comparisons. Native object list equality remains
    // an external correspondence requirement; these scenarios have no children.
    '''+expression+'''
}
fn assert_ball_added_to_client_parks(actions:&mut OriginalServerParkActions,ball:destiny_original_spec::OriginalParkBall,timestamp:i64) {
    let history=actions.queue.flush_history();let mut i=0;
    while i<history.len() {
        if let Some(record)=history.get(i) {
            if record.ball_id==V::Integer(ball.id) && record.timestamp==timestamp && record.action.name==ActionName::AddBallsToPark {
                assert!(record.action.parameters.len()+usize::from(record.action.stream.is_some())==1);
                let stream=record.action.stream;assert!(stream.is_some());
                let mut reader=OriginalParkProjection::default();
                assert!(reader.read_original_state(&stream.unwrap_or_default(),0)==Ok(()));
                assert!(reader.contains(ball.id)); // original dictionary index must not raise
                let loaded=reader.retained(ball.id);
                assert!(ball.mini_balls.is_empty() && ball.mini_capsules.is_empty() && ball.mini_boxes.is_empty());
                if balls_are_equal_ignoring_bubbles(ball,loaded) { return; }
            }
        }i+=1;
    }
    assert!(!true); // original assertFalse(True, "Ball not added ...")
}
'''

def translate(case):
    source,fixture,test=case.split('::')
    allowed={'test_add_ball_to_client_parks','test_make_ball_local','test_make_ball_global_makes_ball_global',
        'test_make_ball_global_adds_ball_to_client_parks','test_make_ball_global_sends_out_signal_with_src_id',
        'test_remove_ball_removes_ball_right_away','test_remove_ball_does_not_note_removal_of_non_global_ball',
        'test_remove_ball_notifies_removal_of_global_ball','test_clean_up_followers'}
    if source!=SOURCE or fixture!='TestActions' or test not in allowed:raise ValueError('Unreviewed native action scenario')
    text=original(source);cls=next(n for n in ast.parse(text).body if isinstance(n,ast.ClassDef) and n.name==fixture)
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test)
    if fn.decorator_list:raise ValueError('Unreviewed decorators')
    helper=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='assertBallAddedToClientParks')
    expected=ast.parse('def assertBallAddedToClientParks(self,ball,timestamp):\n history=self._actions.flush_history()\n for entry in history:\n  ball_id,entry_timestamp,action=entry\n  if ball_id!=ball.id:\n   continue\n  if entry_timestamp!=timestamp:\n   continue\n  actionName,parameters=action\n  if actionName!="AddBallsToPark":\n   continue\n  self.assertEqual(1,len(parameters))\n  bytes=parameters[0]\n  park=self.read_balls_from_stream_into_park(bytes)\n  loaded_ball=park.balls[ball.id]\n  if self.ballsAreEqual(ball,loaded_ball,ignore_bubble_id=True):\n   return\n self.assertFalse(True,"Ball not added to park (or was added at the wrong time).")\n').body[0]
    if ast.dump(helper)!=ast.dump(expected):raise ValueError('Complete original add-ball helper changed')
    t=sg.Translator(sg.methods(original(sg.MODEL_SOURCE)));lines=[]
    for node in fn.body:
        if isinstance(node,ast.Assign) and len(node.targets)==1 and isinstance(node.targets[0],ast.Name) and isinstance(node.value,ast.Call) and ast.unparse(node.value.func)=='add_ball_to_park':
            created=t.native_creation(node.targets[0].id,node.value)
            lines += [x.replace('actions.dynamical_orientation','actions.queue.dynamical_orientation') for x in created]
            lines.append(f'assert!(park.insert_new_query_ball({node.targets[0].id})==Ok(()));');continue
        if isinstance(node,ast.Assign) and len(node.targets)==1 and ast.unparse(node.targets[0])=='callback_parameters' and isinstance(node.value,ast.List) and not node.value.elts:
            lines.append('assert!(actions.made_global_callback_parameters.is_empty()); // original callback list initialized empty');continue
        if isinstance(node,ast.FunctionDef) and node.name=='callback':
            expected=ast.parse('def callback(src_id):\n callback_parameters.append(src_id)\n').body[0]
            if ast.dump(node)!=ast.dump(expected):raise ValueError('Callback helper changed')
            continue
        if isinstance(node,ast.Assign) and len(node.targets)==1 and ast.unparse(node.targets[0])=='expected_action':
            code,kind=t.expression(node.value)
            if kind!='action':raise ValueError('Full expected action required')
            lines.append('let expected_action='+code+';');t.kinds['expected_action']='action';continue
        if isinstance(node,ast.Assign) and len(node.targets)==1 and ast.unparse(node.targets[0])=='remote' and ast.unparse(node.value)=='self._park.GetRemoteFollowers()':
            lines+=['let remote=park.remote_followers();assert!(remote.is_ok());','let remote=remote.unwrap_or_default();'];continue
        if not isinstance(node,ast.Expr) or not isinstance(node.value,ast.Call):raise ValueError('Unreviewed native action statement '+ast.unparse(node))
        c=node.value;method=ast.unparse(c.func)
        if method=='self._actions.on_ball_made_global.connect' and [ast.unparse(x) for x in c.args]==['callback'] and not c.keywords:
            lines.append('actions.made_global_connected=true;');continue
        if method.startswith('self._actions.'):
            name=c.func.attr
            if name=='clean_up_followers' and not c.args and not c.keywords:lines.append('assert!(actions.clean_up_followers(&park)==Ok(()));');continue
            if name not in {'add_ball_to_client_parks','make_ball_local','make_ball_global','remove_ball'} or len(c.args)!=1 or c.keywords:raise ValueError('Unreviewed native action call')
            id=t.scalar(c.args[0],'i64');borrow='&park' if name=='add_ball_to_client_parks' else '&mut park'
            lines.append(f'assert!(actions.{name}({borrow},{id})==Ok(()));');continue
        if method=='self._park.FollowBall' and len(c.args)==3 and not c.keywords:
            lines.append(f'park.follow({t.scalar(c.args[0],"i64")},{t.scalar(c.args[1],"i64")},{t.scalar(c.args[2],"f32")},false);');continue
        if method=='self._park.SetBallPosition' and len(c.args)==4 and not c.keywords:
            lines.append(f'park.set_position({t.scalar(c.args[0],"i64")},Vec3{{'+', '.join(axis+':'+t.scalar(value,'f64') for axis,value in zip('xyz',c.args[1:]))+'});');continue
        if method.startswith('self.assert'):
            lines.append(f'// original-assertion: {source}:{c.lineno}')
            if method=='self.assertBallAddedToClientParks' and len(c.args)==2 and ast.unparse(c.args[1])=='INITIAL_TIMESTAMP' and not c.keywords:
                lines.append(f'assert_ball_added_to_client_parks(&mut actions,park.retained({t.scalar(ast.Attribute(value=c.args[0],attr="id"),"i64")}),INITIAL_TIMESTAMP);');continue
            if method=='self.assertTrue' and len(c.args)==1 and ast.unparse(c.args[0])=='ball.isGlobal':lines.append('assert!(park.retained(ball).is_global);');continue
            if method=='self.assertNotIn' and [ast.unparse(x) for x in c.args]==['ball.id','self._park.balls'] and not c.keywords:lines.append('assert!(!park.contains(ball));');continue
            if method=='self.assertListEqual' and [ast.unparse(x) for x in c.args]==['[ball.id]','callback_parameters'] and not c.keywords:
                lines.append('assert!(actions.made_global_callback_parameters.len()==1 && actions.made_global_callback_parameters.get(0)==Some(ball));');continue
            if method=='self.assertEqual' and len(c.args)==2 and ast.literal_eval(c.args[0])==1 and ast.unparse(c.args[1])=='len(remote)' and not c.keywords:lines.append('assert!(remote.len()==1);');continue
            assertion=t.assertion(c).replace('actions.flush_history()','actions.queue.flush_history()')
            lines.append(assertion);continue
        raise ValueError('Unreviewed native action call '+method)
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+f'\n#[cfg_attr(kani,kani::proof)]\n#[cfg_attr(kani,kani::unwind(129))]\n#[cfg_attr(test,test)]\nfn {test}() {{\n    let mut park=OriginalParkProjection::default();park.is_master=true;\n    let mut actions=OriginalServerParkActions::default();\n'+'\n'.join('    '+x for x in lines)+'\n    park.clear_all();actions.queue.clear_park();actions.queue.reset_settings_to_default();\n    assert!(!park.scope_error);\n    #[cfg(kani)] kani::cover!(true,"complete original native server action and helper assertions completed");\n}\n'

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN work packet required')
    proof=translate(args.case);header=preamble();old=TARGET.read_text() if TARGET.exists() else header
    if '// original-case: '+args.case+'\n' in old:raise ValueError('Case already translated')
    first=old.find('\n// original-case: ')
    if first>=0:old=header+old[first:]
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
