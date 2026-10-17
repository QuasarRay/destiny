#!/usr/bin/env python3
"""Unchanged Original spherical partition ownership oracle, without Blue.

Compile original headers, allocator, boxes, spherical membership and master
insertion. The scaffold supplies only selected Ball fields and observations;
AddToBubble is an observed external boundary, not a dictionary/Blue oracle.
Only null box objects are admitted. No production C++ dependency is introduced.
"""
import ast
import ctypes
import hashlib
import json
from pathlib import Path
import random
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from tools.migrate_destiny import LATEST, STATE, blob, checked, save


def digest(data):
    return hashlib.sha256(data).hexdigest()


def liveness_oracle(directory, sources, function, regions):
    """Execute unchanged membership methods with real CPython containers/GIL."""
    import sysconfig
    first_region = len(regions)
    symbols = ["bool Ballpark::InDeadBubble(", "size_t Ballpark::GetInteractiveCnt(",
               "void Ballpark::SetBallInteractive(", "void Ballpark::IncreaseInteractiveCnt(",
               "void Ballpark::DecreaseInteractiveCnt(", "void Ballpark::UpdateInteractiveCnt(",
               "void Ballpark::UpdateKeepAliveBalls(", "void Ballpark::AddKeepAliveBall(",
               "void Ballpark::RemoveKeepAliveBall(", "void Ballpark::ClearBubbles()"]
    definitions = [function("src/Ballpark.cpp", symbol) for symbol in symbols]
    scaffold = r'''#include <Python.h>
#include <cstdint>
#include <map>
#include <utility>
using ID = int64_t;
constexpr int DSTBALL_WARP=3;
#define CCP_LOGWARN(...) ((void)0)
template<class F> struct ScopeExit {F action; ~ScopeExit(){action();}};
#define ON_BLOCK_EXIT(action) ScopeExit source_scope_exit{action}
struct Partitionable {ID mId=0; long mNewBubble=-1,mOldBubble=-1; bool isInteractive=false;};
struct Ball:Partitionable {int mMode=0;};
struct Ballpark {
 bool isMaster=false;
 PyObject *bubbles,*bubbleInteractives,*bubbleKeepAlives,*bubbleKeepAliveBalls;
 std::map<ID,Ball*> mBalls;
 bool InDeadBubble(Ball*);size_t GetInteractiveCnt(Ball*);
 void SetBallInteractive(const ID&,bool);
 void IncreaseInteractiveCnt(Partitionable*,long);void DecreaseInteractiveCnt(Partitionable*,long);
 void UpdateInteractiveCnt(Partitionable*,long,long);void UpdateKeepAliveBalls(Partitionable*,long,long);
 void AddKeepAliveBall(PyObject*,long);void RemoveKeepAliveBall(PyObject*,long);void ClearBubbles();
};
''' + "\n".join(definitions) + r'''
struct Oracle {Ballpark park;std::map<ID,Ball> balls;};
extern "C" void* destiny_liveness_new(int master,PyObject* b,PyObject* i,PyObject* k,PyObject* r) {
 auto* o=new Oracle;o->park.isMaster=master;o->park.bubbles=b;o->park.bubbleInteractives=i;
 o->park.bubbleKeepAlives=k;o->park.bubbleKeepAliveBalls=r;return o;
}
extern "C" void destiny_liveness_delete(void* raw) {delete static_cast<Oracle*>(raw);}
extern "C" void destiny_liveness_action(void* raw,int op,ID id,long old_bubble,long new_bubble,int flag) {
 auto& o=*static_cast<Oracle*>(raw);auto& p=o.park;
 if(op==0) {auto& b=o.balls[id];b.mId=id;b.mOldBubble=old_bubble;b.mNewBubble=new_bubble;b.isInteractive=flag;p.mBalls[id]=&b;}
 else if(op==1) p.SetBallInteractive(id,flag);
 else if(op==2) {auto& b=o.balls.at(id);b.mOldBubble=old_bubble;b.mNewBubble=new_bubble;p.UpdateInteractiveCnt(&b,old_bubble,new_bubble);p.UpdateKeepAliveBalls(&b,old_bubble,new_bubble);}
 else if(op==3 || op==4) {auto* key=PyLong_FromLongLong(id);if(op==3)p.AddKeepAliveBall(key,new_bubble);else p.RemoveKeepAliveBall(key,new_bubble);Py_DECREF(key);}
 else if(op==5) p.ClearBubbles();
}
extern "C" int destiny_liveness_query(void* raw,long bubble,int mode,int count) {
 auto& o=*static_cast<Oracle*>(raw);Ball b;b.mNewBubble=bubble;b.mMode=mode;
 return count ? static_cast<int>(o.park.GetInteractiveCnt(&b)) : o.park.InDeadBubble(&b);
}
'''
    path = directory / "liveness_oracle.cpp"
    library = directory / "liboriginal_liveness.so"
    path.write_text(scaffold)
    include = Path(sysconfig.get_path("include"))
    libdir = Path(sysconfig.get_config_var("LIBDIR"))
    libname = sysconfig.get_config_var("LDLIBRARY")
    libpath = libdir / libname
    command = ["clang++", "-std=c++20", "-shared", "-fPIC", "-O0", "-I"+str(include), str(path), str(libpath), "-o", str(library)]
    checked(command, "world-liveness-oracle-build", 60)
    loaded = ctypes.PyDLL(str(library))  # Python APIs require the caller's GIL.
    create, action, query, dispose = (getattr(loaded,"destiny_liveness_"+name) for name in ["new","action","query","delete"])
    create.argtypes, create.restype = [ctypes.c_int]+[ctypes.py_object]*4, ctypes.c_void_p
    action.argtypes, action.restype = [ctypes.c_void_p,ctypes.c_int,ctypes.c_int64,ctypes.c_long,ctypes.c_long,ctypes.c_int], None
    query.argtypes, query.restype = [ctypes.c_void_p,ctypes.c_long,ctypes.c_int,ctypes.c_int], ctypes.c_int
    dispose.argtypes, dispose.restype = [ctypes.c_void_p], None
    steps = [
        ["seed",1,-1,7,False],["seed",2,-1,7,True],
        ["register",1,True],["transition",1,-1,7],["transition",2,-1,7],
        ["register",1,False],["transition",1,7,9],
        ["interactive",2,False],["add",99,7],["remove",1,7],
        ["register",1,True],["transition",1,9,9],["transition",1,9,7],
        ["transition",1,7,7],["interactive",1,True],["interactive",1,True],
        ["set_interactive",7,[]],["interactive",1,True],["interactive",1,False],
        ["interactive",1,True],["set_keep_alive",-1,[2**40+7]],
        ["set_interactive",-1,[2**40+9]],["set_keep_alive",9,[]],
        ["seed",2**40+7,-1,2**35+3,True],["register",2**40+7,True],
        ["transition",2**40+7,-1,2**35+3],["transition",2**40+7,2**35+3,-1],
        ["remove",99,7],["clear"],["interactive",1,True],
        ["register",1,True],["transition",1,7,7],["transition",1,7,9],
        ["set_keep_alive",7,[1000+i for i in range(100)]],
        ["set_interactive",9,[2000+i for i in range(100)]],
    ]
    scenarios=[]
    for master in [False,True]:
        bubbles,interactives,keepers,registered={},{},{},set()
        handle=create(master,bubbles,interactives,keepers,registered)
        rows=[]
        try:
            for step in steps:
                op,*args=step
                if op=="register":
                    ident,enabled=args
                    (registered.add if enabled else registered.discard)(ident)
                elif op=="set_interactive": interactives[args[0]]=set(args[1])
                elif op=="set_keep_alive": keepers[args[0]]=set(args[1])
                else:
                    codes={"seed":0,"interactive":1,"transition":2,"add":3,"remove":4,"clear":5}
                    ident=args[0] if args else 0
                    old,new,flag=0,0,False
                    if op=="seed": old,new,flag=args[1:]
                    elif op=="interactive": flag=args[1]
                    elif op=="transition": old,new=args[1:]
                    elif op in {"add","remove"}: new=args[1]
                    action(handle,codes[op],ident,old,new,flag)
                canonical=lambda groups: [[key,sorted(value)] for key,value in sorted(groups.items()) if value]
                observations=[]
                for bubble in [-1,7,9,2**35+3]:
                    for mode in range(13):
                        observations.append([bubble,mode,bool(query(handle,bubble,mode,0)),query(handle,bubble,mode,1)])
                rows.append({"action":step,"expected":{"registered":sorted(registered),"keep_alive":canonical(keepers),"interactives":canonical(interactives),"queries":observations}})
        finally: dispose(handle)
        scenarios.append({"master":master,"rows":rows})
    environment={"python":sys.version,"executable":str(Path(sys.executable).resolve()),
                 "external_sha256":{str(p.resolve()):digest(p.read_bytes()) for p in [Path(sys.executable),libpath,*sorted(include.rglob("*.h"))]},
                 "config":{"include":str(include),"libdir":str(libdir),"ldlibrary":libname,"soabi":sysconfig.get_config_var("SOABI")}}
    report={"schema":1,"original_commit":LATEST,"scenarios":scenarios,"command":command,
            "checker_sha256":digest(Path(__file__).read_bytes()),"source_regions":regions[first_region:],
            "source_sha256":{"src/Ballpark.cpp":digest(sources["src/Ballpark.cpp"])},
            "scaffold_sha256":digest(path.read_bytes()),"library_sha256":digest(library.read_bytes()),"environment":environment,
            "scope":"Unchanged InDeadBubble/GetInteractiveCnt/interactive and keep-alive methods/ClearBubbles with real CPython integer/dictionary/set behavior on valid typed values. Queries cover all 13 modes, client/master and sentinel/large bubble IDs; empty-map keys are canonicalized only for comparisons. Field declarations and scope-exit cleanup are scaffold adapters. Transition composes the two unchanged methods in source order; full AddToBubble, full Evolve, malformed/aliased host objects, Blue ABI, platform matrix and formal correspondence remain open.","proof_credit":False}
    save(ROOT/"verification/migration/liveness_oracle_samples.json",report)
    return sum(len(s["rows"]) for s in scenarios)


