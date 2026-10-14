//! Extract only the notifications produced by one operation. Std VecDeque
//! allocation/element ownership is a separately recorded primitive contract.
use std::collections::VecDeque;

macro_rules! notification_batch_body {
    ($declare:ident) => {
        $declare!(take_notifications<T>(queue: &mut VecDeque<T>, start: usize) -> Option<VecDeque<T>> {
            if start > queue.len() { None } else { Some(queue.split_off(start)) }
        });
    };
}
macro_rules! notification_next_body {
    ($declare:ident) => {
        $declare!(take_next<T>(queue: &mut VecDeque<T>) -> Option<T> {
            queue.pop_front()
        });
    };
}
#[cfg(not(verus_keep_ghost))]
macro_rules! ordinary_batch {
    ($name:ident<$type:ident>($queue:ident: &mut VecDeque<$element:ident>,$start:ident:usize) -> Option<VecDeque<$result:ident>> $body:block) => {
        pub(crate) fn $name<$type>(
            $queue: &mut VecDeque<$element>,
            $start: usize,
        ) -> Option<VecDeque<$result>> {
            #[cfg(kani)]
            kani::cover!(
                true,
                "actual production notification batch extraction executed"
            );
            $body
        }
    };
}
#[cfg(not(verus_keep_ghost))]
notification_batch_body!(ordinary_batch);
#[cfg(not(verus_keep_ghost))]
macro_rules! ordinary_next {
    ($name:ident<$type:ident>($queue:ident: &mut VecDeque<$element:ident>) -> Option<$result:ident> $body:block) => {
        pub(crate) fn $name<$type>($queue:&mut VecDeque<$element>)->Option<$result> $body
    };
}
#[cfg(not(verus_keep_ghost))]
notification_next_body!(ordinary_next);

macro_rules! notification_prepend_body {
    ($declare:ident) => {
        $declare!(prepend<T>(continuations: &mut VecDeque<T>, owned: &mut VecDeque<T>) -> () {
            #[cfg(kani)]
            kani::cover!(true, "actual callback continuation prepend executed");
            continuations.append(owned);
            std::mem::swap(continuations, owned);
        });
    };
}
#[cfg(not(verus_keep_ghost))]
macro_rules! ordinary_prepend {
    ($name:ident<$type:ident>($queue:ident: &mut VecDeque<$element:ident>,$owned:ident: &mut VecDeque<$other:ident>) -> () $body:block) => {
        pub(crate) fn $name<$type>($queue:&mut VecDeque<$element>,$owned:&mut VecDeque<$other>) $body
    };
}
#[cfg(not(verus_keep_ghost))]
notification_prepend_body!(ordinary_prepend);
#[cfg(verus_keep_ghost)]
pub(crate) use notification_batch_body;
#[cfg(verus_keep_ghost)]
pub(crate) use notification_next_body;
#[cfg(verus_keep_ghost)]
pub(crate) use notification_prepend_body;
