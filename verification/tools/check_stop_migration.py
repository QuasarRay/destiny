#!/usr/bin/env python3
"""Independent unchanged Original stop/time-factor/integration observations.

The source slice has its real Vector3d arithmetic and scalar widths. The
scaffold supplies fields/declarations only; it is not a Blue/layout oracle.
"""
import ctypes
import hashlib
import json
from pathlib import Path
import random
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from tools.migrate_destiny import LATEST, STATE, blob, checked, save


def digest(data):
    return hashlib.sha256(data).hexdigest()


def kinematic_oracle(directory,header,compiler):
    """Exact source field writes/swap/query; full Evolve hosting is excluded."""
    sources={p:blob(LATEST,p) for p in ['src/Ballpark.cpp','src/Ball.cpp','src/Vector3d.h']};regions=[]
    def take(path,a,b):
        data=b''.join(sources[path].splitlines(keepends=True)[a-1:b]);regions.append({'source':path,'start':a,'end':b,'region_sha256':digest(data)});return data.decode()
    create=take('src/Ballpark.cpp',3356,3371);position=take('src/Ballpark.cpp',4750,4757);velocity=take('src/Ballpark.cpp',4782,4789);stage=take('src/Ballpark.cpp',520,521);swap=take('src/Ballpark.cpp',581,582);acceleration=take('src/Ball.cpp',1484,1484)
    receipt=json.loads((ROOT/'.aegis/destiny/math-sources/latest/source-receipt.json').read_text())
    assert receipt['commit']=='540edbfbb779a8f5616e9bbdfbf595c564af5a2c'
    math=ROOT/receipt['source_root'];constructor_file=math/'include/Vector3_inline.h';vector_header=math/'include/Vector3.h'
    constructor=b''.join(constructor_file.read_bytes().splitlines(keepends=True)[12:18]).decode()
    assert constructor.startswith('inline constexpr Vector3::Vector3( float x_')
    scaffold=r'''#include <cmath>
#include <utility>
#include <cstdint>
struct Vector3 {float x,y,z;constexpr Vector3(float,float,float);};
'''+constructor+r'''
#include "Vector3d.h"
struct Park{double dt;};
struct Ball{Vector3d mNewPos,mNewVel,mOldPos,mOldVel;};
extern "C" void destiny_kinematic_step(double* state,int operation,const double* value,double dt,double* result) {
 Ball object;Ball* ball=&object;
 Vector3d* fields[]={&ball->mNewPos,&ball->mNewVel,&ball->mOldPos,&ball->mOldVel};
 for(int i=0;i<4;++i)*fields[i]=Vector3d(state[i*3],state[i*3+1],state[i*3+2]);
 double x=value[0],y=value[1],z=value[2],vx=value[3],vy=value[4],vz=value[5];
 if(operation==0 || operation==1) {bool created=operation==0;
'''+create+r'''
 }else if(operation==2){
'''+position+r'''
 }else if(operation==3){vx=value[0];vy=value[1];vz=value[2];
'''+velocity+r'''
 }else if(operation==4){Vector3d p(value[0],value[1],value[2]),v(value[3],value[4],value[5]);
'''+stage+r'''
 }else if(operation==5){
'''+swap+r'''
 }
 for(int i=0;i<4;++i){state[i*3]=fields[i]->x;state[i*3+1]=fields[i]->y;state[i*3+2]=fields[i]->z;}
 Park park{dt};Park* mPark=&park;Vector3d mNewVel=ball->mNewVel,mOldVel=ball->mOldVel;Vector3 out(0.,0.,0.);Vector3* in=&out;
'''+acceleration+r'''
 result[0]=out.x;result[1]=out.y;result[2]=out.z;
}
'''
    path=directory/'kinematic_oracle.cpp';library=directory/'liboriginal_kinematic.so';path.write_text(scaffold)
    command=['clang++','-std=c++17','-O2','-shared','-fPIC','-fno-fast-math','-ffp-contract=off',str(path),'-o',str(library)]
    checked(command,'world-kinematic-oracle-build',60)
    call=ctypes.CDLL(str(library)).destiny_kinematic_step;call.argtypes=[ctypes.POINTER(ctypes.c_double),ctypes.c_int,ctypes.POINTER(ctypes.c_double),ctypes.c_double,ctypes.POINTER(ctypes.c_double)];call.restype=None
    cases=[]
    for dt in [1.,.25,.375,.0625]:
        actions=[(0,0,[2.,-3.,.25,7.1,-9.3,1e-5]),(0,1,[11.,13.,17.,.75,-.125,1./3.]),(0,2,[.125,-.5,20.]),(0,3,[3.000000119,1e-30,-2.5]),(0,4,[21.,31.,41.,1.00000003,2.0000001,3.0000002]),(0,5,[])]
        cases.append({'name':'lifecycle-'+str(dt),'dt':dt,'bodies':1,'actions':actions})
    cases.append({'name':'callback-replaces-later-pending-result','dt':.25,'bodies':2,'actions':[(0,0,[0.,1.,2.,3.,4.,5.]),(1,0,[10.,20.,30.,6.,7.,8.]),(0,4,[100.,101.,102.,103.,104.,105.]),(1,4,[200.,201.,202.,203.,204.,205.]),(0,5,[]),(1,2,[-50.,-60.,-70.]),(1,3,[.125,-.25,.375]),(1,5,[])]})
    cases.append({'name':'actual-stop-tick-publication','dt':.25,'friction':5.,'bodies':1,'actions':[(0,0,[0.,1.,1.,500.,0.,0.]),(0,7,[])]})
    cases.append({'name':'underflow-retains-prior-velocity','dt':.25,'friction':1_000_000.,'bodies':1,'actions':[(0,0,[0.,1.,1.,500.,0.,0.]),(0,7,[])]})
    stop_library=directory/'liboriginal_stop.so'
    stop=ctypes.CDLL(str(stop_library)).destiny_original_stop
    stop.argtypes=[ctypes.POINTER(ctypes.c_double),ctypes.c_double,ctypes.c_float,ctypes.c_double,ctypes.c_double];stop.restype=None
    for case in cases:
        states=[(ctypes.c_double*12)() for _ in range(case['bodies'])];present=set();observations=[]
        for target,operation,values in case['actions']:
            result=(ctypes.c_double*3)()
            if operation==7:
                future=(ctypes.c_double*6)(*list(states[target])[:6]);stop(future,10.,1.,case['friction'],case['dt'])
                call(states[target],4,future,case['dt'],result)
                call(states[target],5,(ctypes.c_double*6)(),case['dt'],result)
            else:call(states[target],operation,(ctypes.c_double*6)(*(values+[0.]*(6-len(values)))),case['dt'],result)
            if operation in [0,1]:present.add(target)
            rows=[]
            for index,state in enumerate(states):
                if index not in present:rows.append(None);continue
                query=(ctypes.c_double*3)();call(state,6,(ctypes.c_double*6)(),case['dt'],query)
                rows.append({'state_bits':[struct.pack('>d',v).hex() for v in state],'acceleration_bits':[struct.pack('>d',v).hex() for v in query]})
            observations.append(rows)
        case['observations']=observations
    external={str(p.relative_to(ROOT)):digest(p.read_bytes()) for p in [constructor_file,vector_header]}
    report={'schema':1,'original_commit':LATEST,'source_sha256':{p:digest(v) for p,v in sources.items()},'source_regions':regions,'external_source_sha256':external,'external_commit':receipt['commit'],'scaffold_sha256':digest(path.read_bytes()),'library_sha256':digest(library.read_bytes()),'stop_reference_library_sha256':digest(stop_library.read_bytes()),'command':command,'compiler':compiler,'checker_sha256':digest(Path(__file__).read_bytes()),'cases':cases,'proof_credit':False,'scope':'Unchanged Original AddBall old/new vector writes, command writes, result staging, publication swap and live acceleration expression with actual Vector3d and pinned Carbon Vector3 copy constructor. Explicit field-state/phase inputs; second-body command occurs between source swaps. One live noncolliding Stop tick also executes the existing unchanged Stop/integration oracle at mass=10/agility=1/friction=5/dt=.25 with initial velocity 500, then stages/swaps exact source fields. No full AddBall/Evolve/partition/angle/automatic callback/Blue ABI, lifetime, dynamic-orientation or formal claim.'}
    save(ROOT/'verification/migration/kinematic_oracle_samples.json',report)


