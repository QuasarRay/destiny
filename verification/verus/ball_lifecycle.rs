//! Checked body shared with the production retained-object registry. This is
//! a kernel contract, not a proof of the registry, Python or allocator callers.
use vstd::prelude::*;
#[path = "../../src/ball_lifecycle.rs"]
mod production;
#[path = "../../src/missile.rs"]
mod missile_production;
#[path="../../src/harmonic.rs"]
mod harmonic_production;

macro_rules! verified_harmonic_action {
    ($name:ident($mode:ident:u8,$field:ident:bool)->u8 $body:block)=>{
        verus! {
            pub fn $name($mode:u8,$field:bool)->(result:u8)
                ensures result==if $field {10u8} else if $mode==10 {2u8} else {255u8},
            $body
        }
    };
}
harmonic_production::harmonic_action_body!(verified_harmonic_action);

macro_rules! verified_missile_admission {
    ($name:ident($src:ident:i64,$target:ident:i64,$owner:ident:i64,$src_exists:ident:bool,$target_exists:ident:bool,$moribund:ident:bool)->bool $body:block) => {
        verus! {
            pub fn $name($src:i64,$target:i64,$owner:i64,$src_exists:bool,$target_exists:bool,$moribund:bool)->(result:bool)
                ensures result == ($src_exists && $target_exists && !$moribund && $src != $target && $src != $owner && $target != $owner),
            $body
        }
    };
}
missile_production::missile_admission_body!(verified_missile_admission);

macro_rules! verified_ball_lifecycle {
    ($name:ident($attached:ident: bool, $moribund:ident: bool, $references:ident: usize, $event:ident: u8)
        -> Option<(bool, bool, usize)> $body:block) => {
        verus! {
            pub fn $name($attached:bool, $moribund:bool, $references:usize, $event:u8)
                -> (result:Option<(bool, bool, usize)>)
                ensures
                    result.is_some() == match $event {
                        0 => ($attached || $references > 0) && $references < usize::MAX,
                        1 => $references > 0,
                        2..=5 => $attached,
                        _ => false,
                    },
                    result.is_some() ==> {
                        let next = result.unwrap();
                        &&& next.0 == ($attached && $event != 2 && $event != 3)
                        &&& next.1 == ($event == 2 || $event == 5 || ($event != 4 && $moribund))
                        &&& next.2 as int == $references as int + (if $event == 0 { 1int } else if $event == 1 { -1int } else { 0int })
                    },
            $body
        }
    };
}
production::ball_lifecycle_body!(verified_ball_lifecycle);

macro_rules! verified_follower_action {
    ($name:ident($live:ident: bool, $follows:ident: bool, $mode:ident: u8, $interactive:ident: bool) -> u8 $body:block) => {
        verus! {
            pub fn $name($live:bool, $follows:bool, $mode:u8, $interactive:bool)->(result:u8)
                ensures result == if !$live || !$follows { 0u8 }
                    else if $mode == 5 { 3u8 }
                    else if $mode == 4 && $interactive { 2u8 }
                    else { 1u8 },
            $body
        }
    };
}
production::follower_action_body!(verified_follower_action);

macro_rules! verified_remote_follower {
    ($name:ident($free:ident:bool,$mode:ident:u8,$exists:ident:bool,$same:ident:bool)->bool $body:block) => {
        verus! {
            pub fn $name($free:bool,$mode:u8,$exists:bool,$same:bool)->(result:bool)
                ensures result==($free && ($mode==1 || $mode==4 || $mode==5 || $mode==12) && (!$exists || !$same)),
            $body
        }
    };
}
production::remote_follower_body!(verified_remote_follower);

verus! {
pub fn feasible_follower_reentry_and_special_modes() {
    let stop = follower_action(true, true, 1, false);
    let retargeted = follower_action(true, false, 1, false);
    let absent = follower_action(false, true, 5, true);
    let missile = follower_action(true, true, 5, false);
    let orbit = follower_action(true, true, 4, true);
    let passive_orbit = follower_action(true, true, 4, false);
    assert((stop, retargeted, absent, missile, orbit, passive_orbit) == (1u8, 0u8, 0u8, 3u8, 2u8, 1u8));
}
pub fn feasible_retained_remove_alias() {
    let first = transition(true, false, 0, 0).unwrap();
    let alias = transition(first.0, first.1, first.2, 0).unwrap();
    let removed = transition(alias.0, alias.1, alias.2, 2).unwrap();
    assert(removed == (false, true, 2usize));
    let copied = transition(removed.0, removed.1, removed.2, 0).unwrap();
    assert(copied == (false, true, 3usize));
    let release_one = transition(copied.0, copied.1, copied.2, 1).unwrap();
    let release_two = transition(release_one.0, release_one.1, release_one.2, 1).unwrap();
    let last = transition(release_two.0, release_two.1, release_two.2, 1).unwrap();
    assert(last == (false, true, 0usize));
    let dead_retain = transition(last.0, last.1, last.2, 0);
    assert(dead_retain.is_none());
}

pub fn feasible_clear_and_delayed_repair() {
    let clear = transition(true, false, 1, 3);
    assert(clear == Some((false, false, 1usize)));
    let delayed = transition(true, false, 1, 5).unwrap();
    assert(delayed == (true, true, 1usize));
    let repair = transition(delayed.0, delayed.1, delayed.2, 4);
    assert(repair == Some((true, false, 1usize)));
}
}
