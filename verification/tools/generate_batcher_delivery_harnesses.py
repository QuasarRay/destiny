#!/usr/bin/env python3
"""Translate the reviewed original batcher recipient/count scenarios.

Every count-helper assertion and each fixture operation are retained. Python
set ordering, connected signals, native interfaces and dispatch are residuals.
"""
import argparse,ast,hashlib,json,importlib.util
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2]
REV='ef738406b61ad06bae2f402857e464735ab106a2'
SOURCE='python/destiny/test/net/server/test_updatebatcher.py'
TARGET=ROOT/'verification/kani/src/batcher_delivery_contracts.rs'

def original(path):
    return subprocess.run(['git','show',REV+':'+path],cwd=ROOT,capture_output=True,text=True,check=True).stdout

spec=importlib.util.spec_from_file_location('batcher_generator',ROOT/'verification/tools/generate_batcher_harnesses.py')
bg=importlib.util.module_from_spec(spec);spec.loader.exec_module(bg)

PREAMBLE=bg.PREAMBLE+'''
use destiny_original_spec::{OriginalBallInit,OriginalNetworkInterests,OriginalNetworkOutput,
    OriginalNetworkCast,Vec3};
const BALL_ID_1:i64=1001;const CLIENT_ID_1:i64=13579;
fn set_up_ball(park:&mut OriginalParkProjection,interests:&mut OriginalNetworkInterests)->i64 {
    let id=park.add_ball(OriginalBallInit{id:BALL_ID_1,mass:1.0,radius:1.0,max_velocity:1.0,
        is_free:true,is_global:false,is_massive:true,is_interactive:true,is_space_junk:false,
        position:Vec3::ZERO,velocity:Vec3::ZERO,agility:1.0,speed_fraction:1.0});
    assert!(park.contains(id) && !park.scope_error);
    assert!(park.insert_new_query_ball(id)==Ok(()));
    assert!(interests.add_client_interest(id,CLIENT_ID_1).is_ok());
    assert!(interests.add_character_interest(id,CHAR_ID_1).is_ok());
    assert!(interests.map_to_client(CHAR_ID_1,Some(CLIENT_ID_1)).is_ok());
    assert!(interests.set_ball_info_characters(id,&[CHAR_ID_1]).is_ok());
    id
}
fn ids_contain(ids:&BoundedList<i64,8>,id:i64)->bool {
    let mut i=0;while i<ids.len() { if ids.get(i)==Some(id) { return true; }i+=1; }false
}
fn assert_cast_count(casts:&BoundedList<OriginalNetworkCast,8>,client:i64,expected:u8,single:bool) {
    let mut found=false;let mut i=0;
    while i<casts.len() {
        if let Some(cast)=casts.get(i) {
            let recipient=if single { cast.clients.get(0)==Some(client) } else { ids_contain(&cast.clients,client) };
            // The source send_batch uses DoDestinyUpdate for every cast.
            if recipient {
                assert!(expected==cast.casts_in_batch); // original helper assertEqual
                found=true;
            }
        }
        i+=1;
    }
    assert!(found); // original helper assertTrue(found)
}
'''

