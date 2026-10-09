use destiny_metaverification::paired_regression;
use destiny_original_spec::{
    NO_FORMATION, assign_formation, free_formation_slot, reserve_formation_slot,
};

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot_returns_negative_one_when_ball_is_not_set_up_properly
paired_regression!(
    unconfigured_formation_returns_negative_one,
    formation_unconfigured,
    {
        assert_eq!(reserve_formation_slot(0, None), (0, -1));
    }
);

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot
paired_regression!(
    #[kani::unwind(17)]
    first_formation_slot_is_zero,
    formation_first_slot,
    {
        assert_eq!(reserve_formation_slot(0, Some(4)), (1, 0));
    }
);

// original-test: python/destiny/test/test_ball.py::test_slots_are_reserved_in_incremental_order
paired_regression!(
    #[kani::unwind(17)]
    original_four_slots_are_incremental,
    formation_incremental,
    {
        let mut reserved = 0;
        for expected in 0..4 {
            let (next, observed) = reserve_formation_slot(reserved, Some(4));
            assert_eq!(observed, expected);
            reserved = next;
        }
    }
);

// original-test: python/destiny/test/test_ball.py::test_reserving_too_many_formation_slots_fails
paired_regression!(
    #[kani::unwind(17)]
    exhausted_formation_returns_negative_one,
    formation_exhausted,
    {
        assert_eq!(reserve_formation_slot(0b1111, Some(4)), (0b1111, -1));
    }
);

// original-test: python/destiny/test/test_ball.py::test_free_formation_slot
paired_regression!(
    #[kani::unwind(17)]
    original_freed_slot_two_is_reused,
    formation_reuse,
    {
        let reserved = free_formation_slot(0b1111, Some(4), 2);
        assert_eq!(reserve_formation_slot(reserved, Some(4)), (0b1111, 2));
    }
);

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_valid_formation_gets_set
paired_regression!(valid_formation_is_assigned, formation_valid_assignment, {
    assert_eq!(assign_formation(NO_FORMATION, 0, 0, 3), (0, 0));
});

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_formation_out_of_range_does_not_get_set
paired_regression!(
    out_of_range_formation_is_ignored,
    formation_invalid_assignment,
    {
        assert_eq!(assign_formation(NO_FORMATION, 0, 3, 3), (NO_FORMATION, 0));
    }
);

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(17)]
fn formation_reserve_selects_the_first_free_bit_and_changes_only_that_bit() {
    let reserved: u16 = kani::any();
    let count: usize = kani::any();
    kani::assume(count <= 16);
    let (next, slot) = reserve_formation_slot(reserved, Some(count));
    if slot < 0 {
        assert_eq!(next, reserved);
        for index in 0..count {
            assert_ne!(reserved & (1_u16 << index), 0);
        }
    } else {
        let index = slot as usize;
        assert!(index < count);
        let bit = 1_u16 << index;
        assert_eq!(reserved & bit, 0);
        assert_eq!(next, reserved | bit);
        for prior in 0..index {
            assert_ne!(reserved & (1_u16 << prior), 0);
        }
    }
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(17)]
fn formation_reserve_rejects_invalid_setup_without_mutation() {
    let reserved: u16 = kani::any();
    let count: usize = kani::any();
    kani::assume(count > 16);
    assert_eq!(
        reserve_formation_slot(reserved, Some(count)),
        (reserved, -1)
    );
    assert_eq!(reserve_formation_slot(reserved, None), (reserved, -1));
}

#[cfg(kani)]
#[kani::proof]
fn formation_free_changes_only_the_requested_bit_or_is_a_noop() {
    let reserved: u16 = kani::any();
    let count: Option<usize> = kani::any();
    let slot: i64 = kani::any();
    let next = free_formation_slot(reserved, count, slot);
    if let Some(count) = count
        && (0..16).contains(&slot)
        && (slot as usize) < count
    {
        assert_eq!(next, reserved & !(1_u16 << slot));
    } else {
        assert_eq!(next, reserved);
    }
}

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(17)]
fn formation_full_bitset_reuses_any_valid_freed_slot() {
    let count: usize = kani::any();
    let slot: i64 = kani::any();
    kani::assume(count > 0 && count <= 16 && slot >= 0 && (slot as usize) < count);
    let freed = free_formation_slot(u16::MAX, Some(count), slot);
    assert_eq!(
        reserve_formation_slot(freed, Some(count)),
        (u16::MAX, slot as i8)
    );
}

#[cfg(kani)]
#[kani::proof]
fn formation_clearing_an_assigned_ball_resets_reservations() {
    let current: u8 = kani::any();
    let reserved: u16 = kani::any();
    let count: usize = kani::any();
    kani::assume(current <= 127);
    assert_eq!(
        assign_formation(current, reserved, -1, count),
        (NO_FORMATION, 0)
    );
    assert_eq!(
        assign_formation(current, reserved, 255, count),
        (NO_FORMATION, 0)
    );
}

#[cfg(kani)]
#[kani::proof]
fn formation_invalid_assignment_preserves_id_and_reservations() {
    let current: u8 = kani::any();
    let reserved: u16 = kani::any();
    let requested: i64 = kani::any();
    let count: usize = kani::any();
    kani::assume(requested != -1 && requested != 255);
    kani::assume(!(0..=127).contains(&requested) || requested as usize >= count);
    assert_eq!(
        assign_formation(current, reserved, requested, count),
        (current, reserved)
    );
}
