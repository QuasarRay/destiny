#!/usr/bin/env python3
"""Complete original iterative scenarios, with checked bounds and full traces.

Uses the reviewed park AST translator for existing operations/assertion semantics.
Added syntax fails closed: dynamic indexing, float lists, original conditionals,
substep loops and source method fallback. Only the exact OPEN case is mutated.
"""
import argparse,ast,copy,hashlib,importlib.util,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('park_generator',ROOT/'verification/tools/generate_park_harnesses.py')
pg=importlib.util.module_from_spec(spec);spec.loader.exec_module(pg)
SOURCE='python/destiny/test/ballpark/evolve/test_iterative_collision.py'
TARGET=ROOT/'verification/kani/src/iterative_collision_contracts.rs'

PREAMBLE=pg.PREAMBLE+'''
// Iterative-specific source helpers. Native float pow/sqrt and Python list
// indexing/alias correspondence remain separate executable requirements.
fn point_at<const N:usize>(values:&BoundedList<Vec3,N>,index:i64)->Vec3 {
    let length=i64::try_from(values.len());assert!(length.is_ok());
    let actual=if index<0 { length.unwrap_or(0).checked_add(index) } else { Some(index) };
    assert!(actual.is_some());let actual=actual.unwrap_or(-1);
    assert!(actual>=0);let index=usize::try_from(actual);assert!(index.is_ok());
    let value=values.get(index.unwrap_or(usize::MAX));assert!(value.is_some());value.unwrap_or(Vec3::ZERO)
}
fn float_at<const N:usize>(values:&BoundedList<f64,N>,index:i64)->f64 {
    let length=i64::try_from(values.len());assert!(length.is_ok());
    let actual=if index<0 { length.unwrap_or(0).checked_add(index) } else { Some(index) };
    assert!(actual.is_some());let actual=actual.unwrap_or(-1);
    assert!(actual>=0);let index=usize::try_from(actual);assert!(index.is_ok());
    let value=values.get(index.unwrap_or(usize::MAX));assert!(value.is_some());value.unwrap_or(0.0)
}
fn distance_between_points(a:Vec3,b:Vec3)->f64 {
    // Complete pinned Python helper: sqrt((x1-x2)**2 + ...), in its order.
    ((a.x-b.x).powf(2.0)+(a.y-b.y).powf(2.0)+(a.z-b.z).powf(2.0)).sqrt()
}
'''

