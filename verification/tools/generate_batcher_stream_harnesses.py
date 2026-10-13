#!/usr/bin/env python3
"""Full-state batcher original with complete decoded-ball assertion helper.

Actual bytes use the reviewed old-style codec; Python/Blue/lifetime and native
dispatch remain separate requirements. Other packaged cases fail closed here.
"""
import argparse,ast,hashlib,importlib.util,json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
for name,path in [('delivery','generate_batcher_delivery_harnesses.py'),('native','generate_server_park_action_harnesses.py')]:
    spec=importlib.util.spec_from_file_location(name,ROOT/'verification/tools'/path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);globals()[name]=module
SOURCE=delivery.SOURCE;TARGET=ROOT/'verification/kani/src/batcher_stream_contracts.rs'

def header():
    helpers=ast.parse(delivery.original('python/destiny/test/net/server/helpers.py'))
    cls=next(n for n in helpers.body if isinstance(n,ast.ClassDef) and n.name=='DestinyTestCase')
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='assertBallsEqual')
    checks=[]
    for n in fn.body:
        if isinstance(n,ast.If):
            if ast.unparse(n.test)!='not ignore_bubble_id' or n.orelse:raise ValueError('Bubble helper conditional changed')
            # Source callers pass ignore_bubble_id=True. Retain the exact guard
            # and checked caller argument; these two checks are excluded only
            # by that original branch, not because the model mismatches.
            continue
        if not isinstance(n,ast.Expr) or not isinstance(n.value,ast.Call) or ast.unparse(n.value.func) not in {'self.assertEqual','self.assertListEqual'} or len(n.value.args)!=2 or n.value.keywords:raise ValueError('Unreviewed complete ball assertion helper')
        checks += [f'// original-assertion: python/destiny/test/net/server/helpers.py:{n.lineno}',
            f'assert!({native.equality_expr(n.value.args[0])}=={native.equality_expr(n.value.args[1])});']
    return delivery.PREAMBLE+'''\nfn assert_balls_equal_without_bubbles(first:destiny_original_spec::OriginalParkBall,second:destiny_original_spec::OriginalParkBall) {
'''+''.join('    '+x+'\n' for x in checks)+'''}
fn assert_packaged_bubble_action(park:&OriginalParkProjection,batcher:&OriginalUpdateBatcher,event:destiny_original_spec::BatcherEvent,ball:i64,additions:bool) {
    let package=match event.payload { BatcherPayload::Packaged{package_index}=>Some(package_index),_=>None };
    assert!(package.is_some()); // original marshal.Load argument must be bytes
    let bytes=batcher.package_bytes(package.unwrap_or(0));assert!(bytes.is_some());
    let unpacked=destiny_original_spec::unmarshal_original_event(&bytes.unwrap_or_default());
    assert!(unpacked.is_ok()); // original native marshal.Load must not raise
    if let Ok(unpacked)=unpacked {
        assert!(unpacked.action_count==1); // original len(unpackaged_action_list)
        assert!(unpacked.timestamp==park.current_time);
        assert!(unpacked.parameter_count==1); // original len(action_parameters)
        if additions {
            let stream=match unpacked.action { destiny_original_spec::OriginalMarshalledAction::AddBalls{stream}=>Some(stream),_=>None };
            assert!(stream.is_some()); // original AddBallsToPark name and one byte parameter
            let mut reader=OriginalParkProjection::default();
            assert!(reader.read_original_state(&stream.unwrap_or_default(),0)==Ok(()));
            assert!(reader.contains(ball)); // original dictionary index must not raise
            assert_balls_equal_without_bubbles(park.retained(ball),reader.retained(ball));
        } else {
            let ids=match unpacked.action { destiny_original_spec::OriginalMarshalledAction::RemoveBalls{ids}=>Some(ids),_=>None };
            assert!(ids.is_some()); // original RemoveBalls name and one list parameter
            if let Some(ids)=ids { assert!(ids.len()==1 && ids.get(0)==Some(ball)); }
        }
    }
}
fn count_set_state_actions(park:&OriginalParkProjection,batcher:&OriginalUpdateBatcher,history:BatcherHistory,ball:i64)->usize {
    let mut count=0;let mut i=0;
    while i<history.len() {
        let event=history.get(i);assert!(event.is_some());
        if let Some(event)=event {
            if event.name==ActionName::SetState {
                let payload=match event.payload { BatcherPayload::Stream{stream_index,reference}=>Some((stream_index,reference)),_=>None };
                assert!(payload.is_some()); // original parameter[0] bytes and source argument exist
                if let Some((index,_))=payload {
                    let bytes=batcher.stream_bytes(index);assert!(bytes.is_some());
                    let mut reader=OriginalParkProjection::default();
                    assert!(reader.read_original_state(&bytes.unwrap_or_default(),0)==Ok(()));
                    assert!(reader.contains(ball)); // original helper assertIn
                    assert_balls_equal_without_bubbles(park.retained(ball),reader.retained(ball));
                    count+=1;
                }
            }
        }i+=1;
    }count
}
'''

