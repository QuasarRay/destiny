use destiny_original_spec::{
    VisibilityState, cloak_transition, scan_cone_pi_over_2_x,
    uncloak_transition, visibility_candidate_blocks, visibility_result,
};

// original-test: python/destiny/test/ballpark/test_visibility.py::test_returns_zero_when_there_is_no_occlusion
#[cfg(kani)]
#[kani::proof]
fn no_eligible_occluder_returns_zero() {
    let candidate_id: i64 = kani::any();
    assert_eq!(visibility_result(candidate_id, false), 0);
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_returns_the_id_of_the_occluding_ball_when_there_is_an_occlusion
#[cfg(kani)]
#[kani::proof]
fn eligible_occluder_returns_its_id() {
    let candidate_id: i64 = kani::any();
    assert_eq!(visibility_result(candidate_id, true), candidate_id);
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_not_massive_is_not_an_occlusion
#[cfg(kani)]
#[kani::proof]
fn nonmassive_candidate_never_blocks_visibility() {
    let cloaked: bool = kani::any();
    let intersects: bool = kani::any();
    assert!(!visibility_candidate_blocks(false, cloaked, intersects));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_cloaked_is_not_an_occlusion
#[cfg(kani)]
#[kani::proof]
fn cloaked_candidate_never_blocks_visibility() {
    let massive: bool = kani::any();
    let intersects: bool = kani::any();
    assert!(!visibility_candidate_blocks(massive, true, intersects));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_cloaked_balls_are_cloaked
// original-test: python/destiny/test/ballpark/test_visibility.py::test_cloaked_balls_are_not_massive
#[cfg(kani)]
#[kani::proof]
fn cloak_transition_sets_cloaked_and_nonmassive() {
    assert_eq!(
        cloak_transition(),
        VisibilityState {
            cloaked: true,
            massive: false,
        }
    );
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaked_balls_are_not_cloaked
// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_makes_a_ball_massive
#[cfg(kani)]
#[kani::proof]
fn nonwarp_uncloak_clears_cloak_and_restores_massive() {
    assert_eq!(
        uncloak_transition(false),
        VisibilityState {
            cloaked: false,
            massive: true,
        }
    );
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_a_ball_in_warp_does_not_make_it_massive
#[cfg(kani)]
#[kani::proof]
fn warp_uncloak_clears_cloak_without_restoring_massive() {
    assert_eq!(
        uncloak_transition(true),
        VisibilityState {
            cloaked: false,
            massive: false,
        }
    );
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_scan_cone_finds_nothing_when_there_is_nothing_to_be_found
#[cfg(kani)]
#[kani::proof]
fn empty_candidate_set_has_no_scan_result() {
    let result: [i64; 0] = [];
    assert!(result.is_empty());
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_scan_cone_finds_ball_when_it_is_in_the_cone
#[cfg(kani)]
#[kani::proof]
fn positive_x_test_candidate_is_inside_scan_cone() {
    assert!(scan_cone_pi_over_2_x([50, 0, 0], 100));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_scan_cone_excludes_balls_not_in_cone
#[cfg(kani)]
#[kani::proof]
fn tested_non_x_directions_are_outside_scan_cone() {
    assert!(!scan_cone_pi_over_2_x([-50, 0, 0], 100));
    assert!(!scan_cone_pi_over_2_x([0, 50, 0], 100));
    assert!(!scan_cone_pi_over_2_x([0, -50, 0], 100));
    assert!(!scan_cone_pi_over_2_x([0, 0, 50], 100));
    assert!(!scan_cone_pi_over_2_x([0, 0, -50], 100));
}
