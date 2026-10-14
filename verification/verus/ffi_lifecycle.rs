//! Verify the actual production machine-counter bodies; full pointer/atomic
//! ownership, mutex, callback and reclamation correspondence is separate.
use vstd::prelude::*;
use vstd::atomic::{PAtomicUsize,PermissionUsize};
#[path="../../src/ffi_lifecycle.rs"]
mod production;
mod reference_protocol;
mod ball_lifecycle;

macro_rules! verified_lifecycle_function {
    (retain_count($count:ident: usize) -> Option<usize> $body:block) => {
        verus! {
            pub fn retain_count($count:usize) -> (result:Option<usize>)
                ensures
                    ($count==0 || $count==usize::MAX) ==> result==None,
                    (0<$count && $count<usize::MAX) ==> result==Some(($count+1) as usize),
            $body
        }
    };
    (release_count($count:ident: usize) -> Option<usize> $body:block) => {
        verus! {
            pub fn release_count($count:usize) -> (result:Option<usize>)
                ensures
                    $count==0 ==> result==None,
                    0<$count ==> result==Some(($count-1) as usize),
            $body
        }
    };
}
production::ffi_lifecycle_bodies!(verified_lifecycle_function);

macro_rules! verified_reference_cas {
    ($counter:ident,$permission:ident,$expected:ident,$next:ident) => {
        verus_builtin_macros::verus_exec_expr!(
            $counter.compare_exchange_weak(Tracked(&mut *$permission),$expected,$next)
        )
    };
}
macro_rules! verified_reference_attempt {
    (retain_attempt($counter:ident,$expected:ident,$permission:ident) -> Option<Result<usize,usize>> $body:block) => {
        verus! {
            pub fn retain_attempt($counter:&PAtomicUsize,$expected:usize,Tracked($permission):Tracked<&mut PermissionUsize>)
                -> (result:Option<Result<usize,usize>>)
                requires old($permission)@.patomic==$counter.id(),
                ensures
                    final($permission)@.patomic==old($permission)@.patomic,
                    match result {
                        None => ($expected==0 || $expected==usize::MAX) && final($permission)@.value==old($permission)@.value,
                        Some(Ok(previous)) => 0<$expected && $expected<usize::MAX && previous==$expected
                            && old($permission)@.value==$expected && final($permission)@.value==$expected+1,
                        Some(Err(actual)) => 0<$expected && $expected<usize::MAX && actual==old($permission)@.value
                            && final($permission)@.value==old($permission)@.value,
                    },
            $body
        }
    };
    (release_attempt($counter:ident,$expected:ident,$permission:ident) -> Option<Result<usize,usize>> $body:block) => {
        verus! {
            pub fn release_attempt($counter:&PAtomicUsize,$expected:usize,Tracked($permission):Tracked<&mut PermissionUsize>)
                -> (result:Option<Result<usize,usize>>)
                requires old($permission)@.patomic==$counter.id(),
                ensures
                    final($permission)@.patomic==old($permission)@.patomic,
                    match result {
                        None => $expected==0 && final($permission)@.value==old($permission)@.value,
                        Some(Ok(previous)) => 0<$expected && previous==$expected && old($permission)@.value==$expected
                            && final($permission)@.value==$expected-1,
                        Some(Err(actual)) => 0<$expected && actual==old($permission)@.value
                            && final($permission)@.value==old($permission)@.value,
                    },
            $body
        }
    };
}
production::ffi_reference_attempt_bodies!(verified_reference_attempt,verified_reference_cas);

