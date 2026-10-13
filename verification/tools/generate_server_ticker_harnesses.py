#!/usr/bin/env python3
"""Translate complete pinned server ticker tests and their assertion helpers.

One exact OPEN case per mutation. Actual queue/park/bubbles/codec/batcher run in
the original order. Native dispatch, signal and Python correspondence stay open.
"""
import argparse,ast,hashlib,importlib.util,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];REV='ef738406b61ad06bae2f402857e464735ab106a2'
SOURCE='python/destiny/test/net/server/test_ticker.py'
TARGET=ROOT/'verification/kani/src/server_ticker_contracts.rs'
for name,file in [('client','generate_client_ticker_harnesses.py'),('ball_helpers','generate_batcher_stream_harnesses.py')]:
    spec=importlib.util.spec_from_file_location(name,ROOT/'verification/tools'/file);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);globals()[name]=module
def original(path):return subprocess.run(['git','show',REV+':'+path],cwd=ROOT,capture_output=True,text=True,check=True).stdout

def review(cls):
    client.exact_method(cls,'setUp','''def setUp(self):
 self._park = destiny.Ballpark()
 self._park.isMaster = True
 self._ball_info = TestBallInfo()
 self._actions = Actions(self._park)
 self._network = TestNetwork()
 self._character_interests = TestCharacterInterests()
 self._client_interests = TestClientInterests()
 self._bubble_updater = BubbleUpdater(self._park)
 self._update_batcher = ParkUpdateBatcher(self._park, self._network, self._character_interests, self._client_interests, self._bubble_updater)
 self._ticker = Ticker(self._park, self._ball_info, self._actions, self._update_batcher)
''')
    client.exact_method(cls,'tearDown','''def tearDown(self):
 del self._ticker
 del self._update_batcher
 del self._bubble_updater
 del self._actions
 del self._park
''')
    client.exact_method(cls,'_set_up_ball','''def _set_up_ball(self, ball_id, char_id, client_id):
 ball = add_ball_to_park(self._park, ball_id=ball_id)
 self._client_interests.add_client_interest(ball_id, client_id)
 self._character_interests.add_interest(ball_id, char_id)
 self._character_interests.map_to_client(char_id, client_id)
 self._ball_info.set_characters_for_ball(ball_id, [char_id])
 return ball
''')
    client.exact_method(cls,'_set_up_single_ball','def _set_up_single_ball(self):\n return self._set_up_ball(BALL_ID_1, CHAR_1, CLIENT_1)\n')
    client.exact_method(cls,'_set_up_two_balls_in_same_bubble','''def _set_up_two_balls_in_same_bubble(self):
 ball_one = self._set_up_ball(BALL_ID_1, CHAR_1, CLIENT_1)
 ball_two = self._set_up_ball(BALL_ID_2, CHAR_2, CLIENT_2)
 return ball_one, ball_two
''')
    client.exact_method(cls,'_set_up_three_balls_in_same_bubble','''def _set_up_three_balls_in_same_bubble(self):
 ball_one, ball_two = self._set_up_two_balls_in_same_bubble()
 ball_three = self._set_up_ball(BALL_ID_3, CHAR_3, CLIENT_3)
 return ball_one, ball_two, ball_three
''')
    # Ignore function docstrings only; compare every executable statement.
    perform=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='_perform_tick')
    executable=perform.body[1:] if isinstance(perform.body[0],ast.Expr) and isinstance(perform.body[0].value,ast.Constant) and isinstance(perform.body[0].value.value,str) else perform.body
    expected=ast.parse('self._ticker.tick()\nif not skip_evolve:\n self._park.Evolve()\nself._update_batcher.send_batch()\n').body
    if ast.dump(perform.args)!=ast.dump(ast.parse('def f(self,skip_evolve=False):\n pass\n').body[0].args) or [ast.dump(x) for x in executable]!=[ast.dump(x) for x in expected]:raise ValueError('Changed full tick helper')
    for suffix in ('Singlecast','Narrowcast'):
        single=suffix=='Singlecast';parameter='client_id' if single else 'client_ids';field='singlecasts' if single else 'narrowcasts'
        comparison='entry_client_id != client_id' if single else 'set(clients) != set(client_ids)'
        unpack='entry_client_id, method_name, actions, wait_for_bubble, casts_in_batch' if single else 'clients, method_name, actions, wait_for_bubble, casts_in_batch'
        client.exact_method(cls,'assertUpdate'+suffix,f'''def assertUpdate{suffix}(self, {parameter}, action, timestamp=None):
 if timestamp is None:
  timestamp = self._ticker.stamp_for_system - 1
 for entry in self._network.{field}:
  {unpack} = entry
  if method_name != "DoDestinyUpdate":
   continue
  if {comparison}:
   continue
  if (timestamp, action) in actions:
   return
 self.assertTrue(False, "Could not find specified action in {field}")
''')
    client.exact_method(cls,'_countMatchingAddBallsToParkActions','''def _countMatchingAddBallsToParkActions(self, actions, ball_id):
 count = 0
 for timestamp, action in actions:
  action_name, action_parameters = action
  if action_name == "AddBallsToPark":
   ball_data = action_parameters[0]
   park = self.read_balls_from_stream_into_park(ball_data)
   self.assertIn(ball_id, park.balls, "Ball %s not in AddBallsToPark update stream" % ball_id)
   loaded_ball = park.balls[ball_id]
   self.assertBallsEqual(self._park.balls[ball_id], loaded_ball, ignore_bubble_id=True)
   count += 1
 return count
''')
    for suffix in ('Singlecast','Narrowcast'):
        single=suffix=='Singlecast';parameter='client_id' if single else 'client_ids';field='singlecasts' if single else 'narrowcasts'
        comparison='entry_client_id != client_id' if single else 'set(entry_client_id) != set(client_ids)'
        client.exact_method(cls,'assertAddBallsToPark'+suffix+'ExistsForBall',f'''def assertAddBallsToPark{suffix}ExistsForBall(self, ball_id, {parameter}):
 found = 0
 for entry in self._network.{field}:
  entry_client_id, method_name, actions, wait_for_bubble, casts_in_batch = entry
  if method_name != "DoDestinyUpdate":
   continue
  if {comparison}:
   continue
  found += self._countMatchingAddBallsToParkActions(actions, ball_id)
 self.assertEqual(1, found)
''')

