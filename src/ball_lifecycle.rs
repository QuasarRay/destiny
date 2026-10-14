//! Retained object state, distinct from the public ID's park membership.
//! Clear detaches without setting moribund; Remove sets moribund before either
//! delayed removal or detachment. A detached object can live only through leases.
macro_rules! ball_lifecycle_body {
    ($declare:ident) => {
        $declare! {
            transition(attached: bool, moribund: bool, references: usize, event: u8)
                -> Option<(bool, bool, usize)> {
                #[cfg(kani)]
                kani::cover!(true, "actual retained ball lifecycle transition executed");
                match event {
                    0 => {
                        if (!attached && references == 0) || references == usize::MAX {
                            None
                        } else {
                            Some((attached, moribund, references + 1))
                        }
                    }
                    1 => {
                        if references == 0 { None }
                        else { Some((attached, moribund, references - 1)) }
                    }
                    2 => {
                        if attached { Some((false, true, references)) } else { None }
                    }
                    3 => {
                        if attached { Some((false, moribund, references)) } else { None }
                    }
                    4 => {
                        if attached { Some((true, false, references)) } else { None }
                    }
                    5 => {
                        if attached { Some((true, true, references)) } else { None }
                    }
                    _ => None,
                }
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use ball_lifecycle_body;

#[cfg(not(verus_keep_ghost))]
macro_rules! rust_ball_lifecycle {
    ($name:ident($attached:ident: bool, $moribund:ident: bool, $references:ident: usize, $event:ident: u8)
        -> Option<(bool, bool, usize)> $body:block) => {
        pub(crate) fn $name($attached: bool, $moribund: bool, $references: usize, $event: u8)
            -> Option<(bool, bool, usize)> $body
    };
}
#[cfg(not(verus_keep_ghost))]
ball_lifecycle_body!(rust_ball_lifecycle);

// Ballpark::StopAllFollowers checks membership and the live follow relation
// after every callback. 1 is Stop, 2 is Goto, 3 is nonmassive missile Goto.
macro_rules! follower_action_body {
    ($declare:ident) => {
        $declare! {
            follower_action(live: bool, follows: bool, mode: u8, interactive: bool) -> u8 {
                #[cfg(kani)]
                kani::cover!(true, "actual follower removal action selected");
                if !live || !follows { 0 }
                else if mode == 5 { 3 }
                else if mode == 4 && interactive { 2 }
                else { 1 }
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use follower_action_body;
#[cfg(not(verus_keep_ghost))]
macro_rules! rust_follower_action {
    ($name:ident($live:ident: bool, $follows:ident: bool, $mode:ident: u8, $interactive:ident: bool) -> u8 $body:block) => {
        pub(crate) fn $name($live:bool, $follows:bool, $mode:u8, $interactive:bool)->u8 $body
    };
}
#[cfg(not(verus_keep_ghost))]
follower_action_body!(rust_follower_action);

macro_rules! remote_follower_body {
    ($declare:ident) => {
        $declare! {
            remote_follower(free: bool, mode: u8, target_exists: bool, same_bubble: bool) -> bool {
                #[cfg(kani)]
                kani::cover!(true,"actual remote follower selection executed");
                free && matches!(mode, 1 | 4 | 5 | 12) && (!target_exists || !same_bubble)
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use remote_follower_body;
#[cfg(not(verus_keep_ghost))]
macro_rules! rust_remote_follower {
    ($name:ident($free:ident:bool,$mode:ident:u8,$exists:ident:bool,$same:ident:bool)->bool $body:block) => {
        pub(crate) fn $name($free:bool,$mode:u8,$exists:bool,$same:bool)->bool $body
    };
}
#[cfg(not(verus_keep_ghost))]
remote_follower_body!(rust_remote_follower);