def translate(case):
    if case.endswith('::test_update_bubbles_inserts_add_balls_actions_to_character_history') or case.endswith('::test_update_bubbles_inserts_remove_balls_actions_to_character_history'):
        return translate_bubble(case)
    expected_case=SOURCE+'::TestUpdateBatcher::test_send_full_state_update'
    if case!=expected_case:raise ValueError('Packaged action codec dependency not yet implemented')
    text=delivery.original(SOURCE);cls=next(n for n in ast.parse(text).body if isinstance(n,ast.ClassDef) and n.name=='TestUpdateBatcher')
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='test_send_full_state_update')
    expected=ast.parse('def test_send_full_state_update(self):\n self._set_up_ball(BALL_ID_1,CHAR_ID_1,CLIENT_ID_1)\n self._update_batcher.send_full_state_update(CHAR_ID_1,BALL_ID_1)\n character_history=self._update_batcher.get_character_history(CHAR_ID_1)\n count=self._count_matching_set_state_actions(character_history,BALL_ID_1)\n self.assertEqual(1,count,"Could not find set state action including ball %s"%BALL_ID_1)\n').body[0]
    if ast.dump(fn)!=ast.dump(expected):raise ValueError('Complete full-state scenario changed')
    helper=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='_count_matching_set_state_actions')
    expected=ast.parse('def _count_matching_set_state_actions(self,actions,ball_id):\n count=0\n for timestamp,action in actions:\n  action_name,action_parameters=action\n  if action_name=="SetState":\n   ball_data=action_parameters[0]\n   park=self.read_balls_from_stream_into_park(ball_data)\n   self.assertIn(ball_id,park.balls,"Ball %s not in SetState update stream"%ball_id)\n   loaded_ball=park.balls[ball_id]\n   self.assertBallsEqual(self._park.balls[ball_id],loaded_ball,ignore_bubble_id=True)\n   count+=1\n return count\n').body[0]
    if ast.dump(helper)!=ast.dump(expected):raise ValueError('Complete SetState count/stream helper changed')
    sites=[x.lineno for x in ast.walk(helper) if isinstance(x,ast.Call) and ast.unparse(x.func).startswith('self.assert')]
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+'''\n#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn test_send_full_state_update() {
    let mut park=OriginalParkProjection::default();park.is_master=true;
    {
    let mut batcher=OriginalUpdateBatcher::default();let mut interests=OriginalNetworkInterests::default();
    let _=set_up_ball(&mut park,&mut interests);
    assert!(batcher.send_full_state_update(&park,CHAR_ID_1,BALL_ID_1)==Ok(()));
    let character_history=batcher.get_character_history(CHAR_ID_1);
    // Full original count helper assertion sites: '''+str(sites)+'''
    let count=count_set_state_actions(&park,&batcher,character_history,BALL_ID_1);
    // original-assertion: '''+SOURCE+':'+str(fn.body[-1].lineno)+'''
    assert!(count==1);
    } // source batcher/reference destruction boundary
    park.clear_all();assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original full-state event, complete equality helpers and teardown completed");
}
'''