verus! {
// This adapter checks R across one actual shared CAS attempt and the logical
// token transition. A concrete atomic-invariant/Box/caller integration is
// still required; borrowing PermissionUsize here is one atomic step's scope.
pub fn coupled_retain_attempt(
    atomic:&PAtomicUsize,expected:usize,
    Tracked(permission):Tracked<&mut PermissionUsize>,
    Tracked(instance):Tracked<&reference_protocol::runtime_references::Instance>,
    Tracked(count):Tracked<&mut reference_protocol::runtime_references::count>,
    Tracked(owned):Tracked<&reference_protocol::runtime_references::references>,
) -> (result:(Option<Result<usize,usize>>,Tracked<Option<reference_protocol::runtime_references::references>>))
    requires
        old(permission)@.patomic==atomic.id(),
        old(count).instance_id()==instance.id(),
        owned.instance_id()==instance.id(),
        old(count).value()==old(permission)@.value,
    ensures
        final(permission)@.patomic==old(permission)@.patomic,
        final(count).instance_id()==instance.id(),
        final(count).value()==final(permission)@.value,
        match result.0 {
            Some(Ok(_)) => result.1@.is_some()
                && result.1@.unwrap().instance_id()==instance.id()
                && result.1@.unwrap().element()==owned.element(),
            _ => result.1@.is_none() && final(count).value()==old(count).value(),
        },
{
    let result=retain_attempt(atomic,expected,Tracked(&mut *permission));
    let tracked mut newly_owned=None;
    match result {
        Some(Ok(_)) => {
            proof { newly_owned=Some(instance.retain(owned.element(),count,owned)); }
        },
        _ => {},
    }
    (result,Tracked(newly_owned))
}

pub proof fn initialized_live_reference_is_feasible(tracked allocation: nat)
    ensures exists |state: reference_protocol::runtime_references::State|
        state.count == 1 && state.allocation == Some(allocation) && #[trigger] state.ownership(),
{
    let tracked (Tracked(instance), Tracked(count), Tracked(references)) =
        reference_protocol::runtime_references::Instance::new(allocation, Some(allocation));
    assert(count.value() == 1);
    assert(count.instance_id() == instance.id());
    let state=reference_protocol::runtime_references::State {
        allocation:Some(allocation),count:1,references:vstd::multiset::Multiset::singleton(allocation),
    };
    assert(state.ownership());
}

pub fn coupled_release_attempt(
    atomic:&PAtomicUsize,expected:usize,
    Tracked(permission):Tracked<&mut PermissionUsize>,
    Tracked(instance):Tracked<&reference_protocol::runtime_references::Instance>,
    Tracked(count):Tracked<&mut reference_protocol::runtime_references::count>,
    Tracked(owned):Tracked<reference_protocol::runtime_references::references>,
) -> (result:(Option<Result<usize,usize>>,
    Tracked<Option<reference_protocol::runtime_references::references>>,Tracked<Option<nat>>))
    requires
        old(permission)@.patomic==atomic.id(),
        old(count).instance_id()==instance.id(),
        owned.instance_id()==instance.id(),
        old(count).value()==old(permission)@.value,
    ensures
        final(permission)@.patomic==old(permission)@.patomic,
        final(count).instance_id()==instance.id(),
        final(count).value()==final(permission)@.value,
        match result.0 {
            Some(Ok(previous)) => result.1@.is_none()
                && (previous==1 ==> result.2@==Some(owned.element()) && final(count).value()==0)
                && (previous!=1 ==> result.2@.is_none() && final(count).value()>0),
            _ => result.1@.is_some() && result.1@.unwrap().instance_id()==instance.id()
                && result.1@.unwrap().element()==owned.element()
                && result.2@.is_none() && final(count).value()==old(count).value(),
        },
{
    let result=release_attempt(atomic,expected,Tracked(&mut *permission));
    let tracked remaining;
    let tracked released;
    match result {
        Some(Ok(previous)) => {
            proof {
                if previous==1 { released=Some(instance.release_final(owned.element(),count,owned)); }
                else { instance.release_nonfinal(owned.element(),count,owned);released=None; }
                remaining=None;
            }
        },
        _ => { proof { remaining=Some(owned);released=None; } },
    }
    (result,Tracked(remaining),Tracked(released))
}
} // verus!

#[path = "../../src/collision_identity.rs"]
mod identity_production;

macro_rules! verified_collision_identity {
    ($name:ident($master:ident:bool,$requested:ident:i64,$local:ident:i64,$high:ident:i64)->Option<(i64,i64,i64)> $body:block) => {
        verus! {
            pub fn $name($master:bool,$requested:i64,$local:i64,$high:i64)->(result:Option<(i64,i64,i64)>)
                ensures
                    if ($master && $requested<0) || (!$master && $requested == -1) {
                        if $high==i64::MIN {result==None}
                        else {result==Some((($high-1) as i64,$local,($high-1) as i64))}
                    } else if $master && $requested==0 {
                        if $local==i64::MIN {result==None}
                        else {result==Some((($local-1) as i64,($local-1) as i64,$high))}
                    } else {result==Some(($requested,$local,$high))},
            $body
        }
    };
}
identity_production::collision_identity_body!(verified_collision_identity);
