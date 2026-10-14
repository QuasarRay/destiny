//! Exact source slot rejection decision; native caller/collection and mode
//! callback publication remain separate from this whole machine-domain kernel.
#[path = "../../../src/formation.rs"]
mod production;
#[cfg(kani)]
#[kani::proof]
fn total_oversized_formation_follower() {
    let old: usize = kani::any();
    let new: usize = kani::any();
    let mode: u8 = kani::any();
    let slot: i64 = kani::any();
    let result = production::oversized_follower(old, new, mode, slot);
    assert_eq!(result.is_some(), new <= i32::MAX as usize);
    if let Some(drop) = result {
        assert_eq!(drop, old > new && mode == 12 && slot >= new as i64);
    }
    kani::cover!(
        result == Some(true) && new == 0 && slot == 0,
        "zero-slot formation drops slot0"
    );
    kani::cover!(
        result == Some(true) && old == 4 && new == 1 && slot == 1,
        "original4to1slot shrink"
    );
    kani::cover!(
        result == Some(false) && old == 4 && new == 1 && slot == 0,
        "valid slot0 survives"
    );
    kani::cover!(
        result.is_none(),
        "out-of-C-int count retains caller rejection"
    );
}