def dispatch_oracle(directory, header, compiler):
    """Unchanged DispatchCollisions with a typed, reentrant host-call boundary."""
    source=blob(LATEST,'src/Ball.cpp');lines=source.decode().splitlines(keepends=True)
    selected=''.join(lines[467:488])
    assert selected.startswith('void Ball::DispatchCollisions()') and selected.rstrip().endswith('}')
    scaffold=r'''#include <cstdint>
#include <vector>
#include <cstring>
#include <cstdlib>
#include <cmath>
struct Vector3 {float x,y,z;Vector3(float,float,float);};
#include "Vector3d.h"
struct Ball;using IBall=Ball;
using Hook=int(*)(void*,int,int64_t,double,double,double);
struct Environment {
 Hook hook;Ball* active;int errors=0;
 bool SendEvent(IBall* ball,const char* channel,const char* method,void* target,const char* format,int64_t other,double x,double y,double z) {
  if(std::strcmp(channel,"Destiny::Ball::DoCollision") || std::strcmp(method,"DoCollision") || target || std::strcmp(format,"Lddd"))std::abort();
  active=ball;return hook(ball,6,other,x,y,z)!=0;
 }
 void PyError(){++errors;hook(active,0,0,0.,0.,0.);}
};
static Environment* PyOS=nullptr;
struct Ball {std::vector<int64_t> mCollisions;Vector3d mLastC;void DispatchCollisions();};
'''+selected+r'''
extern "C" int destiny_dispatch(const int64_t* ids,size_t count,const double* force,Hook hook) {
 Ball ball;ball.mCollisions.assign(ids,ids+count);ball.mLastC=Vector3d(force[0],force[1],force[2]);
 Environment environment{hook,&ball};auto previous=PyOS;PyOS=&environment;
 ball.DispatchCollisions();PyOS=previous;return environment.errors;
}
extern "C" void destiny_dispatch_append(void* pointer,int64_t id){static_cast<Ball*>(pointer)->mCollisions.push_back(id);}
extern "C" void destiny_dispatch_replace(void* pointer,size_t index,int64_t id){static_cast<Ball*>(pointer)->mCollisions.at(index)=id;}
extern "C" void destiny_dispatch_force(void* pointer,const double* force){static_cast<Ball*>(pointer)->mLastC=Vector3d(force[0],force[1],force[2]);}
extern "C" void destiny_dispatch_again(void* pointer){static_cast<Ball*>(pointer)->DispatchCollisions();}
'''
    path=directory/'collision_dispatch_oracle.cpp';library=directory/'liboriginal_collision_dispatch.so';path.write_text(scaffold)
    command=['clang++','-std=c++17','-O2','-shared','-fPIC','-fno-fast-math','-ffp-contract=off',str(path),'-o',str(library)]
    checked(command,'world-collision-dispatch-oracle-build',60)
    loaded=ctypes.CDLL(str(library));hook_type=ctypes.CFUNCTYPE(ctypes.c_int,ctypes.c_void_p,ctypes.c_int,ctypes.c_int64,ctypes.c_double,ctypes.c_double,ctypes.c_double)
    call=loaded.destiny_dispatch;call.argtypes=[ctypes.POINTER(ctypes.c_int64),ctypes.c_size_t,ctypes.POINTER(ctypes.c_double),hook_type];call.restype=ctypes.c_int
    append=loaded.destiny_dispatch_append;append.argtypes=[ctypes.c_void_p,ctypes.c_int64];append.restype=None
    replace=loaded.destiny_dispatch_replace;replace.argtypes=[ctypes.c_void_p,ctypes.c_size_t,ctypes.c_int64];replace.restype=None
    force=loaded.destiny_dispatch_force;force.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_double)];force.restype=None
    again=loaded.destiny_dispatch_again;again.argtypes=[ctypes.c_void_p];again.restype=None
    bits=lambda x:struct.pack('>d',x).hex()
    decode=lambda words:[struct.unpack('>d',bytes.fromhex(w))[0] for w in words]
    cases=[
      {'name':'empty','ids':[],'force_bits':list(map(bits,[1.,2.,3.])),'actions':[]},
      {'name':'ordered-duplicates-and-full-ids','ids':[5,5,0,-8,-9223372036854775808,9223372036854775807],'force_bits':list(map(bits,[-0.,2.,-3.])),'actions':[]},
      {'name':'append-replace-force-and-error','ids':[7,7,0,-2],'force_bits':list(map(bits,[1.,2.,3.])),'actions':[{'at':0,'append':[77,88],'replace':[1,9223372036854775807],'force_bits':list(map(bits,[4.,5.,6.])),'success':False,'error_force_bits':list(map(bits,[-1.,-2.,-3.]))},{'at':2,'replace':[3,-9223372036854775808],'force_bits':list(map(bits,[0.,-0.,1e-300]))}]},
      {'name':'nonfinite-payload-bits','ids':[0,0],'force_bits':['8000000000000000','7ff8000000000035','fff0000000000000'],'actions':[{'at':0,'force_bits':['0000000000000001','7ff0000000000000','fff8000000000047']}]},
      {'name':'three-hundred-source-events','ids':[i%7-3 for i in range(300)],'force_bits':list(map(bits,[.25,-.5,.75])),'actions':[{'at':100,'success':False}]},
      {'name':'nested-same-owner','ids':[9,7,0],'force_bits':list(map(bits,[1.,2.,3.])),'expected_contacts':7,'outer_host_errors':0,'actions':[{'at':0,'append':[99],'force_bits':list(map(bits,[4.,5.,6.])),'reenter':True},{'at':1,'replace':[1,-7],'force_bits':list(map(bits,[-4.,-5.,-6.])),'success':False,'error_force_bits':list(map(bits,[9.,8.,7.]))}]},
    ]
    for case in cases:
        trace=[];ordinal=0;pending_error=None;callback_errors=[]
        def observe(pointer,kind,other,x,y,z):
            nonlocal ordinal,pending_error
            if kind==0:
                trace.append({'kind':'host_error'})
                if pending_error is not None:force(pointer,(ctypes.c_double*3)(*decode(pending_error)))
                return 1
            trace.append({'kind':'contact','other_id':other,'force_bits':list(map(bits,[x,y,z]))})
            action=next((a for a in case['actions'] if a['at']==ordinal),{})
            for id in action.get('append',[]):append(pointer,id)
            if 'replace' in action:replace(pointer,*action['replace'])
            if 'force_bits' in action:force(pointer,(ctypes.c_double*3)(*decode(action['force_bits'])))
            pending_error=action.get('error_force_bits');ordinal+=1
            if action.get('reenter'):again(pointer)
            return int(action.get('success',True))
        def guarded(*arguments):
            try:return observe(*arguments)
            except BaseException as error:
                callback_errors.append(repr(error));return 0
        hook=hook_type(guarded)
        errors=call((ctypes.c_int64*len(case['ids']))(*case['ids']),len(case['ids']),(ctypes.c_double*3)(*decode(case['force_bits'])),hook)
        assert not callback_errors,callback_errors
        case['trace']=trace;case['host_errors']=errors
        assert ordinal==case.get('expected_contacts',len(case['ids']))
    report={'schema':1,'original_commit':LATEST,'source_sha256':{'src/Ball.cpp':digest(source),'src/Vector3d.h':digest(header)},'source_regions':[{'source':'src/Ball.cpp','start':468,'end':488,'region_sha256':digest(selected.encode())}],'scaffold_sha256':digest(path.read_bytes()),'library_sha256':digest(library.read_bytes()),'compiler':compiler,'checker_sha256':digest(Path(__file__).read_bytes()),'command':command,'cases':cases,'proof_credit':False,'scope':'Unchanged Ball::DispatchCollisions, exact channel/method/Lddd descriptor and dynamic vector/Vector3d field semantics, at a typed host event/error hook that returns normally. Hook mutation is explicit environmental input, not a substitute implementation or Blue runtime; callback exceptions invalidate this corpus. Actual PyOS/PyError unwinding/private host behavior remains unresolved. Source-shrinking-below-captured-size/dangling-owner undefined behavior is not executed. No full Evolve timing, Blue layout/ABI, platform-matrix or formal claim.'}
    save(ROOT/'verification/migration/collision_dispatch_oracle_samples.json',report)