def header():
    full=ball_helpers.header();start=full.index('fn assert_balls_equal_without_bubbles(');end=full.index('fn assert_packaged_bubble_action(',start)
    return '''// Copyright © 2023 CCP ehf. Original tests' MIT license retained.
// Generated from pinned Git blobs by generate_server_ticker_harnesses.py.
// Complete fixture/action ordering, real evolution, bubbles, bytes and casts.
// Native parent lookup, Blue/Python aliases and signal lifetime remain open.
#![allow(non_snake_case)]
use destiny_original_spec::{ActionName,ActionValue as V,BatcherEvent,BatcherHistory,
    BatcherPayload,BoundedList,OriginalBallInit,OriginalMode,OriginalNetworkCast,
    OriginalParkProjection,OriginalServerTicker,ServerTickerSignal,Vec3};

const BALL_ID_1:i64=1001;const BALL_ID_2:i64=1002;const BALL_ID_3:i64=1003;
const CHAR_1:i64=90000128;const CHAR_2:i64=90000256;const CHAR_3:i64=90000512;
const CLIENT_1:i64=13579;const CLIENT_2:i64=24680;const CLIENT_3:i64=80808;
fn ids(values:&[i64])->BoundedList<i64,8> {
    let mut result=BoundedList::new();let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); }i+=1; }result
}
fn args(values:&[V])->BoundedList<V,8> {
    let mut result=BoundedList::new();let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); }i+=1; }result
}
fn ids_equal(left:&BoundedList<i64,8>,right:&BoundedList<i64,8>)->bool {
    let mut i=0;while i<left.len() { let mut found=false;let mut j=0;
        while j<right.len() { found|=left.get(i)==right.get(j);j+=1; }if !found { return false; }i+=1; }
    i=0;while i<right.len() { let mut found=false;let mut j=0;
        while j<left.len() { found|=right.get(i)==left.get(j);j+=1; }if !found { return false; }i+=1; }true
}
fn add_ball(park:&mut OriginalParkProjection,id:i64)->i64 {
    assert!(!park.dynamical_orientation); // actual helper selects old-style AddBall
    let replacing=park.contains(id);
    let ball=park.add_ball(OriginalBallInit{id,mass:1.0,radius:1.0,max_velocity:1.0,
        is_free:true,is_global:false,is_massive:true,is_interactive:true,is_space_junk:false,
        position:Vec3::ZERO,velocity:Vec3::ZERO,agility:1.0,speed_fraction:1.0});
    assert!(park.contains(ball) && !park.scope_error);
    if !replacing { assert!(park.insert_new_query_ball(ball)==Ok(())); }
    ball
}
fn set_up_ball(park:&mut OriginalParkProjection,ticker:&mut OriginalServerTicker,id:i64,character:i64,client:i64)->i64 {
    let ball=add_ball(park,id);
    assert!(ticker.interests.add_client_interest(id,client).is_ok());
    assert!(ticker.interests.add_character_interest(id,character).is_ok());
    assert!(ticker.interests.map_to_client(character,Some(client)).is_ok());
    assert!(ticker.interests.set_ball_info_characters(id,&[character]).is_ok());
    ball
}
fn perform_tick(park:&mut OriginalParkProjection,ticker:&mut OriginalServerTicker,skip_evolve:bool) {
    assert!(ticker.tick(park)==Ok(()));
    if !skip_evolve { assert!(park.evolve_two_spherical_bodies()==Ok(())); }
    assert!(ticker.send_batch(park)==Ok(()));
}
fn assert_update(ticker:&OriginalServerTicker,clients:BoundedList<i64,8>,event:BatcherEvent,single:bool) {
    let casts=if single { &ticker.network.singlecasts } else { &ticker.network.narrowcasts };let mut i=0;
    while i<casts.len() {
        if let Some(cast)=casts.get(i) {
            // Every actual source batcher dispatch uses DoDestinyUpdate.
            let selected=if single { cast.clients.get(0)==clients.get(0) } else { ids_equal(&cast.clients,&clients) };
            if selected { let mut j=0;while j<cast.history.len() { if cast.history.get(j)==Some(event) { return; }j+=1; } }
        }i+=1;
    }
    assert!(false); // original full helper assertTrue(False,...)
}
'''+full[start:end]+'''
fn count_add_ball_actions(park:&OriginalParkProjection,ticker:&OriginalServerTicker,history:BatcherHistory,id:i64)->usize {
    let mut count=0;let mut i=0;
    while i<history.len() {
        if let Some(event)=history.get(i) { if event.name==ActionName::AddBallsToPark {
            let index=match event.payload { BatcherPayload::Stream{stream_index,..}=>Some(stream_index),_=>None };
            assert!(index.is_some()); // original action_parameters[0] must be bytes
            let bytes=ticker.batcher.stream_bytes(index.unwrap_or(usize::MAX));assert!(bytes.is_some());
            let mut reader=OriginalParkProjection::default();
            assert!(reader.read_original_state(&bytes.unwrap_or_default(),0)==Ok(()));
            assert!(reader.contains(id)); // original helper assertIn
            assert!(park.contains(id)); // original dictionary index must exist
            assert_balls_equal_without_bubbles(park.retained(id),reader.retained(id));count+=1;
        } }i+=1;
    }count
}
fn assert_add_ball_cast(park:&OriginalParkProjection,ticker:&OriginalServerTicker,id:i64,clients:BoundedList<i64,8>,single:bool) {
    let casts=if single { &ticker.network.singlecasts } else { &ticker.network.narrowcasts };
    let mut found=0;let mut i=0;
    while i<casts.len() { if let Some(cast)=casts.get(i) {
        let selected=if single { cast.clients.get(0)==clients.get(0) } else { ids_equal(&cast.clients,&clients) };
        if selected { found+=count_add_ball_actions(park,ticker,cast.history,id); }
    }i+=1; }
    assert!(found==1); // original assertEqual(1, found)
}
'''