def main():
    directory = STATE / "partition-oracle"
    directory.mkdir(parents=True, exist_ok=True)
    paths = ["src/Partition.h", "src/Box.h", "src/SortedSets.h", "src/Vector3d.h", "src/Partitionable.h"]
    sources = {path: blob(LATEST, path) for path in paths + ["src/Partition.cpp", "src/Box.cpp", "src/Ball.cpp", "src/Partitionable.cpp", "src/Ballpark.cpp", "src/Ballpark.h", "src/Ball.h"]}
    for path in paths:
        (directory / Path(path).name).write_bytes(sources[path])
    regions = []

    def function(path, signature):
        lines = sources[path].decode().splitlines(keepends=True)
        starts = [i for i, line in enumerate(lines) if line.startswith(signature)]
        if len(starts) != 1:
            raise ValueError(f"ambiguous/missing source definition: {signature}")
        start = starts[0]
        end = next(i for i in range(start + 1, len(lines)) if lines[i].strip("\r\n") in {"}", "{}"})
        selected = "".join(lines[start:end + 1])
        regions.append({"source": path, "symbol": signature, "start": start + 1, "end": end + 1, "sha256": digest(selected.encode())})
        return selected

    definitions = []
    for path, signatures in [
        ("src/Partition.cpp", ["Partition::Partition()", "void Partition::GetBoxKey(", "Box* Partition::GetBox(int64_t", "Box* Partition::CheckBox(", "Box* Partition::GetBox(double", "Box* Partition::GetExistingTopBox(", "Box* Partition::GetTopmost(", "Box* Partition::CheckOffsetBox(", "BoxAllocator::BoxAllocator()", "BoxAllocator::~BoxAllocator()", "Box * BoxAllocator::GetNewBox(", "void BoxAllocator::ReleaseBox("]),
        ("src/Box.cpp", ["Box::Box(", "Box::~Box()", "void Box::ReleaseBox()", "void Box::SetParent()", "void Box::AddChildren(", "void Box::AddBall(", "void Box::RemoveChildren(", "void Box::RemoveBall(", "long Box::NearbyBubbles()", "bool BoxSortComparer::operator"]),
        ("src/Partitionable.cpp", ["Partitionable::Partitionable()"]),
        ("src/Ball.cpp", ["void Ball::ClearBoxes()", "void Ball::DeleteFromBoxes()", "void Ball::InsertInBox(", "double Ball::GetInflatedRadius(", "float Ball::GetBoundingRadius()", "void Ball::SetBoxActive(", "void Ball::InsertInBoxes(", "bool BallSortComparer::operator"]),
        ("src/Ballpark.cpp", ["void Ballpark::InsertInBoxes(", "void Ballpark::Integrate(", "void Ballpark::EvolveStop(", "void Ballpark::SetBallTimeFactor("]),
    ]:
        definitions.extend(function(path, signature) for signature in signatures)
    # Null-only object boundary, checked rather than pretending to decrement a
    # Python object. Logging has no selected world-state effect.
    evolve_lines = sources["src/Ballpark.cpp"].decode().splitlines(keepends=True)
    def phase(first, last):
        starts=[i for i,line in enumerate(evolve_lines) if first in line]
        if len(starts)!=1: raise ValueError("ambiguous phase start: "+first)
        start=starts[0];end=next(i for i in range(start,len(evolve_lines)) if last in evolve_lines[i])
        selected="".join(evolve_lines[start:end+1])
        regions.append({"source":"src/Ballpark.cpp","symbol":"Evolve phase: "+first,"start":start+1,"end":end+1,"sha256":digest(selected.encode())})
        return selected
    swap = phase("std::swap( ball->mNewPos, ball->mOldPos )", "std::swap( ball->mNewVel, ball->mOldVel )")
    insert = phase("Vector3d delta = ball->mNewPos - ball->mOldPos;", "InsertBallInBoxes(ball);")
    scaffold = r'''#include <cmath>
#include <cstdint>
#include <cstring>
#include <cstdlib>
#include <memory>
#include <sstream>
#include <iomanip>
#include <algorithm>
#include <map>
struct PyObject;
static void Py_XDECREF(PyObject* object) { if(object) std::abort(); }
#define CCP_LOGERR_CH(...) ((void)0)
#define CCP_LOGWARN_CH(...) ((void)0)
struct Vector3 {float x,y,z; Vector3(float,float,float);};
#include "Box.h"
#include "Partitionable.h"
struct Ball : Partitionable {
 ID mOwnerId=0; bool isFree=false; float mRadius=0,mAgility=0,mMaxVel=0,mSpeedFraction=0;
 double mMass=0,mTimeFactor=0; Vector3d mNewVel,mOldVel; short mMod[3]; bool mActiveBoxes[27];
 void ClearBoxes() override; void DeleteFromBoxes() override; void InsertInBox(Box*) override;
 float GetBoundingRadius() override; double GetInflatedRadius(double) override;
 void SetBoxActive(int,bool) override; void InsertInBoxes(Box*,Box*,long) override;
};
struct Ballpark {
 Partition* mPartition; double mFriction=1000000.0,dt=1.0; bool isMaster; ID mProbe=-999999;
 long mCurrentTime=0,mBubbleId=0; std::vector<ID> events;
 void AddToBubble(Partitionable* p) { events.push_back(p->mId); }
 void InsertInBoxes(Partitionable*);
 void InsertBallInBoxes(Ball* ball) {InsertInBoxes(ball);}
 void Integrate(Vector3d&,Vector3d&,const Vector3d&,double,double,double,double);
 void EvolveStop(Ball*);void SetBallTimeFactor(Ball*);
 void PublishSourceMembership(const std::vector<Ball*>& balls) {
  for(auto ball:balls) {
''' + swap + insert + r'''
  }
 }
};
bool g_useDynamicalOrientation=false;
''' + "\n".join(definitions) + r'''
struct Oracle {
 Partition partition; Ballpark park; std::map<ID,std::unique_ptr<Ball>> balls; bool observe_motion=false;
 explicit Oracle(bool master):park{&partition,1000000.0,1.0,master} {}
 ~Oracle() {for(auto& entry:balls) entry.second->DeleteFromBoxes();}
};
static std::string output;
extern "C" void* destiny_partition_new(int master) {return new Oracle(master!=0);}
extern "C" void destiny_partition_delete(void* raw) {delete static_cast<Oracle*>(raw);}
extern "C" void destiny_partition_action(void* raw,int op,int64_t id,const double* values) {
 auto& oracle=*static_cast<Oracle*>(raw); auto& park=oracle.park;
 if(op==0) {
  auto ball=std::make_unique<Ball>();ball->ClearBoxes();ball->mId=id;ball->mPark=&park;
  ball->mNewPos=Vector3d(values[0],values[1],values[2]);park.InsertInBoxes(ball.get());oracle.balls.emplace(id,std::move(ball));
 } else if(op==4) {auto& ball=oracle.balls.at(id);ball->DeleteFromBoxes();oracle.balls.erase(id);}
 else if(op==5) {for(auto& entry:oracle.balls) entry.second->DeleteFromBoxes();oracle.balls.clear();}
 else if(op==6) {park.mFriction=values[0];park.dt=values[1];oracle.observe_motion=true;}
 else if(op==9) {
  // Restricted collision-free Stop path. No master dead-bubble, callback,
  // angular, collision, warp or mushroom behavior is replaced by this scaffold.
  if(park.isMaster) std::abort();
  std::vector<Ball*> phaseBalls;
  for(auto& entry:oracle.balls) {
   auto ball=entry.second.get();if(!ball->isFree)continue;
   park.EvolveStop(ball);ball->mOldPos=ball->mNewPos;ball->mOldVel=ball->mNewVel;
   park.Integrate(ball->mOldPos,ball->mOldVel,Vector3d(0.0,0.0,0.0),ball->mMass*ball->mAgility,park.mFriction,ball->mTimeFactor,park.dt);
   phaseBalls.push_back(ball);
  }
  park.PublishSourceMembership(phaseBalls);
 }
 else {
  auto& ball=*oracle.balls.at(id);
  if(op==1) {ball.mNewPos=Vector3d(values[0],values[1],values[2]);park.InsertInBoxes(&ball);}
  if(op==2) {ball.mRadius=float(values[0]);park.InsertInBoxes(&ball);}
  if(op==3) {ball.mNewVel=Vector3d(values[0],values[1],values[2]);}
  if(op==7) {ball.mMass=values[0];ball.mAgility=float(values[1]);ball.mSpeedFraction=float(values[2]);park.SetBallTimeFactor(&ball);ball.isFree=true;}
  if(op==8) {ball.mMaxVel=float(values[0]);park.InsertInBoxes(&ball);}
 }
}
extern "C" const char* destiny_partition_state(void* raw) {
 auto& oracle=*static_cast<Oracle*>(raw);std::ostringstream stream;
 stream<<"{\"next_bubble\":"<<oracle.park.mBubbleId<<",\"boxes\":[";
 bool first=true;
 // Canonical observation order only: this does not claim to reproduce the
 // native unordered-map order of GetActiveBoxes.
 for(int level=0;level<8;++level) {
  std::vector<Box*> boxes;for(auto entry:oracle.partition.mLevels[level]) boxes.push_back(entry.second);
  std::sort(boxes.begin(),boxes.end(),BoxSortComparer{});
  for(auto box:boxes) {if(!first)stream<<",";first=false;
   stream<<"["<<level<<","<<box->mKey.ix<<","<<box->mKey.iy<<","<<box->mKey.iz<<","<<box->mBubble<<"]";
  }
 }
 stream<<"],\"balls\":[";first=true;
 for(auto& entry:oracle.balls) {auto& ball=*entry.second;if(!first)stream<<",";first=false;
  stream<<"{\"id\":"<<ball.mId<<",\"bubble\":"<<ball.mNewBubble<<",\"owned\":[";
  bool firstbox=true;for(auto box:ball.mBoxes) {if(!firstbox)stream<<",";firstbox=false;
   stream<<"["<<box->mLevel<<","<<box->mKey.ix<<","<<box->mKey.iy<<","<<box->mKey.iz<<"]";}
  auto box=ball.mMyBox;stream<<"],\"primary\":["<<box->mLevel<<","<<box->mKey.ix<<","<<box->mKey.iy<<","<<box->mKey.iz<<"]}";
 }
 stream<<"]";
 if(oracle.observe_motion) {
  auto bits=[](double value) {uint64_t word;std::memcpy(&word,&value,sizeof(word));std::ostringstream text;text<<std::hex<<std::setw(16)<<std::setfill('0')<<word;return text.str();};
  stream<<",\"motion\":[";first=true;
  for(auto& entry:oracle.balls) {auto& ball=*entry.second;if(!first)stream<<",";first=false;
   stream<<"{\"id\":"<<ball.mId<<",\"position_bits\":[\""<<bits(ball.mNewPos.x)<<"\",\""<<bits(ball.mNewPos.y)<<"\",\""<<bits(ball.mNewPos.z)<<"\"],\"velocity_bits\":[\""<<bits(ball.mNewVel.x)<<"\",\""<<bits(ball.mNewVel.y)<<"\",\""<<bits(ball.mNewVel.z)<<"\"]}";
  }
  stream<<"]";
 }
 stream<<"}";output=stream.str();return output.c_str();
}
'''
    path, library = directory / "partition_oracle.cpp", directory / "liboriginal_partition.so"
    path.write_text(scaffold)
    command = ["clang++", "-std=c++17", "-O2", "-shared", "-fPIC", "-fno-fast-math", "-ffp-contract=off", str(path), "-o", str(library)]
    checked(command, "world-partition-oracle-build", 60)
    loaded = ctypes.CDLL(str(library))
    create, action, state, dispose = (getattr(loaded, "destiny_partition_" + name) for name in ["new", "action", "state", "delete"])
    create.argtypes, create.restype = [ctypes.c_int], ctypes.c_void_p
    action.argtypes, action.restype = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int64, ctypes.POINTER(ctypes.c_double)], None
    state.argtypes, state.restype = [ctypes.c_void_p], ctypes.c_char_p
    dispose.argtypes, dispose.restype = [ctypes.c_void_p], None
    rng = random.Random(1142354)
    steps = []
    for ident, position in [(1,[1.,1.,1.]), (2,[481.,1.,1.]), (3,[1.,1.,1.]), (4,[0.,0.,0.])]:
        steps.append([0,ident,position])
    steps += [[1,1,[481.,-1.,-481.]], [2,2,[160.]], [2,2,[160.00001525878906]], [3,3,[1e9,-1e8,0.]], [1,4,[100.,100.,100.]], [2,1,[5e6]], [1,1,[4e7,0.,0.]], [4,2,[]], [4,3,[]], [4,4,[]], [4,1,[]], [0,1,[1e9,0.,0.]], [5,0,[]], [0,1,[1e9,0.,0.]]]
    for ident in range(2,22):
        steps.append([0,ident,[float(rng.randrange(-8,9)*7864320+1) for _ in range(3)]])
    steps += [[2,7,[0.]], [1,7,[-1.,-1.,-1.]], [4,7,[]]]
    scenarios=[]
    for master in [False,True]:
        handle=create(master)
        rows=[]
        try:
            for op,ident,values in steps:
                action(handle,op,ident,(ctypes.c_double*3)(*(values+[0.]*(3-len(values)))))
                rows.append({"action":[op,ident,values],"expected":json.loads(state(handle))})
        finally:
            dispose(handle)
        scenarios.append({"master":master,"rows":rows})
    dynamic=[]
    for friction,dt,velocity,maximum in [(5.,.125,[.125,0.,0.],0.),(5.,.25,[2.,3.,-4.],500.),(1e6,.5,[500.,-2.,1.],1000.),(.125,2.,[100.,0.,0.],200.)]:
        handle=create(False);rows=[]
        actions=[[6,0,[friction,dt]],[0,1,[479.,1.,1.]],[7,1,[13e6,.9,.95]],[3,1,velocity],[8,1,[maximum]],[2,1,[2.]]]+[[9,0,[]] for _ in range(12)]+[[3,1,[10000.,0.,0.]],[1,1,[960.,1.,1.]]]
        try:
            for op,ident,values in actions:
                action(handle,op,ident,(ctypes.c_double*3)(*(values+[0.]*(3-len(values)))))
                rows.append({"action":[op,ident,values],"expected":json.loads(state(handle))})
        finally: dispose(handle)
        dynamic.append({"master":False,"friction":friction,"dt":dt,"rows":rows})
    fixture = blob(LATEST,"python/destiny/test/ballpark/test_boxes_and_bubbles.py")
    tree=ast.parse(fixture)
    selected=next(node for node in tree.body if isinstance(node,ast.ClassDef) and node.name=="TestBallBox")
    fixture_cases=[{"id":"python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestBallBox::"+node.name,"source_assertions":[ast.get_source_segment(fixture.decode(),stmt) for stmt in node.body if isinstance(stmt,ast.Expr) and isinstance(stmt.value,ast.Call) and isinstance(stmt.value.func,ast.Attribute) and stmt.value.func.attr.startswith("assert")]} for node in selected.body if isinstance(node,ast.FunctionDef)]
    report={"schema":1,"original_commit":LATEST,"seed":1142354,"source_sha256":{p:digest(v) for p,v in sources.items()},"source_regions":regions,"scaffold_sha256":digest(path.read_bytes()),"library_sha256":digest(library.read_bytes()),"compiler":checked(["clang++","--version"],"world-partition-oracle-compiler"),"command":command,"checker_sha256":digest(Path(__file__).read_bytes()),"scenarios":scenarios,"fixture_cases":fixture_cases,"fixture_sha256":digest(fixture),"scope":"Fixed spherical objects, unchanged original allocator/parent/shared-box removal, radius/position/velocity query state, master adjacency/counter, and clear-owner sequences. Sorted observations compare state, not native active-box iteration order. AddToBubble recorded only as an external call. No object/Blue ABI, dynamic integration, static geometry, full-engine, proof or platform claim.","proof_credit":False}
    save(ROOT/"verification/migration/partition_oracle_samples.json",report)
    dynamic_report={**report,"scenarios":dynamic,"scope":"Collision-free client Stop numeric integration and the unchanged source position/velocity swap/displacement-threshold membership phase, with source nonunit-dt inflated radius and stale velocity-setter ownership. No full Evolve, angular, dead-bubble, collision/callback/master-dynamic, object/Blue, proof or platform claim."}
    save(ROOT/"verification/migration/partition_dynamic_oracle_samples.json",dynamic_report)
    liveness_rows=liveness_oracle(directory,sources,function,regions)
    print(json.dumps({"liveness_rows":liveness_rows,"status":"ORIGINAL_CPP_OBSERVATIONS","scenarios":len(scenarios),"transitions":sum(len(s['rows']) for s in scenarios),"original_assertions":sum(len(c['source_assertions']) for c in fixture_cases)}))


if __name__ == "__main__":
    main()
