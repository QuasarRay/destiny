//! Concrete word-domain counterpart for the shared production ABI kernel.
//! No pointer, OS-atomic, token ownership or concurrent-schedule claim.
#[path = "../../../src/ffi_lifecycle.rs"]
mod production;

#[cfg(kani)]
#[kani::proof]
fn retain_count_is_total_and_rejects_zero_and_maximum() {
    let count: usize = kani::any();
    let result = production::retain_count(count);
    if count == 0 || count == usize::MAX {
        assert!(result.is_none());
    } else {
        assert!(result == Some(count + 1));
    }
    kani::cover!(count == 0, "zero reference count rejects retain");
    kani::cover!(count == usize::MAX, "machine maximum rejects retain");
    kani::cover!(
        count == usize::MAX - 1,
        "last nonoverflowing retain is feasible"
    );
}

#[cfg(kani)]
#[kani::proof]
fn release_count_is_total_and_only_one_can_reach_zero() {
    let count: usize = kani::any();
    let result = production::release_count(count);
    if count == 0 {
        assert!(result.is_none());
    } else {
        assert!(result == Some(count - 1));
        assert!((result == Some(0)) == (count == 1));
    }
    kani::cover!(count == 0, "zero reference count rejects release");
    kani::cover!(
        count == 1,
        "exact final-reference counter transition is feasible"
    );
    kani::cover!(
        count == usize::MAX,
        "maximum reference count releases without underflow"
    );
}