class Translator:
    def __init__(self):self.variables={};self.balls=set();self.callback=None
    def primitive(self,n):
        if isinstance(n,ast.Constant) and type(n.value) in (int,float,bool):return str(n.value).lower() if isinstance(n.value,bool) else repr(n.value)+('_i64' if isinstance(n.value,int) else '_f64')
        if isinstance(n,ast.Name) and n.id in self.variables:return n.id
        if isinstance(n,ast.Name) and n.id in {'BALL_ID_1','BALL_ID_2','BALL_ID_3','CLIENT_1','CLIENT_2','CLIENT_3','CHAR_1','CHAR_2','CHAR_3'}:return n.id
        if ast.unparse(n)=='destiny.DSTNORMALCLOAK':return '1_i64'
        if ast.unparse(n)=='destiny.DSTBALL_GOTO':return 'OriginalMode::Goto'
        if isinstance(n,ast.Attribute) and isinstance(n.value,ast.Name) and n.value.id in self.balls:
            field={'mode':'mode','isCloaked':'is_cloaked',**{c:'position.'+c for c in 'xyz'}}.get(n.attr)
            if field:
                value='park.retained('+n.value.id+').'+field
                return 'i64::from('+value+')' if n.attr=='isCloaked' else value
        if isinstance(n,ast.Tuple):return '('+','.join(self.primitive(x) for x in n.elts)+')'
        raise ValueError('Unreviewed primitive '+ast.unparse(n))
    def value(self,n):
        if isinstance(n,ast.Name) and n.id in self.variables:kind=self.variables[n.id]
        elif isinstance(n,ast.Constant):kind={int:'Integer',float:'Float',bool:'Boolean'}.get(type(n.value))
        elif ast.unparse(n)=='destiny.DSTNORMALCLOAK' or isinstance(n,ast.Name) and n.id.startswith(('BALL_ID_','CLIENT_','CHAR_')):kind='Integer'
        else:kind=None
        if kind is None:raise ValueError('Unreviewed typed action value '+ast.unparse(n))
        return 'V::'+kind+'('+self.primitive(n)+')'
    def client_ids(self,n,single=False):
        values=[n] if single else n.elts if isinstance(n,ast.List) else None
        if values is None:raise ValueError('Unreviewed recipients')
        return 'ids(&['+','.join(self.primitive(x) for x in values)+'])'
    def action(self,call):
        method=call.func.attr;values=list(call.args);kwargs={k.arg:k.value for k in call.keywords}
        if len(kwargs)!=len(call.keywords):raise ValueError('Duplicate action kwargs')
        if method in {'make_ball_local','make_ball_global','remove_ball','add_ball_to_client_parks'}:
            if len(values)!=1 or kwargs:raise ValueError('Unreviewed park action arity')
            return ['assert!(ticker.actions.'+method+'(&mut park,'+self.primitive(values[0])+')==Ok(()));']
        if method=='set_ball_position':
            if len(values)!=4 or set(kwargs)-{'is_local_ball'}:raise ValueError('Unreviewed position action')
            values.append(kwargs.pop('is_local_ball',ast.Constant(False)))
        elif kwargs:raise ValueError('Unreviewed action keyword')
        expected={'go_to_point':4,'cloak_ball':3,'uncloak_ball':1,'set_ball_position':5,'set_ball_interactive':2,'set_speed_fraction':2}.get(method)
        if expected!=len(values):raise ValueError('Unreviewed source action '+method)
        result='ticker.actions.queue.'+method+'('+','.join(self.value(x) for x in values)+')'
        return ['assert!(ticker.refresh_action_balls(&park)==Ok(()));','assert!('+result+'.is_ok());']
    def setup(self,count,target=None):
        lines=[]
        for i in range(1,count+1):
            result='set_up_ball(&mut park,&mut ticker,BALL_ID_'+str(i)+',CHAR_'+str(i)+',CLIENT_'+str(i)+')'
            if target and count==1:self.balls.add(target);lines.append('let '+target+'='+result+';')
            else:lines.append('let _='+result+';')
        return lines
    def statement(self,n):
        if isinstance(n,ast.Expr) and isinstance(n.value,ast.Constant) and isinstance(n.value.value,str):return [] # docstring
        if isinstance(n,ast.Assign) and len(n.targets)==1:
            target=n.targets[0];value=n.value
            if isinstance(target,ast.Tuple) and isinstance(value,ast.Tuple) and len(target.elts)==len(value.elts):
                return sum((self.statement(ast.Assign(targets=[a],value=b)) for a,b in zip(target.elts,value.elts)),[])
            if isinstance(target,ast.Name) and isinstance(value,ast.Constant) and type(value.value) in (float,int,bool):
                self.variables[target.id]={int:'Integer',float:'Float',bool:'Boolean'}[type(value.value)]
                return ['let '+target.id+'='+self.primitive(value)+';']
            if isinstance(target,ast.Name) and isinstance(value,ast.Call):
                name=ast.unparse(value.func)
                if name=='self._set_up_single_ball' and not value.args and not value.keywords:return self.setup(1,target.id)
                if name=='add_ball_to_park' and len(value.args)==1 and ast.unparse(value.args[0])=='self._park' and len(value.keywords)==1 and value.keywords[0].arg=='ball_id':
                    self.balls.add(target.id);return ['let '+target.id+'=add_ball(&mut park,'+self.primitive(value.keywords[0].value)+');']
                if name=='mock.Mock' and not value.args and not value.keywords:
                    self.callback=target.id;return ['// original mock callback constructed with no invocations']
        if isinstance(n,ast.Expr) and isinstance(n.value,ast.Call):
            c=n.value;name=ast.unparse(c.func)
            if name.startswith('self._actions.'):return self.action(c)
            if name in {'self._set_up_single_ball','self._set_up_two_balls_in_same_bubble','self._set_up_three_balls_in_same_bubble'} and not c.args and not c.keywords:
                return self.setup({'self._set_up_single_ball':1,'self._set_up_two_balls_in_same_bubble':2,'self._set_up_three_balls_in_same_bubble':3}[name])
            if name=='add_ball_to_park' and len(c.args)==1 and ast.unparse(c.args[0])=='self._park' and len(c.keywords)==1 and c.keywords[0].arg=='ball_id':return ['let _=add_ball(&mut park,'+self.primitive(c.keywords[0].value)+');']
            if name=='self._ticker.tick' and not c.args and not c.keywords:return ['assert!(ticker.tick(&mut park)==Ok(()));']
            if name=='self._perform_tick' and not c.args and (not c.keywords or len(c.keywords)==1 and c.keywords[0].arg=='skip_evolve' and isinstance(c.keywords[0].value,ast.Constant) and type(c.keywords[0].value.value)is bool):
                return ['perform_tick(&mut park,&mut ticker,'+(self.primitive(c.keywords[0].value) if c.keywords else 'false')+');']
            if name=='self._network.clear' and not c.args and not c.keywords:return ['ticker.clear_network();']
            signal={'self._ticker.on_ball_cloaking.connect':('cloaking','Cloaking'),'self._ticker.on_ball_uncloaking.connect':('uncloaking','Uncloaking'),'self._ticker.on_set_ball_position.connect':('position','SetPosition')}
            if name in signal and len(c.args)==1 and isinstance(c.args[0],ast.Name) and c.args[0].id==self.callback and not c.keywords:
                field,self.callback_variant=signal[name];return ['ticker.'+field+'_connected=true;']
            if name==str(self.callback)+'.assert_called_once_with' and not c.keywords:
                expected=2 if self.callback_variant=='SetPosition' else 1
                if len(c.args)!=expected:raise ValueError('Changed mock assertion arity')
                return [f'// original-assertion: {SOURCE}:{n.lineno}','assert!(ticker.signal_calls.len()==1 && ticker.signal_calls.get(0)==Some(ServerTickerSignal::'+self.callback_variant+'('+','.join(self.primitive(x) for x in c.args)+')));']
            if name in {'self.assertEqual','self.assertTrue','self.assertFalse'} and not c.keywords:
                if name=='self.assertEqual' and len(c.args)==2:
                    if isinstance(c.args[0],ast.List) and not c.args[0].elts and ast.unparse(c.args[1]) in {'self._network.narrowcasts','self._network.singlecasts'}:condition='ticker.network.'+c.args[1].attr+'.is_empty()'
                    else:condition=self.primitive(c.args[0])+'=='+self.primitive(c.args[1])
                elif name in {'self.assertTrue','self.assertFalse'} and len(c.args)==1:
                    value=self.primitive(c.args[0]);condition=value+'!=0' if isinstance(c.args[0],ast.Attribute) and c.args[0].attr=='isCloaked' else value
                    if name=='self.assertFalse':condition='!('+condition+')'
                else:raise ValueError('Unreviewed assertion arity')
                return [f'// original-assertion: {SOURCE}:{n.lineno}','assert!('+condition+');']
            if name in {'self.assertUpdateNarrowcast','self.assertUpdateSinglecast'} and len(c.args)==2 and not c.keywords:
                single=name.endswith('Singlecast');action=c.args[1]
                if not isinstance(action,ast.Tuple) or len(action.elts)!=2 or not isinstance(action.elts[1],ast.Tuple):raise ValueError('Unreviewed expected action')
                code='BatcherEvent{timestamp:ticker.stamp_for_system-1,name:ActionName::'+ast.literal_eval(action.elts[0])+',payload:BatcherPayload::Tuple(args(&['+','.join(self.value(x) for x in action.elts[1].elts)+']))}'
                return [f'// original-assertion helper call: {SOURCE}:{n.lineno}',f'assert_update(&ticker,{self.client_ids(c.args[0],single)},{code},{str(single).lower()});']
            if name in {'self.assertAddBallsToParkSinglecastExistsForBall','self.assertAddBallsToParkNarrowcastExistsForBall'} and len(c.args)==2 and not c.keywords:
                single='Singlecast' in name
                return [f'// original-assertion helper call: {SOURCE}:{n.lineno}',f'assert_add_ball_cast(&park,&ticker,{self.primitive(c.args[0])},{self.client_ids(c.args[1],single)},{str(single).lower()});']
        raise ValueError('Unreviewed original statement '+ast.unparse(n))

