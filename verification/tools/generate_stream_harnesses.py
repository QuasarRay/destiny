#!/usr/bin/env python3
"""Translate complete original stream scenarios with all assertion helpers.

The Rust byte codec models the pinned little-endian old-orientation layout;
Blue stream dispatch, references and ABI require separate runtime evidence.
"""
import argparse,ast,hashlib,importlib.util,json
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[2]
REV='ef738406b61ad06bae2f402857e464735ab106a2'
SOURCE='python/destiny/test/ballpark/test_stream_read_write.py'
TARGET=ROOT/'verification/kani/src/stream_contracts.rs'

def original(path):
    return subprocess.run(['git','show',REV+':'+path],cwd=ROOT,capture_output=True,text=True,check=True).stdout

spec=importlib.util.spec_from_file_location('park_generator',ROOT/'verification/tools/generate_park_harnesses.py')
pg=importlib.util.module_from_spec(spec);spec.loader.exec_module(pg)

PREAMBLE=pg.PREAMBLE+'''\nuse destiny_original_spec::OriginalStream;
fn stream_ids(ids:&[i64])->BoundedList<i64,8> { id_list(ids) }
'''

def snapshot_value(node,environment):
    if isinstance(node,ast.Call) and ast.unparse(node.func)=='len' and len(node.args)==1:
        return snapshot_value(node.args[0],environment)+'.len()'
    if not isinstance(node,ast.Attribute):raise ValueError('Unreviewed helper expression '+ast.unparse(node))
    name=ast.unparse(node.value)
    if name not in environment:raise ValueError('Unreviewed helper receiver')
    receiver,kind=environment[name]
    if kind=='ball':
        mapping={**{k:v[0] for k,v in pg.FIELDS.items()},'id':'id','formationID':'formation_id',
            'showBoxes':'show_boxes','miniBalls':'mini_balls','miniCapsules':'mini_capsules','miniBoxes':'mini_boxes'}
        if node.attr in {'surfaceDist','centerDist'}:
            # These exact fixtures have no ego, so the original readonly
            # getters return zero. The caller proves this checked setup.
            return '0.0_f64'
        if node.attr not in mapping:raise ValueError('Unreviewed ball helper field '+node.attr)
        return receiver+'.'+mapping[node.attr]
    mapping={'mini-ball':{'id':'id','x':'position.x','y':'position.y','z':'position.z'},
        'mini-capsule':{'id':'id','ax':'a.x','ay':'a.y','az':'a.z','bx':'b.x','by':'b.y','bz':'b.z'},
        'mini-box':{'id':'id',**{axis+str(i):field+'.'+component for axis,field in [('c','corner'),('x','x'),('y','y'),('z','z')] for i,component in enumerate('xyz')}}}
    if node.attr not in mapping[kind]:raise ValueError('Unreviewed child helper field')
    return receiver+'.'+mapping[kind][node.attr]