def force_oracle(directory, source, header, integration, compiler):
    """Source Potential and overlap-force observations at a typed field boundary."""
    dependencies={"src/Ballpark.cpp":source,"src/Collision.cpp":blob(LATEST,"src/Collision.cpp"),"src/Vector3d.h":header,"src/IDstConstants.h":blob(LATEST,"src/IDstConstants.h")}
    regions=[]
    def function(path, signature):
        lines=dependencies[path].decode().splitlines(keepends=True)
        starts=[i for i,l in enumerate(lines) if l.startswith(signature)]
        if len(starts)!=1:raise ValueError('ambiguous force oracle source '+signature)
        a=starts[0];b=next(i for i in range(a+1,len(lines)) if lines[i].rstrip('\r\n')=='}')
        text=''.join(lines[a:b+1]);regions.append({'source':path,'symbol':signature,'start':a+1,'end':b+1,'sha256':digest(text.encode())});return text
    definitions=[function('src/Collision.cpp','bool Quadratic('),function('src/Collision.cpp','double CollideTwoSpheres('),function('src/Ballpark.cpp','void Ballpark::Potential('),function('src/Ballpark.cpp','Vector3d Ballpark::GotoThrust(')]
    lines=source.decode().splitlines(keepends=True)
    reset=''.join(lines[790:793])
    overlap_start=next(i for i,l in enumerate(lines) if l.strip()=='collision.normal = ballLoc.p - otherLoc.p;')
    overlap_end=next(i for i in range(overlap_start,len(lines)) if '// Now get the point of impact for both balls' in lines[i])
    overlap=''.join(lines[overlap_start:overlap_end])
    assert overlap.rstrip().endswith('}')
    for a,b in [(791,793),(overlap_start+1,overlap_end)]:regions.append({'source':'src/Ballpark.cpp','start':a,'end':b,'sha256':digest(''.join(lines[a-1:b]).encode())})
    (directory/'IDstConstants.h').write_bytes(dependencies['src/IDstConstants.h'])
    scaffold=r'''#include <cmath>
#include <cstdint>
#include <vector>
#include <cstdlib>
struct Vector3 {float x,y,z;Vector3(float,float,float);};
#include "Vector3d.h"
#include "IDstConstants.h"
#define CCP_LOG_CH(...) ((void)0)
#define CCP_LOGWARN_CH(...) ((void)0)
#define CCP_LOGERR_CH(...) ((void)0)
bool g_useDynamicalOrientation=false;
struct Ball {
 int64_t mId=0,mFollowId=0;int mMode=DSTBALL_STOP;
 bool isFree=true;Vector3d mNewPos,mNewVel,mLastG,mLastC;
 double mMass=10.,mTimeFactor=0.,mLastCollision=-1.;float mAgility=1.,mRadius=.5f,mMaxVel=10.,mSpeedFraction=1.;
 std::vector<int64_t> mCollisions;
};
struct Ballpark {double mFriction,dt;Vector3d GotoThrust(const Ball*,const Vector3d&,bool);void Potential(Ball*,Ball*,int);void EvolveStop(Ball*);void SetBallTimeFactor(Ball*);void Integrate(Vector3d&,Vector3d&,const Vector3d&,double,double,double,double);};
'''+integration+'\n'+'\n'.join(definitions)+r'''
static void read(Ball& b,const double* s,int64_t id,double mass,float agility,float radius,float maximum) {
 b.mId=id;b.mNewPos=Vector3d(s[0],s[1],s[2]);b.mNewVel=Vector3d(s[3],s[4],s[5]);
 b.mLastG=Vector3d(s[6],s[7],s[8]);b.mLastC=Vector3d(s[9],s[10],s[11]);
 b.mMass=mass;b.mAgility=agility;b.mRadius=radius;b.mMaxVel=maximum;
}
static void write(const Ball& b,double* s) {
 const Vector3d vectors[]={b.mNewPos,b.mNewVel,b.mLastG,b.mLastC};
 for(int i=0;i<4;++i) {s[i*3]=vectors[i].x;s[i*3+1]=vectors[i].y;s[i*3+2]=vectors[i].z;}
}
extern "C" void destiny_force_goto(double* result,const double* target,double friction,double dt,double mass,float agility,float maximum) {
 Ballpark park{friction,dt};Ball ball;ball.mNewPos=Vector3d(0.,1.,1.);ball.mMass=mass;ball.mAgility=agility;ball.mMaxVel=maximum;
 auto g=park.GotoThrust(&ball,Vector3d(target[0],target[1],target[2]),false);result[0]=g.x;result[1]=g.y;result[2]=g.z;
}
extern "C" int destiny_force_gradient(double* state,int64_t* contacts,double friction,double dt,double mass,float agility,float radius,float maximum) {
 Ballpark park{friction,dt};Ball balls[2];
 for(int i=0;i<2;++i) {read(balls[i],state+i*12,i+1,mass,agility,radius,maximum);park.SetBallTimeFactor(&balls[i]);}
 balls[0].mMode=DSTBALL_GOTO;
 // Selected live-Stop behavior reset; the dead global candidate skips it.
 Ball* ball=&balls[1];
'''+reset+r'''
 park.EvolveStop(ball);
 ball->mLastCollision=-1.;park.Potential(ball,&balls[0],0);
 park.Integrate(ball->mNewPos,ball->mNewVel,ball->mLastG+ball->mLastC,ball->mMass*ball->mAgility,park.mFriction,ball->mTimeFactor,park.dt);
 for(int i=0;i<2;++i)write(balls[i],state+i*12);
 for(size_t i=0;i<ball->mCollisions.size();++i)contacts[i]=ball->mCollisions[i];
 return static_cast<int>(ball->mCollisions.size());
}
extern "C" int destiny_contact_many(int64_t* contacts,int count) {
 Ballpark park{5.,.25};Ball me;me.mId=500;me.mNewPos=Vector3d(0.,1.,1.);me.mNewVel=Vector3d(500.,0.,0.);me.mMaxVel=500.;park.SetBallTimeFactor(&me);
 for(int i=0;i<count;++i) {Ball other;other.mId=i-65;if(other.mId>=-1)++other.mId;other.isFree=false;other.mNewPos=Vector3d(2.,1.,1.);park.SetBallTimeFactor(&other);park.Potential(&me,&other,0);}
 for(size_t i=0;i<me.mCollisions.size();++i)contacts[i]=me.mCollisions[i];
 return static_cast<int>(me.mCollisions.size());
}
extern "C" void destiny_force_overlap(double* state,double friction,double dt,double mass,float agility,float radius,float maximum) {
 Ballpark park{friction,dt};Ball balls[2];
 for(int i=0;i<2;++i) read(balls[i],state+i*12,2-i,mass,agility,radius,maximum);
 struct Property {Ball* m_ball;};struct Location {Vector3d p;};struct Collision {Vector3d normal;double timeOfImpact=0.;};
 Property ballProp{&balls[1]},otherProp{&balls[0]};Location ballLoc{balls[1].mNewPos},otherLoc{balls[0].mNewPos};Collision collision,otherCollision;
 const auto mFriction=park.mFriction;
'''+overlap+r'''
 for(int i=0;i<2;++i)write(balls[i],state+i*12);
}
'''
    path=directory/'force_oracle.cpp';library=directory/'liboriginal_force.so';path.write_text(scaffold)
    command=['clang++','-std=c++17','-O2','-shared','-fPIC','-fno-fast-math','-ffp-contract=off',str(path),'-o',str(library)]
    checked(command,'world-force-oracle-build',60)
    loaded=ctypes.CDLL(str(library));gradient=loaded.destiny_force_gradient;overlap_call=loaded.destiny_force_overlap
    scalar_args=[ctypes.c_double,ctypes.c_double,ctypes.c_double,ctypes.c_float,ctypes.c_float,ctypes.c_float]
    overlap_call.argtypes=[ctypes.POINTER(ctypes.c_double),*scalar_args];overlap_call.restype=None
    gradient.argtypes=[ctypes.POINTER(ctypes.c_double),ctypes.POINTER(ctypes.c_int64),*scalar_args];gradient.restype=ctypes.c_int
    goto=loaded.destiny_force_goto
    goto.argtypes=[ctypes.POINTER(ctypes.c_double),ctypes.POINTER(ctypes.c_double),ctypes.c_double,ctypes.c_double,ctypes.c_double,ctypes.c_float,ctypes.c_float];goto.restype=None
    cases=[]
    for name,parameters,position,target in [('underflow',(1e6,1.,10.,1.,.5,10.),[3.,1.,1.],[10000.,1.,1.]),('nonunit',(5.,.25,10.,1.,.5,500.),[2.,1.,1.],[10000.,1.,1.]),('oblique-3d',(5.,.25,10.,1.,.5,650.),[6./13.,1.+8./13.,1.+24./13.],[3000.,4001.,12001.])]:
        g=(ctypes.c_double*3)();goto(g,(ctypes.c_double*3)(*target),parameters[0],parameters[1],parameters[2],parameters[3],parameters[5])
        initial=[0.,1.,1.,0.,0.,0.,*g,0.,0.,0.]+[*position,0.,0.,0.,0.,0.,0.,0.,0.,0.]
        variants={};contact_variants={}
        for variant in ['retained','erased']:
            state=(ctypes.c_double*24)(*initial)
            if variant=='erased':
                for i in range(6,9):state[i]=0.
            rows=[];contact_rows=[]
            for _ in range(4):
                contacts=(ctypes.c_int64*8)();count=gradient(state,contacts,*parameters)
                assert 0<=count<=8
                rows.append([struct.pack('>d',x).hex() for x in state]);contact_rows.append(list(contacts)[:count])
            variants[variant]=rows;contact_variants[variant]=contact_rows
        overlap_initial=initial.copy();overlap_initial[12:15]=[.5,1.,1.]
        state=(ctypes.c_double*24)(*overlap_initial);overlap_call(state,*parameters)
        cases.append({'name':name,'parameters':parameters,'initial':initial,'goto':target,'variants':variants,'contact_variants':contact_variants,'overlap_initial':overlap_initial,'overlap_expected_bits':[struct.pack('>d',x).hex() for x in state]})
    many=loaded.destiny_contact_many;many.argtypes=[ctypes.POINTER(ctypes.c_int64),ctypes.c_int];many.restype=ctypes.c_int
    ids=(ctypes.c_int64*512)();count=many(ids,130);assert 0<count<=512
    report={'schema':1,'original_commit':LATEST,'compiler':compiler,'source_sha256':{p:digest(b) for p,b in dependencies.items()},'source_regions':regions,'scaffold_sha256':digest(path.read_bytes()),'library_sha256':digest(library.read_bytes()),'checker_sha256':digest(Path(__file__).read_bytes()),'command':command,'cases':cases,'scope':'Unchanged source GotoThrust/Potential/CollideTwoSpheres/Quadratic/EvolveStop/SetBallTimeFactor/Integrate and exact live force reset / time-zero two-body overlap-force fragments. Explicit fields and supplied dead/global candidate relation are the typed boundary; only the second body is evolved, as live Stop, first body retains its supplied history. No full Evolve, dynamic orientation, full iterative response/history, partition selection, Blue ABI, platform-matrix or formal claim.','proof_credit':False}
    report['many_contacts']={'count':130,'ids':list(ids)[:count],'scope':'Supplied 130 sorted static spherical candidates, including zero and negative IDs, excluding the reserved -1 root ID; unchanged Potential, no host/partition query claim.'}
    # Reuse the already source-bound angular oracle; independently execute the
    # exact aligning-warp input used by the real host tests, at both list states.
    from check_angular_migration import function as angular_function, expected as angular_expected, FIELDS
    angular_dir=ROOT/'.aegis/destiny/migration/angular-oracle'
    angular_receipt=json.loads((angular_dir/'build.json').read_text())
    angular_library=angular_dir/'liboriginal_angular.so'
    assert angular_receipt['source_sha256']==digest(blob(LATEST,'src/Ball.cpp'))
    assert angular_receipt['library_sha256']==digest(angular_library.read_bytes())
    angle_input=[13_000_000.,1.,2.,3.,0.,0.,0.,200000.,300000.,-100000.,1_000_000.,1.,1.,-.5,-3.,-.9]
    call=angular_function(angular_library)
    report['contact_angles']={'input':angle_input,'source_sha256':angular_receipt['source_sha256'],'library_sha256':angular_receipt['library_sha256'],'build_receipt_sha256':digest((angular_dir/'build.json').read_bytes()),'states':[dict(zip(FIELDS,(struct.pack('>d',v).hex() for v in angular_expected(call,[0.]*9,angle_input,3,-1,0,collided,True)))) for collided in [False,True]]}
    save(ROOT/'verification/migration/force_oracle_samples.json',report)
    return len(cases)