class Translation(pg.Translation):
    def vector(self,n):
        if isinstance(n,(ast.Tuple,ast.List)) and len(n.elts)==3:return self.vec(n.elts)
        code,kind=self.expression(n)
        if kind!='point':raise ValueError('Expected original three-coordinate value '+ast.unparse(n))
        return code
    def expression(self,n):
        if isinstance(n,ast.Call) and ast.unparse(n.func) in {'math.sin','math.cos','math.sqrt'} and len(n.args)==1 and not n.keywords:
            return '('+self.typed(n.args[0],'float')+').'+n.func.attr+'()','float'
        if isinstance(n,ast.Call) and ast.unparse(n.func)=='distance_between_points' and len(n.args)==2 and not n.keywords:
            return 'distance_between_points('+','.join(self.vector(x) for x in n.args)+')','float'
        if isinstance(n,ast.Call) and ast.unparse(n.func)=='len' and len(n.args)==1 and not n.keywords:
            code,kind=self.expression(n.args[0])
            if kind.startswith(('points:','floats:')):return '('+code+').len() as i64','int'
        if isinstance(n,ast.Subscript):
            code,kind=self.expression(n.value)
            if kind.startswith(('points:','floats:')):
                fn='point_at' if kind.startswith('points:') else 'float_at'
                return fn+'(&'+code+','+self.typed(n.slice,'int')+')','point' if fn=='point_at' else 'float'
        if isinstance(n,ast.BoolOp) and isinstance(n.op,(ast.And,ast.Or)):
            return '('+(' && ' if isinstance(n.op,ast.And) else ' || ').join(self.typed(x,'bool') for x in n.values)+')','bool'
        if isinstance(n,ast.UnaryOp) and isinstance(n.op,ast.Not):return '!('+self.typed(n.operand,'bool')+')','bool'
        if isinstance(n,ast.Attribute) and ast.unparse(n)=='self.park.friction':return 'park.friction','float'
        return super().expression(n)
    def assignment(self,target,value):
        if ast.unparse(target)=='self.park.friction':
            self.emit('park.friction='+self.typed(value,'float')+'; // original raw double park field write');return
        if isinstance(target,ast.Name) and isinstance(value,ast.List) and target.id not in self.point_variables:
            self.env[target.id]='floats:'+str(max(1,len(value.elts)))
            self.emit('let mut '+target.id+'=BoundedList::<f64,'+str(max(1,len(value.elts)))+'>::new();')
            for item in value.elts:self.emit('assert!('+target.id+'.push('+self.typed(item,'float')+'));')
            return
        if isinstance(target,ast.Name) and isinstance(value,ast.ListComp):
            if len(value.generators)!=1:raise ValueError('Unreviewed nested comprehension')
            generator=value.generators[0]
            if generator.ifs or generator.is_async or not isinstance(generator.target,ast.Name) or not isinstance(generator.iter,ast.Call) or ast.unparse(generator.iter.func)!='range' or len(generator.iter.args)!=1 or generator.iter.keywords:raise ValueError('Unreviewed comprehension scope')
            self.env[target.id]='floats:'+str(self.point_capacity)
            self.emit(f'let mut {target.id}=BoundedList::<f64,{self.point_capacity}>::new();')
            self.env[generator.target.id]='int';counter='comprehension_'+str(len(self.lines))
            self.emit(f'let mut {counter}=0_i64;');self.emit(f'while {counter}<{self.typed(generator.iter.args[0],"int")} {{');self.indentation+=1
            self.emit('let '+generator.target.id+'='+counter+';')
            self.emit('assert!('+target.id+'.push('+self.typed(value.elt,'float')+'));')
            self.emit(counter+'+=1;');self.indentation-=1;self.emit('}');return
        if isinstance(target,ast.Name) and isinstance(value,ast.Constant) and type(value.value)is bool:
            if target.id in self.env:self.emit(target.id+'='+self.typed(value,'bool')+';')
            else:self.env[target.id]='bool';self.emit('let mut '+target.id+'='+self.typed(value,'bool')+';')
            return
        return super().assignment(target,value)
    def statement(self,n):
        if isinstance(n,ast.Expr) and isinstance(n.value,ast.Constant) and isinstance(n.value.value,str):return # docstring
        if isinstance(n,ast.If):
            self.emit('if '+self.typed(n.test,'bool')+' {');self.indentation+=1
            for statement in n.body:self.statement(statement)
            self.indentation-=1;self.emit('}')
            if n.orelse:
                self.emit('else {');self.indentation+=1
                for statement in n.orelse:self.statement(statement)
                self.indentation-=1;self.emit('}')
            return
        if isinstance(n,ast.Break):self.emit('break; // original early loop exit');return
        if isinstance(n,ast.Try):
            if len(n.body)!=1 or len(n.handlers)!=1 or n.orelse or n.finalbody or ast.unparse(n.handlers[0].type)!='AttributeError' or n.handlers[0].name or len(n.handlers[0].body)!=1 or not isinstance(n.handlers[0].body[0],ast.Pass):raise ValueError('Unreviewed native fallback')
            call=n.body[0].value if isinstance(n.body[0],ast.Expr) else None
            if not isinstance(call,ast.Call) or ast.unparse(call.func)!='self.park.SetBallRotation' or len(call.args)!=5 or call.keywords:raise ValueError('Unreviewed optional native method')
            self.emit('// Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.')
            self.emit('park.set_ball_rotation('+self.typed(call.args[0],'int')+',['+','.join(self.typed(x,'float') for x in call.args[1:])+']);')
            return
        if isinstance(n,ast.For) and isinstance(n.target,ast.Name) and isinstance(n.iter,ast.Call) and ast.unparse(n.iter.func)=='range' and len(n.iter.args)==1 and not n.iter.keywords and not n.orelse:
            bound=self.typed(n.iter.args[0],'int');counter='iteration_'+str(len(self.lines));self.env[n.target.id]='int'
            self.emit('let mut '+counter+'=0_i64;');self.emit('while '+counter+'<'+bound+' {');self.indentation+=1
            self.emit('let '+n.target.id+'='+counter+';')
            for statement in n.body:self.statement(statement)
            self.emit(counter+'+=1;');self.indentation-=1;self.emit('}');return
        if isinstance(n,ast.Expr) and isinstance(n.value,ast.Call):
            call=n.value
            if ast.unparse(call.func)=='self.assertAlmostEqual' and len(call.args)==3:
                if any(k.arg=='places' for k in call.keywords):raise ValueError('Duplicate original places argument')
                places=ast.literal_eval(call.args[2])
                if type(places)is not int:raise ValueError('Unreviewed positional decimal precision')
                normalized=copy.deepcopy(n)
                normalized.value.args=normalized.value.args[:2]
                normalized.value.keywords.append(ast.keyword(arg='places',value=ast.Constant(places)))
                return super().statement(normalized)
            if isinstance(call.func,ast.Attribute) and call.func.attr=='append' and isinstance(call.func.value,ast.Name) and call.func.value.id in self.point_variables and len(call.args)==1 and not call.keywords and isinstance(call.args[0],(ast.Tuple,ast.List)) and len(call.args[0].elts)==3:
                self.emit('assert!('+call.func.value.id+'.push('+self.vec(call.args[0].elts)+')); // original complete point/velocity trace');return
        return super().statement(n)

