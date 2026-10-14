// Copyright © 2015, 2023 CCP ehf. Original Destiny MIT license retained.
//! Original GetIDForCollisionObject machine-counter policy. The source counters
//! start at -1 and -(1<<30). Checked rejection replaces signed decrement UB;
//! allocation-map uniqueness, registration and caller ownership remain required.
macro_rules! collision_identity_body {
    ($declare:ident) => {
        $declare!(allocate_collision_id(master:bool,requested:i64,local:i64,high:i64)->Option<(i64,i64,i64)> {
            if master && requested<0 || !master && requested == -1 {
                if high==i64::MIN {None} else {
                    let next=high-1;Some((next,local,next))
                }
            } else if master && requested==0 {
                if local==i64::MIN {None} else {
                    let next=local-1;Some((next,next,high))
                }
            } else {Some((requested,local,high))}
        });
    };
}
#[cfg(not(verus_keep_ghost))]
macro_rules! ordinary_identity {
    ($name:ident($master:ident:bool,$requested:ident:i64,$local:ident:i64,$high:ident:i64)->$result:ty $body:block) => {
        pub(crate) fn $name($master: bool, $requested: i64, $local: i64, $high: i64) -> $result {
            #[cfg(kani)]
            kani::cover!(true, "actual production collision identity policy executed");
            $body
        }
    };
}
#[cfg(not(verus_keep_ghost))]
collision_identity_body!(ordinary_identity);
#[cfg(verus_keep_ghost)]
pub(crate) use collision_identity_body;