def translate(case):
    source,fixture,test=case.split('::')
    if source!=SOURCE or fixture!='TestUpdateBatcher':raise ValueError('Unreviewed delivery fixture')
    text=original(source);tree=ast.parse(text);cls=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name==fixture)
    helper=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='_set_up_ball')
    expected=ast.parse('def _set_up_ball(self,ball_id,char_id,client_id):\n ball=add_ball_to_park(self._park,ball_id=ball_id)\n self._client_interests.add_client_interest(ball_id,client_id)\n self._character_interests.add_interest(ball_id,char_id)\n self._character_interests.map_to_client(char_id,client_id)\n self._ball_info.set_characters_for_ball(ball_id,[char_id])\n return ball\n').body[0]
    if ast.dump(helper)!=ast.dump(expected):raise ValueError('Reviewed interests fixture helper changed')
    for name in ('asssertCastsInBatchSinglecast','asssertCastsInBatchNarrowcast'):
        h=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==name)
        if name.endswith('Singlecast'):
            expected=ast.parse('def asssertCastsInBatchSinglecast(self,client_id,expected,cast_list):\n found=False\n for cast in cast_list:\n  cast_client_id,name,_,_,casts_in_batch=cast\n  if name=="DoDestinyUpdate" and cast_client_id==client_id:\n   self.assertEqual(expected,casts_in_batch)\n   found=True\n self.assertTrue(found)\n').body[0]
        else:
            expected=ast.parse('def asssertCastsInBatchNarrowcast(self,client_id,expected,cast_list):\n found=False\n for cast in cast_list:\n  cast_client_id_list,name,_,_,casts_in_batch=cast\n  if name=="DoDestinyUpdate" and client_id in cast_client_id_list:\n   self.assertEqual(expected,casts_in_batch)\n   found=True\n self.assertTrue(found)\n').body[0]
        if ast.dump(h)!=ast.dump(expected):raise ValueError('Reviewed full cast helper changed')
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test)
    if fn.decorator_list:raise ValueError('Unreviewed decorators')
    supported={'test_character_history_only_has_cast_count_one','test_bubble_history_only_has_cast_count_one',
        'test_character_and_bubble_history_has_cast_count_two','test_clients_with_character_history_are_returned_as_a_set'}
    if test not in supported:raise ValueError('Delivery dependency not yet implemented')
    lines=[];has_ball=False
    for n in fn.body:
        call=n.value if isinstance(n,(ast.Expr,ast.Assign)) else None
        if not isinstance(call,ast.Call):raise ValueError('Unreviewed delivery statement '+ast.unparse(n))
        method=ast.unparse(call.func)
        if method=='self._set_up_ball':
            if [ast.unparse(x) for x in call.args]!=['BALL_ID_1','CHAR_ID_1','CLIENT_ID_1'] or call.keywords:raise ValueError('Unreviewed recipient fixture')
            if isinstance(n,ast.Assign) and [ast.unparse(x) for x in n.targets]!=['ball']:raise ValueError('Unreviewed setup result binding')
            lines.append('let ball=set_up_ball(&mut park,&mut interests);');has_ball=True
        elif method in {'self._update_batcher.add_to_character_history','self._update_batcher.add_to_bubble_history'}:
            if len(call.args)!=2 or call.keywords or ast.unparse(call.args[1])!='GOTO_ACTION':raise ValueError('Unreviewed queued delivery event')
            if method.endswith('add_to_character_history') and ast.unparse(call.args[0])=='CHAR_ID_1':
                lines.append('assert!(batcher.add_to_character_history(CHAR_ID_1,goto_action(),false)==Ok(()));')
            elif method.endswith('add_to_bubble_history') and ast.unparse(call.args[0])=='ball.newBubbleId':
                lines.append('assert!(batcher.add_to_bubble_history(park.retained(ball).new_bubble,goto_action())==Ok(()));')
            else:raise ValueError('Unreviewed history recipient')
        elif method=='self._update_batcher.send_batch':
            if call.args or call.keywords:raise ValueError('Unexpected send arguments')
            lines+=['let sent=batcher.send_batch(&park,&interests);','assert!(sent.is_ok());','let network=sent.unwrap_or_default();']
        elif method=='self._update_batcher.get_clients_with_character_history':
            if call.args or call.keywords or not isinstance(n,ast.Assign) or [ast.unparse(x) for x in n.targets]!=['client_ids']:raise ValueError('Getter shape changed')
            lines+=['let clients=batcher.get_clients_with_character_history(&interests);','assert!(clients.is_ok());','let client_ids=clients.unwrap_or_default();']
        elif method=='self.assertEqual' and len(call.args)==2 and not call.keywords:
            expected=ast.literal_eval(call.args[0]);expression=ast.unparse(call.args[1])
            if type(expected)is not int or expression not in {'len(self._network.singlecasts)','len(self._network.narrowcasts)'}:raise ValueError('Unreviewed cast length assertion')
            field='singlecasts' if 'singlecasts' in expression else 'narrowcasts'
            lines += [f'// original-assertion: {source}:{call.lineno}',f'assert!(network.{field}.len()=={expected}_usize);']
        elif method=='self.assertSetEqual':
            if len(call.args)!=2 or call.keywords or ast.unparse(call.args[0])!='{CLIENT_ID_1}' or ast.unparse(call.args[1])!='client_ids':raise ValueError('Unreviewed recipient set assertion')
            lines += [f'// original-assertion: {source}:{call.lineno}','assert!(client_ids.len()==1 && ids_contain(&client_ids,CLIENT_ID_1));']
        elif method in {'self.asssertCastsInBatchSinglecast','self.asssertCastsInBatchNarrowcast'}:
            if len(call.args)!=3 or call.keywords or ast.unparse(call.args[0])!='CLIENT_ID_1':raise ValueError('Unreviewed complete cast helper call')
            count={'ClientUpdateCountThisTick.ONE':1,'ClientUpdateCountThisTick.TWO':2}.get(ast.unparse(call.args[1]));single=method.endswith('Singlecast')
            field='singlecasts' if single else 'narrowcasts'
            if count is None or ast.unparse(call.args[2])!='self._network.'+field:raise ValueError('Original cast shape/count changed')
            h=next(x for x in cls.body if isinstance(x,ast.FunctionDef) and x.name==call.func.attr)
            assertions=[x.lineno for x in ast.walk(h) if isinstance(x,ast.Call) and ast.unparse(x.func).startswith('self.assert')]
            lines += [f'// original-assertion: {source}:{call.lineno}; helper assertion sites {assertions}',f'assert_cast_count(&network.{field},CLIENT_ID_1,{count}_u8,{str(single).lower()});']
        else:raise ValueError('Unreviewed delivery call '+method)
    if not has_ball:raise ValueError('Full recipient fixture required')
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+f'\n#[cfg_attr(kani,kani::proof)]\n#[cfg_attr(kani,kani::unwind(65))]\n#[cfg_attr(test,test)]\nfn {test}() {{\n    let mut park=OriginalParkProjection::default();park.is_master=true;\n    {{\n    let mut batcher=OriginalUpdateBatcher::default();let mut interests=OriginalNetworkInterests::default();\n'+'\n'.join('    '+x for x in lines)+'\n    } // original batcher/reference destruction boundary\n    park.clear_all();assert!(!park.scope_error);\n    #[cfg(kani)] kani::cover!(true,"original batcher recipients/count helpers and teardown completed");\n}\n'

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN case required')
    proof=translate(args.case);old=TARGET.read_text() if TARGET.exists() else PREAMBLE
    if '// original-case: '+args.case+'\n' in old:raise ValueError('Case already translated')
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
