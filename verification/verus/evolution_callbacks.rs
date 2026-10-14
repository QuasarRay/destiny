//! Check the production phase bodies and the permitted callback trace.
//! Foreign callback execution, gate/Mutex implementation, ECS membership and
//! IEEE-754 Warp reachability retain separate correspondence obligations.
use vstd::prelude::*;
#[path="../../src/evolution_phase.rs"]
mod production;
#[path="../../src/proximity.rs"]
mod proximity_production;
#[path="../../src/notification_batch.rs"]
mod batch_production;
use std::collections::VecDeque;

macro_rules! verified_phase_function {
    (begin_tick($phase:ident: u8) -> Option<u8> $body:block) => { verus! {
        pub fn begin_tick($phase:u8)->(result:Option<u8>)
            ensures result==if $phase==0 { Some(1u8) } else { None },
        $body
    }};
    (enter_callback($phase:ident: u8) -> Option<u8> $body:block) => { verus! {
        pub fn enter_callback($phase:u8)->(result:Option<u8>)
            ensures result==if $phase==1 { Some(2u8) } else { None },
        $body
    }};
    (leave_callback($phase:ident: u8) -> Option<u8> $body:block) => { verus! {
        pub fn leave_callback($phase:u8)->(result:Option<u8>)
            ensures result==if $phase==2 { Some(1u8) } else { None },
        $body
    }};
    (finish_tick($phase:ident: u8) -> Option<u8> $body:block) => { verus! {
        pub fn finish_tick($phase:u8)->(result:Option<u8>)
            ensures result==if $phase==1 { Some(0u8) } else { None },
        $body
    }};
    (add_allowed($phase:ident: u8) -> bool $body:block) => { verus! {
        pub fn add_allowed($phase:u8)->(result:bool)
            ensures result==($phase==0),
        $body
    }};
}
production::evolution_phase_bodies!(verified_phase_function);

macro_rules! verified_admission {
    ($name:ident($owner:ident:bool,$free:ident:bool,$interactive:ident:bool,
        $only:ident:bool,$master:ident:bool,$same:ident:bool,$global:ident:bool) -> bool $body:block) => {
        verus! {
            pub fn $name($owner:bool,$free:bool,$interactive:bool,$only:bool,
                $master:bool,$same:bool,$global:bool)->(result:bool)
                ensures result==if $only { $interactive }
                    else { (!$master || $same || $global) && ($owner || $free) },
            $body
        }
    };
}
proximity_production::proximity_admission_body!(verified_admission);

macro_rules! verified_notification_transition {
    ($name:ident($prior:ident:bool,$inside:ident:bool,$enabled:ident:bool) -> (bool,bool) $body:block) => {
        verus! {
            pub fn $name($prior:bool,$inside:bool,$enabled:bool)->(result:(bool,bool))
                ensures result.0==if $enabled { $inside } else { $prior },
                    result.1==($enabled && $prior!=$inside),
            $body
        }
    };
}
proximity_production::notification_transition_body!(verified_notification_transition);

macro_rules! verified_batch {
    ($name:ident<$type:ident>($queue:ident: &mut VecDeque<$element:ident>,$start:ident:usize) -> Option<VecDeque<$result:ident>> $body:block) => {
        verus! {
            pub fn $name<$type>($queue:&mut VecDeque<$element>,$start:usize)->(result:Option<VecDeque<$result>>)
                ensures match result {
                    None => $start>old($queue)@.len() && final($queue)@==old($queue)@,
                    Some(batch) => $start<=old($queue)@.len()
                        && final($queue)@==old($queue)@.subrange(0,$start as int)
                        && batch@==old($queue)@.subrange($start as int,old($queue)@.len() as int),
                },
            $body
        }
    };
}
batch_production::notification_batch_body!(verified_batch);

macro_rules! verified_prepend {
    ($name:ident<$type:ident>($queue:ident: &mut VecDeque<$element:ident>,$owned:ident: &mut VecDeque<$other:ident>) -> () $body:block) => {
        verus! {
            pub fn $name<$type>($queue:&mut VecDeque<$element>,$owned:&mut VecDeque<$other>)
                ensures final($queue)@.len()==0,
                    final($owned)@==old($queue)@ + old($owned)@,
            $body
        }
    };
}
batch_production::notification_prepend_body!(verified_prepend);

macro_rules! verified_next {
    ($name:ident<$type:ident>($queue:ident: &mut VecDeque<$element:ident>) -> Option<$result:ident> $body:block) => {
        verus! {
            pub fn $name<$type>($queue:&mut VecDeque<$element>)->(result:Option<$result>)
                ensures match result {
                    None => old($queue)@.len()==0 && final($queue)@==old($queue)@,
                    Some(value) => old($queue)@.len()>0 && value==old($queue)@[0]
                        && final($queue)@==old($queue)@[1..],
                },
            $body
        }
    };
}
batch_production::notification_next_body!(verified_next);

verus! {
// original-case: python/destiny/test/ballpark/evolve/test_add.py::TestAdd::test_balls_can_not_be_added_during_evolve
// This checks the phase part of the case's conjunction, not the entire test.
// Any number of callbacks composes these total transitions; this execution
// additionally establishes that a concrete permitted trace is feasible.
pub fn feasible_callback_rejection_trace() -> (result:bool)
    ensures !result,
{
    let initial=0u8;
    let initial_allowed=add_allowed(initial);
    assert(initial_allowed);
    let phase=begin_tick(initial).unwrap();
    let evolving_allowed=add_allowed(phase);
    assert(!evolving_allowed);
    let callback=enter_callback(phase).unwrap();
    let rejected=add_allowed(callback);
    assert(!rejected);
    let reentered=begin_tick(callback);
    assert(reentered.is_none());
    let resumed=leave_callback(callback).unwrap();
    assert(resumed==phase);
    let finished=finish_tick(resumed).unwrap();
    assert(finished==initial);
    let after_allowed=add_allowed(finished);
    assert(after_allowed);
    rejected
}

pub fn feasible_nested_read_preserves_the_owned_parent_batch() {
    let mut queue=VecDeque::<u8>::new();
    queue.push_back(10);
    queue.push_back(11);
    let parent=take_notifications(&mut queue,0).unwrap();
    assert(parent@==seq![10u8,11u8]);
    assert(queue@.len()==0);
    // A nested read appends no notifications. It obtains its own empty suffix;
    // ownership of the parent's already extracted batch stays with the caller.
    let nested=take_notifications(&mut queue,0).unwrap();
    assert(nested@.len()==0);
    assert(parent@==seq![10u8,11u8]);
    assert(queue@.len()==0);
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_remove_proximity_sensor
// Own the pass across reentry: one candidate is removed only when reached;
// dropping the remaining pass on cancellation does not execute its work.
// Live geometry and the full callback/cancellation relation remain separate.
pub fn feasible_owned_notification_continuation() {
    let mut candidates=VecDeque::<u64>::new();
    candidates.push_back(4); candidates.push_back(5);
    let first=take_next(&mut candidates).unwrap();
    assert(first==4);
    assert(candidates@==seq![5u64]);
    let mut nested=VecDeque::<u64>::new();
    let read_work=take_next(&mut nested);
    assert(read_work.is_none());
    assert(candidates@==seq![5u64]);
    // The continuing path consumes the remaining original candidate once.
    let second=take_next(&mut candidates).unwrap();
    assert(second==5);
    let finished=take_next(&mut candidates);
    assert(finished.is_none());
    assert(candidates@.len()==0);
}
}
