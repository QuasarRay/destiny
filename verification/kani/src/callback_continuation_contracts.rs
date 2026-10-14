//! Bounded concrete std-container checks supplement the arbitrary-sequence
//! Verus contract of the same production prepend body. Callback values are
//! opaque u64 labels here; foreign handlers, ownership gates and ABI remain.
use std::collections::VecDeque;
#[path = "../../../src/notification_batch.rs"]
mod production;

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(33)]
fn bounded_owned_continuation_prepend() {
    let a: [u64; 8] = kani::any();
    let b: [u64; 8] = kani::any();
    // A stated 0..8 collection partition, not a bound on the public API.
    // Verus separately covers arbitrary valid standard-container sequences.
    let a_len = usize::from(kani::any::<u8>() % 9);
    let b_len = usize::from(kani::any::<u8>() % 9);
    let mut continuations = VecDeque::new();
    let mut owned = VecDeque::new();
    for value in a.iter().take(a_len) {
        continuations.push_back(*value);
    }
    for value in b.iter().take(b_len) {
        owned.push_back(*value);
    }
    production::prepend(&mut continuations, &mut owned);
    assert!(continuations.is_empty());
    assert_eq!(owned.len(), a_len + b_len);
    for (index, value) in a.iter().take(a_len).enumerate() {
        assert_eq!(owned[index], *value);
    }
    for (index, value) in b.iter().take(b_len).enumerate() {
        assert_eq!(owned[a_len + index], *value);
    }
    kani::cover!(a_len == 0 && b_len == 0, "empty parent and continuation");
    kani::cover!(
        a_len == 0 && b_len == 8,
        "parent preserved without continuation"
    );
    kani::cover!(
        a_len == 8 && b_len == 0,
        "continuation without parent suffix"
    );
    kani::cover!(
        a_len == 8 && b_len == 8,
        "maximum declared sequence partition"
    );
}