def translate(case):
    source,fixture,test=case.split('::')
    if source!=SOURCE or fixture not in {'TestWriteBallsToStream','TestWriteFullStateToStream'}:raise ValueError('Unreviewed stream fixture')
    text=original(source);tree=ast.parse(text);cls=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name==fixture)
    if [ast.unparse(x) for x in cls.bases]!=['helpers.BallparkTestCase'] or any(isinstance(n,ast.FunctionDef) and n.name in {'setUp','tearDown'} for n in cls.body):raise ValueError('Stream lifecycle changed')
    fn=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test)
    if fn.decorator_list:raise ValueError('Unreviewed stream decorators')
    helper_source=original('python/destiny/test/helpers.py');base=next(n for n in ast.parse(helper_source).body if isinstance(n,ast.ClassDef) and n.name=='BallparkTestCase')
    helpers={n.name:n for n in base.body if isinstance(n,ast.FunctionDef)}
    t=pg.Translation(source);captured=False;reader_name='park'
    for statement in fn.body:
        if isinstance(statement,ast.Assign) and len(statement.targets)==1 and ast.unparse(statement.targets[0])=='stream' and ast.unparse(statement.value)=='blue.MemStream()':
            t.emit('let mut stream=OriginalStream::default(); // native stream constructor boundary');continue
        if isinstance(statement,ast.Assign) and len(statement.targets)==1 and ast.unparse(statement.targets[0])=='reader_park' and ast.unparse(statement.value)=='destiny.Ballpark()':
            t.emit('let mut reader_park=OriginalParkProjection::default();');reader_name='reader_park';continue
        if isinstance(statement,ast.Assign) and len(statement.targets)==1 and ast.unparse(statement.targets[0])=='loaded_ball' and isinstance(statement.value,ast.Subscript):
            expected=reader_name+'.balls[saved_ball.id]' if reader_name!='park' else 'self.park.balls[saved_ball.id]'
            if ast.unparse(statement.value)!=expected:raise ValueError('Unreviewed loaded-ball lookup')
            t.emit(f'assert!({reader_name}.contains(saved_ball));');t.emit(f'let loaded_ball={reader_name}.retained(saved_ball);');continue
        if isinstance(statement,ast.Expr) and isinstance(statement.value,ast.Call):
            call=statement.value;method=ast.unparse(call.func)
            if method in {'self.park.WriteBallsToStream','self.park.WriteFullStateToStream'}:
                if call.keywords:raise ValueError('Unreviewed stream writer keyword')
                if method.endswith('WriteBallsToStream'):
                    if len(call.args)!=2 or ast.unparse(call.args[1])!='stream' or not isinstance(call.args[0],ast.List):raise ValueError('Writer argument shape changed')
                    ids=[t.typed(x,'int') for x in call.args[0].elts]
                    if ids:code='park.write_original_balls(&stream_ids(&['+', '.join(ids)+']))'
                    else:
                        t.emit('let original_empty_ids=BoundedList::<i64,8>::new();')
                        t.emit('assert!(original_empty_ids.is_empty()); // original [] iteration has zero elements')
                        code='park.write_original_empty_balls(&original_empty_ids)'
                else:
                    if len(call.args)!=1 or ast.unparse(call.args[0])!='stream':raise ValueError('Full writer argument shape changed')
                    code='park.write_original_full_state(None)'
                t.emit('let written_stream='+code+';');t.emit('assert!(written_stream.is_ok()); // original write must not raise');t.emit('stream=written_stream.unwrap_or_default();')
                if 'saved_ball' in t.balls:
                    t.emit('let saved_snapshot=park.retained(saved_ball);');captured=True
                continue
            if method in {'self.park.ReadFullStateFromStream','reader_park.ReadFullStateFromStream'}:
                if [ast.unparse(x) for x in call.args]!=['stream'] or call.keywords:raise ValueError('Read argument shape changed')
                target='park' if method.startswith('self.park.') else 'reader_park'
                t.emit(f'assert!({target}.read_original_state(&stream,0)==Ok(())); // original read must not raise');continue
            if method=='self.assertEqual' and len(call.args)==2 and ast.unparse(call.args[0])=='stream.Read()' and isinstance(call.args[1],ast.Constant) and isinstance(call.args[1].value,bytes):
                t.emit(f'// original-assertion: {source}:{call.lineno}');t.emit('assert!(stream.equals_bytes(&['+', '.join(str(x)+'_u8' for x in call.args[1].value)+']));');continue
            if method=='self.assertIn' and len(call.args)==2 and ast.unparse(call.args[0])=='saved_ball.id' and ast.unparse(call.args[1]) in {'reader_park.balls','self.park.balls'}:
                target='reader_park' if ast.unparse(call.args[1])=='reader_park.balls' else 'park'
                t.emit(f'// original-assertion: {source}:{call.lineno}');t.emit(f'assert!({target}.contains(saved_ball));');continue
            if method in {'self.assertBallEqual','self.assertMiniBallEqual','self.assertMiniCapsuleEqual','self.assertMiniBoxEqual'}:
                if not captured or len(call.args)!=2 or call.keywords:raise ValueError('Complete helper arguments required')
                name=call.func.attr;helper=helpers[name]
                environment={'first':('saved_snapshot','ball'),'second':('loaded_ball','ball')}
                t.emit(f'// original-assertion: {source}:{call.lineno}; complete helper below')
                t.emit(f'assert!(park.ego_id==0 && {reader_name}.ego_id==0 && saved_snapshot.client_ball && loaded_ball.client_ball); // no-ego readonly property caller scope')
                if name!='assertBallEqual':
                    kind={'assertMiniBallEqual':'mini-ball','assertMiniCapsuleEqual':'mini-capsule','assertMiniBoxEqual':'mini-box'}[name]
                    field={'mini-ball':'mini_balls','mini-capsule':'mini_capsules','mini-box':'mini_boxes'}[kind]
                    expected=[f'saved_ball.{ {"mini-ball":"miniBalls","mini-capsule":"miniCapsules","mini-box":"miniBoxes"}[kind]}[0]',f'loaded_ball.{ {"mini-ball":"miniBalls","mini-capsule":"miniCapsules","mini-box":"miniBoxes"}[kind]}[0]']
                    if [ast.unparse(x) for x in call.args]!=expected:raise ValueError('Child helper indices changed')
                    t.emit(f'let saved_child=saved_snapshot.{field}.get(0);let loaded_child=loaded_ball.{field}.get(0);')
                    t.emit('assert!(saved_child.is_some() && loaded_child.is_some()); // original list indices must exist')
                    fallback={'mini-ball':'OriginalMiniBall{id:0,position:Vec3::ZERO,radius:0.0}',
                        'mini-capsule':'OriginalMiniCapsule{id:0,parent_id:0,a:Vec3::ZERO,b:Vec3::ZERO,radius:0.0}',
                        'mini-box':'OriginalMiniBox{id:0,parent_id:0,corner:Vec3::ZERO,x:Vec3::ZERO,y:Vec3::ZERO,z:Vec3::ZERO}'}[kind]
                    t.emit(f'let saved_child=saved_child.unwrap_or({fallback});let loaded_child=loaded_child.unwrap_or({fallback});')
                    environment={'first':('saved_child',kind),'second':('loaded_child',kind)}
                elif [ast.unparse(x) for x in call.args]!=['saved_ball','loaded_ball']:raise ValueError('Ball helper arguments changed')
                for check in helper.body:
                    if not isinstance(check,ast.Expr) or not isinstance(check.value,ast.Call) or ast.unparse(check.value.func)!='self.assertEqual' or len(check.value.args)!=2 or check.value.keywords:raise ValueError('Unreviewed assertion helper statement')
                    t.emit(f'// original-assertion: python/destiny/test/helpers.py:{check.lineno}')
                    t.emit(f'assert!({snapshot_value(check.value.args[0],environment)}=={snapshot_value(check.value.args[1],environment)});')
                continue
        # Reviewed ordinary fixture additions, native child calls and ClearAll.
        t.statement(statement)
    entry=fixture+'__'+test
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+f'\n#[cfg_attr(kani,kani::proof)]\n#[cfg_attr(kani,kani::unwind(129))]\n#[cfg_attr(test,test)]\nfn {entry}() {{\n    let mut park=OriginalParkProjection::default();\n'+'\n'.join(t.lines)+'\n    assert!(!park.scope_error);\n    park.clear_all();\n    assert!(!park.scope_error);\n    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");\n}\n'

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);parser.add_argument('--refresh',action='store_true');args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN work packet required')
    proof=translate(args.case);old=TARGET.read_text() if TARGET.exists() else PREAMBLE
    marker='// original-case: '+args.case+'\n'
    if marker in old:
        if not args.refresh:raise ValueError('Case already translated')
        start=old.index(marker);end=old.find('\n// original-case: ',start+len(marker))
        if end<0:end=len(old)
        old=old[:start]+proof.lstrip('\n')+old[end:];proof=''
    elif args.refresh:raise ValueError('Refresh requires an existing exact case')
    first=old.find('\n// original-case: ')
    if first>=0:old=PREAMBLE+old[first:]
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