def translate_bubble(case):
    source,fixture,test=case.split('::');additions=test=='test_update_bubbles_inserts_add_balls_actions_to_character_history'
    if source!=SOURCE or fixture!='TestUpdateBatcher' or test not in {'test_update_bubbles_inserts_add_balls_actions_to_character_history','test_update_bubbles_inserts_remove_balls_actions_to_character_history'}:raise ValueError('Unknown bubble scenario')
    text=delivery.original(source);cls=next(n for n in ast.parse(text).body if isinstance(n,ast.ClassDef) and n.name==fixture)
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test)
    if additions:
        body='self._set_up_ball(BALL_ID_1,CHAR_ID_1,CLIENT_ID_1)\nself._update_batcher.update_bubbles()\nself._park.Evolve()\nself._update_batcher.clear_character_history(CHAR_ID_1)\nadd_ball_to_park(self._park,ball_id=BALL_ID_2)\nself._update_batcher.update_bubbles()\nchar_history=self._update_batcher.get_character_history(CHAR_ID_1)\nself.assertEqual(1,len(char_history))\ntimestamp,action=char_history[0]\nself.assertEqual(self._park.currentTime,timestamp)\nself.assertBallsInPackagedAddBallsAction(action,[BALL_ID_2])\n'
        scenario='''assert!(batcher.update_bubbles(&mut park,&mut updater,&interests)==Ok(()));
    assert!(park.evolve_two_spherical_bodies()==Ok(()));
    batcher.clear_character_history(CHAR_ID_1);
    let second=add_second_ball(&mut park);
    assert!(batcher.update_bubbles(&mut park,&mut updater,&interests)==Ok(()));'''
    else:
        body='self._set_up_ball(BALL_ID_1,CHAR_ID_1,CLIENT_ID_1)\nadd_ball_to_park(self._park,ball_id=BALL_ID_2)\nself._update_batcher.update_bubbles()\nself._update_batcher.clear_character_history(CHAR_ID_1)\nself._park.RemoveBall(BALL_ID_2)\nself._park.Evolve()\nself._update_batcher.update_bubbles()\nchar_history=self._update_batcher.get_character_history(CHAR_ID_1)\nself.assertEqual(1,len(char_history))\ntimestamp,action=char_history[0]\nself.assertEqual(self._park.currentTime,timestamp)\nself.assertPackagedRemoveBallsActionRemovesBalls(action,[BALL_ID_2])\n'
        scenario='''let second=add_second_ball(&mut park);
    assert!(batcher.update_bubbles(&mut park,&mut updater,&interests)==Ok(()));
    batcher.clear_character_history(CHAR_ID_1);
    park.remove_ball(second,0);
    assert!(park.evolve_two_spherical_bodies()==Ok(()));
    assert!(batcher.update_bubbles(&mut park,&mut updater,&interests)==Ok(()));'''
    expected=ast.parse('def '+test+'(self):\n'+''.join(' '+line+'\n' for line in body.splitlines())).body[0]
    if ast.dump(fn)!=ast.dump(expected):raise ValueError('Complete bubble original changed')
    helper_name='assertBallsInPackagedAddBallsAction' if additions else 'assertPackagedRemoveBallsActionRemovesBalls'
    h=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==helper_name)
    terminal='self.assertBallsInAddBallsAction(add_balls_action,ball_list)' if additions else 'self.assertRemoveBallsActionRemovesBalls(add_balls_action,ball_list)'
    expected=ast.parse('def '+helper_name+'(self,packaged_add_balls_action,ball_list):\n action_name,action_parameters=packaged_add_balls_action\n unpackaged_action_list=blue.marshal.Load(action_parameters)\n self.assertEqual(1,len(unpackaged_action_list))\n timestamp,add_balls_action=unpackaged_action_list[0]\n self.assertEqual(self._park.currentTime,timestamp)\n '+terminal+'\n').body[0]
    if ast.dump(h)!=ast.dump(expected):raise ValueError('Full packaged assertion helper changed')
    inner='assertBallsInAddBallsAction' if additions else 'assertRemoveBallsActionRemovesBalls'
    ih=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==inner)
    if additions:
        expected=ast.parse('def assertBallsInAddBallsAction(self,add_balls_action,ball_list):\n action_name,action_parameters=add_balls_action\n self.assertEqual("AddBallsToPark",action_name)\n self.assertEqual(1,len(action_parameters))\n ball_stream,=action_parameters\n park=self.read_balls_from_stream_into_park(ball_stream)\n for ball_id in ball_list:\n  self.assertBallsEqual(self._park.balls[ball_id],park.balls[ball_id],ignore_bubble_id=True)\n').body[0]
    else:
        expected=ast.parse('def assertRemoveBallsActionRemovesBalls(self,remove_balls_action,ball_list):\n action_name,action_parameters=remove_balls_action\n self.assertEqual("RemoveBalls",action_name)\n self.assertEqual(1,len(action_parameters))\n ball_id_list,=action_parameters\n self.assertListEqual(ball_id_list,ball_list)\n').body[0]
    if ast.dump(ih)!=ast.dump(expected):raise ValueError('Full unpacked assertion helper changed')
    sites=[n.lineno for n in ast.walk(fn) if isinstance(n,ast.Call) and ast.unparse(n.func).startswith('self.assert')]
    helper_sites=[n.lineno for parent in (h,ih) for n in ast.walk(parent) if isinstance(n,ast.Call) and ast.unparse(n.func).startswith('self.assert')]
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+f'''\n#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(1025))]
#[cfg_attr(test,test)]
fn {test}() {{
    let mut park=OriginalParkProjection::default();park.is_master=true;
    {{
    let mut batcher=OriginalUpdateBatcher::default();let mut interests=OriginalNetworkInterests::default();
    let mut updater=destiny_original_spec::OriginalBubbleUpdater::default();
    let _=set_up_ball(&mut park,&mut interests);
    {scenario}
    let char_history=batcher.get_character_history(CHAR_ID_1);
    // original-assertion sites: {source}:{sites}; complete helper sites {helper_sites}
    assert!(char_history.len()==1);
    let action=char_history.get(0);assert!(action.is_some());
    if let Some(action)=action {{
        assert!(park.current_time==action.timestamp);
        assert_packaged_bubble_action(&park,&batcher,action,second,{str(additions).lower()});
    }}
    }} // original updater/batcher/reference destruction boundary
    park.clear_all();assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original packaged bubble action and all nested helper assertions completed");
}}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN packet required')
    extra='''\nfn add_second_ball(park:&mut OriginalParkProjection)->i64 {
    let id=park.add_ball(destiny_original_spec::OriginalBallInit{id:1002,mass:1.0,radius:1.0,max_velocity:1.0,
        is_free:true,is_global:false,is_massive:true,is_interactive:true,is_space_junk:false,
        position:Vec3::ZERO,velocity:Vec3::ZERO,agility:1.0,speed_fraction:1.0});
    assert!(park.contains(id) && !park.scope_error);assert!(park.insert_new_query_ball(id)==Ok(()));id
}
'''
    proof=translate(args.case);prelude=header()+extra;old=TARGET.read_text() if TARGET.exists() else prelude
    if '// original-case: '+args.case+'\n' in old:raise ValueError('Case already translated')
    first=old.find('\n// original-case: ')
    if first>=0:old=prelude+old[first:]
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