def translate(case):
    source,fixture,test=case.split('::')
    if source!=SOURCE or fixture not in {'TestIterativeCollision','TestIterativeCapsuleCollision','TestIterativeBoxCollision'}:raise ValueError('Unreviewed source fixture')
    text=pg.original(source);tree=ast.parse(text);cls=next(n for n in tree.body if isinstance(n,ast.ClassDef) and n.name==fixture)
    if [ast.unparse(x) for x in cls.bases]!=['helpers.BallparkTestCase']:raise ValueError('Changed original inheritance')
    body=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name==test)
    if body.decorator_list:raise ValueError('Unreviewed decorator')
    setup=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='setUp')
    teardown=next(n for n in cls.body if isinstance(n,ast.FunctionDef) and n.name=='tearDown')
    if fixture=='TestIterativeCollision':
        expected_setup='def setUp(self):\n self.maxDiff=None\n super(TestIterativeCollision,self).setUp()\n config=destiny.settings.GetDefault()\n config.useIterativeCollision=True\n destiny.settings.Apply(config)\n'
        expected_teardown='def tearDown(self):\n super(TestIterativeCollision,self).tearDown()\n destiny.settings.Reset()\n'
        suffix=[]
    elif fixture=='TestIterativeCapsuleCollision':
        expected_setup='def setUp(self):\n super(TestIterativeCapsuleCollision,self).setUp()\n config=destiny.settings.GetDefault()\n config.useIterativeCollision=True\n destiny.settings.Apply(config)\n parentBall=helpers.add_ball_to_park(self.park,x=0.0,y=0.0,z=0.0,radius=500.0)\n parentBall.AddMiniCapsule(100.0,100.0,0.0,200.0,100.0,0.0,10.0)\n self.capsule=parentBall.miniCapsules[0]\n'
        expected_teardown='def tearDown(self):\n destiny.settings.Reset()\n super(TestIterativeCapsuleCollision,self).tearDown()\n del self.capsule\n'
        suffix=setup.body[4:]
    else:
        expected_setup='def setUp(self):\n super(TestIterativeBoxCollision,self).setUp()\n config=destiny.settings.GetDefault()\n config.useIterativeCollision=True\n destiny.settings.Apply(config)\n parentBall=helpers.add_ball_to_park(self.park,x=0,y=0,z=0)\n parentBall.AddMiniBox(0.0,0.0,0.0,100.0,0.0,0.0,0.0,100.0,0.0,0.0,0.0,100.0)\n self.box=parentBall.miniBoxes[0]\n'
        expected_teardown='def tearDown(self):\n destiny.settings.Reset()\n super(TestIterativeBoxCollision,self).tearDown()\n del self.box\n'
        suffix=setup.body[4:]
    if ast.dump(setup)!=ast.dump(ast.parse(expected_setup).body[0]) or ast.dump(teardown)!=ast.dump(ast.parse(expected_teardown).body[0]):raise ValueError('Changed complete iterative fixture lifecycle')
    distance=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='distance_between_points')
    expected_distance=ast.parse('def distance_between_points(point1,point2):\n return math.sqrt((point1[0]-point2[0])**2+(point1[1]-point2[1])**2+(point1[2]-point2[2])**2)\n').body[0]
    if ast.dump(distance)!=ast.dump(expected_distance):raise ValueError('Changed complete distance helper')
    class BindFixture(ast.NodeTransformer):
        def visit_Attribute(self,n):
            if isinstance(n.value,ast.Name) and n.value.id=='self' and n.attr in {'capsule','box'}:return ast.copy_location(ast.Name(id='fixture_'+n.attr,ctx=n.ctx),n)
            return self.generic_visit(n)
    body=BindFixture().visit(copy.deepcopy(body));suffix=[BindFixture().visit(copy.deepcopy(n)) for n in suffix]
    t=Translation(source);t.evolve_method='evolve_two_spherical_bodies';t.point_capacity=200;t.maximum_iterations=176
    t.point_variables={n.func.value.id for n in ast.walk(body) if isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and n.func.attr=='append' and isinstance(n.func.value,ast.Name) and len(n.args)==1 and isinstance(n.args[0],(ast.List,ast.Tuple)) and len(n.args[0].elts)==3}
    if any(isinstance(n,ast.Call) and ast.unparse(n.func)=='self.evolve_ball_and_get_coordinates' for n in ast.walk(body)):
        helpers=ast.parse(pg.original('python/destiny/test/helpers.py'));base=next(n for n in helpers.body if isinstance(n,ast.ClassDef) and n.name=='BallparkTestCase')
        helper=next(n for n in base.body if isinstance(n,ast.FunctionDef) and n.name=='evolve_ball_and_get_coordinates')
        wanted=ast.parse('def evolve_ball_and_get_coordinates(self,ball,step_count):\n coordinates=[]\n for i in range(step_count):\n  self.park.Evolve()\n  coordinates.append((ball.x,ball.y,ball.z))\n return coordinates\n').body[0]
        if ast.dump(helper)!=ast.dump(wanted):raise ValueError('Changed full coordinate helper')
        t.helpers['self.evolve_ball_and_get_coordinates']=helper;t.point_variables.add('coordinates')
    if any(isinstance(n,ast.Call) and ast.unparse(n.func)=='helpers.create_space_ball' for n in ast.walk(body)):
        t.env['next_space_object_id']='int';t.emit('let mut next_space_object_id=1_i64; // fresh helper-module counter; native suite order remains an environment obligation')
    t.emit('park.iterative_collision=true; // GetDefault/Apply source default: twenty passes, old orientation')
    for n in suffix:t.statement(n)
    for n in body.body:
        if isinstance(n,ast.Return):
            if n.value is not None:raise ValueError('Unreviewed return value')
            t.emit('// Original unconditional return ends this test body. Its later source assertions are dead Python code; teardown still runs.')
            break
        t.statement(n)
    if fixture!='TestIterativeCollision':t.emit('park.iterative_collision=false; // original Reset occurs before base teardown')
    t.emit('assert!(!park.scope_error);');t.emit('park.clear_all();');t.emit('assert!(!park.scope_error);')
    if fixture=='TestIterativeCollision':t.emit('park.iterative_collision=false; // original Reset occurs after base teardown')
    return '\n// original-case: '+case+'\n// original-source-sha256: '+hashlib.sha256(text.encode()).hexdigest()+f'''
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(177))]
#[cfg_attr(test,test)]
fn {fixture}__{test}() {{
    let mut park=OriginalParkProjection::default();
'''+ '\n'.join(t.lines)+'''
    #[cfg(kani)] kani::cover!(true,"complete original iterative collision scenario and teardown reached");
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--case',required=True);parser.add_argument('--refresh',action='store_true');args=parser.parse_args()
    focus=json.loads((ROOT/'.aegis/destiny/harness-focus.json').read_text())
    if focus.get('status')!='OPEN' or focus.get('request',{}).get('case')!=args.case:raise ValueError('Exact OPEN case required')
    proof=translate(args.case);old=TARGET.read_text() if TARGET.exists() else PREAMBLE;marker='// original-case: '+args.case+'\n'
    if marker in old:
        if not args.refresh:raise ValueError('Case already translated')
        start=old.index(marker);end=old.find('\n// original-case: ',start+len(marker));end=len(old) if end<0 else end
        old=old[:start]+proof.lstrip('\n')+old[end:];proof=''
    elif args.refresh:raise ValueError('Refresh requires an existing exact case')
    first=old.find('\n// original-case: ')
    if first>=0:old=PREAMBLE+old[first:]
    TARGET.write_text(old+proof);print('Translated',args.case)

if __name__=='__main__':main()
