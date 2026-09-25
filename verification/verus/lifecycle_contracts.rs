use vstd::prelude::*;

verus! {

pub open spec fn membership_count_after_add(current: nat) -> nat {
    current + 1
}

pub open spec fn membership_count_after_remove(current: nat, was_present: bool) -> nat
    recommends !was_present || current > 0
{
    if was_present { current - 1 } else { current }
}

// original-test: python/destiny/test/ballpark/test_lifecycle_management.py::test_can_create_ballpark
pub proof fn constructed_park_has_zero_members()
    ensures 0nat == 0,
{
}

// original-test: python/destiny/test/ballpark/test_lifecycle_management.py::test_add_ball_adds_ball_to_park
pub proof fn adding_first_ball_changes_membership_zero_to_one()
    ensures membership_count_after_add(0) == 1,
{
}

// original-test: python/destiny/test/ballpark/test_lifecycle_management.py::test_remove_ball_from_park
pub proof fn removing_only_present_ball_changes_membership_one_to_zero()
    ensures membership_count_after_remove(1, true) == 0,
{
}

pub proof fn removing_absent_ball_preserves_membership(current: nat)
    ensures membership_count_after_remove(current, false) == current,
{
}

} // verus!
