use crate::production_formation;
use destiny_original_spec::{free_formation_slot, reserve_formation_slot};

// Original helper FORMATIONS[1] is Arrow with four offsets. Ball::Ball resets
// all sixteen reservation bits. The wrapper's attachment/identity resolution
// and LoadFormations/SetBallFormation remain separate runtime obligations.
const ARROW_SLOTS: usize = 4;

// original-case: python/destiny/test/test_ball.py::BallTest::test_reserve_formation_slot
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(not(kani), test)]
fn original_first_slot_refines_scan() {
    let (state, slot) = production_formation::reserve_formation_slot(0, Some(ARROW_SLOTS));
    assert!(slot == 0);
    assert!((state, slot) == reserve_formation_slot(0, Some(ARROW_SLOTS)));
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_slots_are_reserved_in_incremental_order
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(not(kani), test)]
fn original_incremental_slots_refine_scan() {
    let mut actual = 0;
    let mut expected = 0;
    let mut index = 0_usize;
    while index < ARROW_SLOTS {
        let (next, slot) = production_formation::reserve_formation_slot(actual, Some(ARROW_SLOTS));
        let reference = reserve_formation_slot(expected, Some(ARROW_SLOTS));
        assert!(slot == index as i8);
        assert!((next, slot) == reference);
        actual = next;
        expected = reference.0;
        index = index.saturating_add(1);
    }
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_reserving_too_many_formation_slots_fails
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(not(kani), test)]
fn original_exhausted_slots_refine_scan() {
    let mut actual = 0;
    let mut expected = 0;
    let mut attempts = 0_usize;
    while attempts < ARROW_SLOTS {
        actual = production_formation::reserve_formation_slot(actual, Some(ARROW_SLOTS)).0;
        expected = reserve_formation_slot(expected, Some(ARROW_SLOTS)).0;
        attempts = attempts.saturating_add(1);
    }
    let observed = production_formation::reserve_formation_slot(actual, Some(ARROW_SLOTS));
    assert!(observed.1 == -1);
    assert!(observed == reserve_formation_slot(expected, Some(ARROW_SLOTS)));
    assert!(observed.0 == actual);
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_free_formation_slot
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(not(kani), test)]
fn original_sixteen_attempts_then_release_refine_scan() {
    let mut actual = 0;
    let mut expected = 0;
    // Preserve all sixteen original attempts, including twelve exhausted ones.
    let mut attempts = 0_u8;
    while attempts < 16 {
        let observed = production_formation::reserve_formation_slot(actual, Some(ARROW_SLOTS));
        let reference = reserve_formation_slot(expected, Some(ARROW_SLOTS));
        assert!(observed == reference);
        actual = observed.0;
        expected = reference.0;
        attempts = attempts.saturating_add(1);
    }
    actual = production_formation::free_formation_slot(actual, Some(ARROW_SLOTS), 2);
    expected = free_formation_slot(expected, Some(ARROW_SLOTS), 2);
    assert!(actual == expected);
    let observed = production_formation::reserve_formation_slot(actual, Some(ARROW_SLOTS));
    assert!(observed.1 == 2);
    assert!(observed == reserve_formation_slot(expected, Some(ARROW_SLOTS)));
}

// This is a kernel generalization, not credit for detached Python construction.
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(17)]
fn reservation_refines_scan_for_all_inputs() {
    let reserved: u16 = kani::any();
    let count: Option<usize> = kani::any();
    let observed = production_formation::reserve_formation_slot(reserved, count);
    assert!(observed == reserve_formation_slot(reserved, count));
    kani::cover!(count.is_none());
    kani::cover!(count == Some(0));
    kani::cover!(count == Some(16) && reserved == 0);
    kani::cover!(count == Some(16) && reserved == u16::MAX);
    kani::cover!(count == Some(17));
}

#[cfg(kani)]
#[kani::proof]
fn release_refines_spec_for_all_inputs() {
    let reserved: u16 = kani::any();
    let count: Option<usize> = kani::any();
    let slot: i64 = kani::any();
    assert!(
        production_formation::free_formation_slot(reserved, count, slot)
            == free_formation_slot(reserved, count, slot)
    );
    kani::cover!(slot == 0 && count == Some(1));
    kani::cover!(slot == 15 && count == Some(16));
    kani::cover!(slot == -1 && count == Some(16));
    kani::cover!(slot == 16 && count == Some(16));
    kani::cover!(count.is_none());
}
