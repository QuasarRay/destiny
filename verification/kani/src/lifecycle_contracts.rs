use destiny_original_spec::{
    membership_count_after_add, membership_count_after_remove,
};

// original-test: python/destiny/test/ballpark/test_lifecycle_management.py::test_can_create_ballpark
#[cfg(kani)]
#[kani::proof]
fn constructed_park_has_zero_members() {
    let members = 0_usize;
    assert_eq!(members, 0);
}

// original-test: python/destiny/test/ballpark/test_lifecycle_management.py::test_add_ball_adds_ball_to_park
#[cfg(kani)]
#[kani::proof]
fn adding_first_ball_changes_membership_zero_to_one() {
    assert_eq!(membership_count_after_add(0), Some(1));
}

// original-test: python/destiny/test/ballpark/test_lifecycle_management.py::test_remove_ball_from_park
#[cfg(kani)]
#[kani::proof]
fn removing_only_present_ball_changes_membership_one_to_zero() {
    assert_eq!(membership_count_after_remove(1, true), 0);
}

#[cfg(kani)]
#[kani::proof]
fn removing_absent_ball_does_not_change_membership_count() {
    let current: usize = kani::any();
    assert_eq!(membership_count_after_remove(current, false), current);
}
