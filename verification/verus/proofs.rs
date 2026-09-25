use vstd::prelude::*;

verus! {

pub struct ParkState {
    pub evolving: bool,
    pub running: bool,
    pub time: int,
}

pub open spec fn add_ball_is_permitted(park: ParkState) -> bool {
    !park.evolving
}

// original-test: python/destiny/test/ballpark/evolve/test_add.py::test_balls_can_not_be_added_during_evolve
pub proof fn add_during_evolve_is_rejected(running: bool, time: int)
    ensures
        !add_ball_is_permitted(ParkState { evolving: true, running, time }),
{
}

pub open spec fn apply_non_negative_setter(current: int, requested: int) -> int {
    if requested >= 0 { requested } else { current }
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_ball_mass_to_a_negative_value_is_ineffective
// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_ball_radius_to_negative_value_is_ineffective
// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_max_speed_to_negative_value_is_ineffective
// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_agility_to_negative_value_is_ineffective
pub proof fn negative_setter_is_noop(current: int, requested: int)
    requires requested < 0,
    ensures apply_non_negative_setter(current, requested) == current,
{
}

pub open spec fn clamp_speed_fraction(requested: int, one: int) -> int {
    if requested < 0 { 0 } else if requested > one { one } else { requested }
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_speed_fraction_clamps_lowe_bound_at_zero
pub proof fn speed_fraction_below_zero_clamps_to_zero(requested: int, one: int)
    requires requested < 0, one > 0,
    ensures clamp_speed_fraction(requested, one) == 0,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_speed_fraction_clamps_upper_bound_at_one
pub proof fn speed_fraction_above_one_clamps_to_one(requested: int, one: int)
    requires requested > one, one > 0,
    ensures clamp_speed_fraction(requested, one) == one,
{
}

pub open spec fn follow_allowed(src_id: int, dst_id: int, target_moribund: bool) -> bool {
    src_id != dst_id && !target_moribund
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_ball_can_not_follow_self
pub proof fn follow_rejects_self(id: int, moribund: bool)
    ensures !follow_allowed(id, id, moribund),
{
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_ball_can_not_follow_moribund_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_follow_moribund_ball
pub proof fn follow_rejects_moribund(src: int, dst: int)
    ensures !follow_allowed(src, dst, true),
{
}

pub open spec fn orbit_allowed(
    src_id: int,
    dst_id: int,
    range_is_finite: bool,
    target_cloaked: bool,
    same_bubble: bool,
) -> bool {
    src_id != dst_id && range_is_finite && !target_cloaked && same_bubble
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_self
pub proof fn orbit_rejects_self(id: int, finite: bool, cloaked: bool, same_bubble: bool)
    ensures !orbit_allowed(id, id, finite, cloaked, same_bubble),
{
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_at_nan_range
pub proof fn orbit_rejects_nonfinite_range(src: int, dst: int, cloaked: bool, same_bubble: bool)
    ensures !orbit_allowed(src, dst, false, cloaked, same_bubble),
{
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_cloaked_ball
pub proof fn orbit_rejects_cloaked_target(src: int, dst: int, finite: bool, same_bubble: bool)
    ensures !orbit_allowed(src, dst, finite, true, same_bubble),
{
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_ball_in_different_bubble
pub proof fn orbit_rejects_cross_bubble_target(src: int, dst: int, finite: bool, cloaked: bool)
    ensures !orbit_allowed(src, dst, finite, cloaked, false),
{
}

pub open spec fn visibility_occluder(massive: bool, cloaked: bool) -> bool {
    massive && !cloaked
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_not_massive_is_not_an_occlusion
pub proof fn non_massive_is_not_occluder(cloaked: bool)
    ensures !visibility_occluder(false, cloaked),
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_cloaked_is_not_an_occlusion
pub proof fn cloaked_is_not_occluder(massive: bool)
    ensures !visibility_occluder(massive, true),
{
}

pub open spec fn uncloak_restores_massive(in_warp: bool) -> bool {
    !in_warp
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_makes_a_ball_massive
pub proof fn uncloak_restores_massive_outside_warp()
    ensures uncloak_restores_massive(false),
{
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_a_ball_in_warp_does_not_make_it_massive
pub proof fn uncloak_keeps_nonmassive_in_warp()
    ensures !uncloak_restores_massive(true),
{
}

pub open spec fn proximity_eligible(
    owner_is_free: bool,
    target_is_interactive: bool,
    only_interactives: bool,
) -> bool {
    owner_is_free && (!only_interactives || target_is_interactive)
}

// original-test: python/destiny/test/ballpark/test_callbacks.py::test_ignores_balls_that_are_not_free
pub proof fn proximity_rejects_nonfree_owner(target_interactive: bool, only_interactives: bool)
    ensures !proximity_eligible(false, target_interactive, only_interactives),
{
}

// original-test: python/destiny/test/ballpark/test_callbacks.py::test_only_interactives_ignores_non_interactive_balls
pub proof fn proximity_only_interactives_rejects_noninteractive()
    ensures !proximity_eligible(true, false, true),
{
}

pub open spec fn stop_mode(_: int) -> int { 0 }

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_stopped_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_following_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_orbiting_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_missile
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_ball_in_formation
pub proof fn stop_maps_every_mode_to_stop(mode: int)
    ensures stop_mode(mode) == 0,
{
}

pub open spec fn missile_follow_range(src_radius: int, dst_radius: int) -> int {
    -(src_radius + dst_radius)
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_sets_follow_range_as_negative_sum_of_src_and_dst_radii
pub proof fn missile_range_is_negative_radius_sum(src_radius: int, dst_radius: int)
    ensures missile_follow_range(src_radius, dst_radius) == -(src_radius + dst_radius),
{
}

} // verus!