def translate(case):
    source,fixture,test=case.split('::')
    if source!=SOURCE or fixture!='TestTicker':raise ValueError('Unreviewed source fixture')
    text=original(SOURCE);tree=ast.parse(text);cls=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name==fixture);review(cls)
    constants={n.targets[0].id:ast.literal_eval(n.value) for n in tree.body if isinstance(n,ast.Assign)}
    if constants!={'BALL_ID_1':1001,'BALL_ID_2':1002,'BALL_ID_3':1003,'CHAR_1':90000128,'CHAR_2':90000256,'CHAR_3':90000512,'CLIENT_1':13579,'CLIENT_2':24680,'CLIENT_3':80808}:raise ValueError('Changed original constants')
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test and n.name.startswith('test_'))
    if fn.decorator_list:raise ValueError('Unreviewed decorators')
    t=Translator();lines=[]
    for node in fn.body:lines+=t.statement(node)
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+f'''
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(1025))]
#[cfg_attr(test,test)]
fn {fixture}__{test}() {{
    let mut park=OriginalParkProjection::default();park.is_master=true;
    {{
    let mut ticker=OriginalServerTicker::default();
'''+''.join('    '+line+'\n' for line in lines)+'''
    assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)] kani::cover!(true,"complete original server ticker fixture and assertions reached");
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);parser.add_argument('--refresh',action='store_true');args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN case required')
    proof=translate(args.case);prelude=header();old=TARGET.read_text() if TARGET.exists() else prelude;marker='// original-case: '+args.case+'\n'
    if marker in old:
        if not args.refresh:raise ValueError('Case already translated')
        start=old.index(marker);end=old.find('\n// original-case: ',start+len(marker));end=len(old) if end<0 else end
        old=old[:start]+proof.lstrip('\n')+old[end:];proof=''
    elif args.refresh:raise ValueError('Refresh requires an existing exact case')
    first=old.find('\n// original-case: ')
    if first>=0:old=prelude+old[first:]
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