def main():
    directory = STATE / "stop-oracle"
    directory.mkdir(parents=True, exist_ok=True)
    source = blob(LATEST, "src/Ballpark.cpp")
    header = blob(LATEST, "src/Vector3d.h")
    lines = source.decode().splitlines(keepends=True)
    regions = [(751, 776), (1339, 1345), (4968, 4983)]
    selected = "\n".join("".join(lines[start-1:end]) for start, end in regions)
    (directory / "Vector3d.h").write_bytes(header)
    scaffold = '''#include <cmath>
struct Vector3 { float x,y,z; Vector3(float,float,float); };
#include "Vector3d.h"
bool g_useDynamicalOrientation = false;
struct Ball { Vector3d mNewVel; double mMass; float mAgility; double mTimeFactor; };
struct Ballpark {
 double mFriction, dt;
 void EvolveStop(Ball*);
 void SetBallTimeFactor(Ball*);
 void Integrate(Vector3d&,Vector3d&,const Vector3d&,double,double,double,double);
};
''' + selected + '''
extern "C" void destiny_original_stop(double* state, double mass, float agility, double friction, double dt) {
 Ballpark park{friction,dt};
 Ball ball{Vector3d(state[3],state[4],state[5]),mass,agility,0.0};
 park.SetBallTimeFactor(&ball);
 park.EvolveStop(&ball);
 Vector3d p(state[0],state[1],state[2]);
 Vector3d v=ball.mNewVel;
 park.Integrate(p,v,Vector3d(0.0,0.0,0.0),ball.mMass*ball.mAgility,park.mFriction,ball.mTimeFactor,park.dt);
 state[0]=p.x; state[1]=p.y; state[2]=p.z;
 state[3]=v.x; state[4]=v.y; state[5]=v.z;
}
'''
    path = directory / "stop_oracle.cpp"
    library = directory / "liboriginal_stop.so"
    path.write_text(scaffold)
    command = ["clang++", "-std=c++17", "-O2", "-shared", "-fPIC", "-fno-fast-math", "-ffp-contract=off", str(path), "-o", str(library)]
    checked(command, "world-stop-oracle-build", 60)
    version = checked(["clang++", "--version"], "world-stop-oracle-compiler")
    call = ctypes.CDLL(str(library)).destiny_original_stop
    call.argtypes = [ctypes.POINTER(ctypes.c_double), ctypes.c_double, ctypes.c_float, ctypes.c_double, ctypes.c_double]
    call.restype = None
    rng = random.Random(1141339)
    rows = []
    for index in range(35):
        if index == 0:
            position, velocity, mass, agility, friction, dt, maximum = [0.0]*3, [10.0,0.0,0.0], 10.0, 0.5, 5.0, 0.1, 1.0
        elif index == 1:
            position, velocity, mass, agility, friction, dt, maximum = [0.0]*3, [10.0,20.0,30.0], 13_000_000.0, 0.9, 1_000_000.0, 1.0, 10.0
        elif index == 34:
            # Exercise the original Taylor-position branch deliberately.
            position, velocity, mass, agility, friction, dt, maximum = [3.0,-4.0,5.0], [10.0,20.0,30.0], 13_000_000.0, 0.9, 0.0001, 1.0, 10.0
        else:
            position = [rng.uniform(-10_000.0,10_000.0) for _ in range(3)]
            velocity = [rng.uniform(-1500.0,1500.0) for _ in range(3)]
            mass, agility = 10.0**rng.uniform(2,8), rng.uniform(0.1,2.5)
            friction, dt, maximum = 10.0**rng.uniform(-2,8), [0.25,0.5,1.0,2.0][index%4], rng.uniform(1.0,50.0)
        state = (ctypes.c_double*6)(*position,*velocity)
        call(state,mass,agility,friction,dt)
        rows.append({"position":position,"velocity":velocity,"mass":mass,"agility":agility,
                     "friction":friction,"dt":dt,"max_velocity":maximum,
                     "expected_bits":[struct.pack(">d",value).hex() for value in state]})
    report = {"schema":1,"original_commit":LATEST,"seed":1141339,"source_sha256":digest(source),
              "header_sha256":digest(header),"source_regions":regions,"scaffold_sha256":digest(path.read_bytes()),
              "library_sha256":digest(library.read_bytes()),"compiler":version,"checker_sha256":digest(Path(__file__).read_bytes()),
              "execution_receipt":".aegis/destiny/migration/world-stop-oracle-build-execution.json",
              "execution_sha256":digest((STATE / "world-stop-oracle-build-execution.json").read_bytes()),
              "variant":"g_useDynamicalOrientation=false; source float agility and double mass/friction/dt; host binary64",
              "scope":"35 collision-free source stop/integration observations, including Taylor position and time-factor underflow; C++ source unchanged. No full original engine, Blue ABI, universal domain, collision or platform-matrix claim.",
              "rows":rows,"proof_credit":False}
    save(ROOT / "verification/migration/stop_oracle_samples.json",report)
    force_oracle(directory,source,header,selected,version)
    dispatch_oracle(directory,header,version)
    kinematic_oracle(directory,header,version)
    print(json.dumps({"status":"ORIGINAL_CPP_OBSERVATIONS","cases":len(rows),"output_lanes":len(rows)*6,"proof_credit":False}))


if __name__ == "__main__":
    main()
