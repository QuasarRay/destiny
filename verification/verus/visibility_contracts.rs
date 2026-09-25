use vstd::prelude::*;

verus! {

pub struct VisibilityState {
    pub cloaked: bool,
    pub massive: bool,
}

pub open spec fn cloak_transition() -> VisibilityState {
    VisibilityState {
        cloaked: true,
        massive: false,
    }
}

pub open spec fn uncloak_transition(in_warp: bool) -> VisibilityState {
    VisibilityState {
        cloaked: false,
        massive: !in_warp,
    }
}

pub open spec fn visibility_candidate_blocks(
    is_massive: bool,
    is_cloaked: bool,
    intersects_open_segment: bool,
) -> bool {
    is_massive && !is_cloaked && intersects_open_segment
}

pub open spec fn visibility_result(candidate_id: int, candidate_blocks: bool) -> int {
    if candidate_blocks { candidate_id } else { 0 }
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_returns_zero_when_there_is_no_occlusion
pub proof fn no_occlusion_returns_zero(candidate_id: int)
    ensures visibility_result(candidate_id, false) == 0,
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_returns_the_id_of_the_occluding_ball_when_there_is_an_occlusion
pub proof fn occlusion_returns_candidate_id(candidate_id: int)
    ensures visibility_result(candidate_id, true) == candidate_id,
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_not_massive_is_not_an_occlusion
pub proof fn nonmassive_candidate_is_not_occluder(cloaked: bool, intersects: bool)
    ensures !visibility_candidate_blocks(false, cloaked, intersects),
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_cloaked_is_not_an_occlusion
pub proof fn cloaked_candidate_is_not_occluder(massive: bool, intersects: bool)
    ensures !visibility_candidate_blocks(massive, true, intersects),
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_cloaked_balls_are_cloaked
// original-test: python/destiny/test/ballpark/test_visibility.py::test_cloaked_balls_are_not_massive
pub proof fn cloak_sets_expected_visibility_state()
    ensures
        cloak_transition().cloaked,
        !cloak_transition().massive,
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaked_balls_are_not_cloaked
// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_makes_a_ball_massive
pub proof fn nonwarp_uncloak_sets_expected_visibility_state()
    ensures
        !uncloak_transition(false).cloaked,
        uncloak_transition(false).massive,
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_a_ball_in_warp_does_not_make_it_massive
pub proof fn warp_uncloak_does_not_restore_massive()
    ensures
        !uncloak_transition(true).cloaked,
        !uncloak_transition(true).massive,
{
}

pub open spec fn scan_cone_pi_over_2_x(x: int, y: int, z: int, range: int) -> bool {
    range > 0
        && x >= 0
        && 2 * x * x >= x * x + y * y + z * z
        && x * x + y * y + z * z <= range * range
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_scan_cone_finds_nothing_when_there_is_nothing_to_be_found
pub proof fn empty_scan_candidate_set_returns_empty()
    ensures Seq::<int>::empty().len() == 0,
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_scan_cone_finds_ball_when_it_is_in_the_cone
pub proof fn positive_x_candidate_is_inside_test_cone()
    ensures scan_cone_pi_over_2_x(50, 0, 0, 100),
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_scan_cone_excludes_balls_not_in_cone
pub proof fn orthogonal_and_backward_test_candidates_are_excluded()
    ensures
        !scan_cone_pi_over_2_x(-50, 0, 0, 100),
        !scan_cone_pi_over_2_x(0, 50, 0, 100),
        !scan_cone_pi_over_2_x(0, -50, 0, 100),
        !scan_cone_pi_over_2_x(0, 0, 50, 100),
        !scan_cone_pi_over_2_x(0, 0, -50, 100),
{
}

} // verus!
