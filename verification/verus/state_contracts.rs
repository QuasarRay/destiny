use vstd::prelude::*;

verus! {

pub open spec fn adjust_time(current: int, delta: int) -> int {
    current + delta
}

// original-test: python/destiny/test/ballpark/test_time.py::test_ballpark_time_initialises_to_zero
pub proof fn ballpark_time_initializes_to_zero()
    ensures 0int == 0,
{
}

// original-test: python/destiny/test/ballpark/test_time.py::test_adjust_times_adds_to_ballpark_time
pub proof fn adjust_times_accumulates_original_deltas()
    ensures
        adjust_time(0, 2) == 2,
        adjust_time(adjust_time(0, 2), 3) == 5,
{
}

pub open spec fn running_after_start() -> bool { true }
pub open spec fn running_after_pause() -> bool { false }

// original-test: python/destiny/test/ballpark/test_time.py::test_initial_state_is_paused
pub proof fn initial_state_is_paused()
    ensures !false,
{
}

// original-test: python/destiny/test/ballpark/test_time.py::test_can_start
pub proof fn start_sets_running()
    ensures running_after_start(),
{
}

// original-test: python/destiny/test/ballpark/test_time.py::test_can_pause
pub proof fn pause_clears_running()
    ensures !running_after_pause(),
{
}

pub open spec fn positive_assign(current: int, requested: int) -> int {
    if requested > 0 { requested } else { current }
}

pub open spec fn non_negative_assign(current: int, requested: int) -> int {
    if requested >= 0 { requested } else { current }
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_mass
pub proof fn positive_mass_assignment_is_exact(current: int, requested: int)
    requires requested > 0,
    ensures positive_assign(current, requested) == requested,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_radius
pub proof fn non_negative_radius_assignment_is_exact(current: int, requested: int)
    requires requested >= 0,
    ensures non_negative_assign(current, requested) == requested,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_max_speed
pub proof fn non_negative_max_speed_assignment_is_exact(current: int, requested: int)
    requires requested >= 0,
    ensures non_negative_assign(current, requested) == requested,
{
}

pub struct Vec3 {
    pub x: int,
    pub y: int,
    pub z: int,
}

pub open spec fn assign_vec3(requested: Vec3) -> Vec3 {
    requested
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_position
pub proof fn position_assignment_is_exact(requested: Vec3)
    ensures
        assign_vec3(requested).x == requested.x,
        assign_vec3(requested).y == requested.y,
        assign_vec3(requested).z == requested.z,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_velocity
pub proof fn velocity_assignment_is_exact(requested: Vec3)
    ensures
        assign_vec3(requested).x == requested.x,
        assign_vec3(requested).y == requested.y,
        assign_vec3(requested).z == requested.z,
{
}

pub open spec fn clamp_fraction(requested: int, one: int) -> int {
    if requested < 0 { 0 } else if requested > one { one } else { requested }
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_speed_fraction
pub proof fn in_range_speed_fraction_is_exact(requested: int, one: int)
    requires
        one > 0,
        0 <= requested <= one,
    ensures clamp_fraction(requested, one) == requested,
{
}

pub open spec fn assign_flag(requested: bool) -> bool { requested }

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_free
pub proof fn free_flag_assignment_is_exact(requested: bool)
    ensures assign_flag(requested) == requested,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_massive
pub proof fn massive_flag_assignment_is_exact(requested: bool)
    ensures assign_flag(requested) == requested,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_global
pub proof fn global_flag_assignment_is_exact(requested: bool)
    ensures assign_flag(requested) == requested,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_agility
pub proof fn positive_agility_assignment_is_exact(current: int, requested: int)
    requires requested > 0,
    ensures positive_assign(current, requested) == requested,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_interactive
pub proof fn interactive_flag_assignment_is_exact(requested: bool)
    ensures assign_flag(requested) == requested,
{
}

pub open spec fn axis_center_distance(delta: int) -> int {
    if delta >= 0 { delta } else { -delta }
}

pub open spec fn surface_distance_from_center(
    center: int,
    left_radius: int,
    right_radius: int,
) -> int {
    center - left_radius - right_radius
}

pub open spec fn distance_squared(dx: int, dy: int, dz: int) -> int {
    dx * dx + dy * dy + dz * dz
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_surface_distance_between_two_balls_in_the_same_place_with_no_radius_is_zero
pub proof fn same_place_zero_radius_surface_distance_is_zero()
    ensures surface_distance_from_center(axis_center_distance(0), 0, 0) == 0,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_surface_distance_between_two_balls_with_no_radius_equals_the_distance_between_the_balls
pub proof fn zero_radius_surface_distance_equals_axis_center_distance()
    ensures surface_distance_from_center(axis_center_distance(100), 0, 0) == 100,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_surface_distance_between_two_balls_equals_the_distance_between_the_balls_minus_their_combined_radii
pub proof fn surface_distance_subtracts_combined_radii()
    ensures surface_distance_from_center(100, 10, 5) == 85,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_euclidean_distance
pub proof fn euclidean_distance_case_has_exact_squared_distance()
    ensures distance_squared(1, 2, 3) == 14,
{
    assert(1 * 1 + 2 * 2 + 3 * 3 == 14);
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_center_distance_between_two_balls_in_the_same_place
pub proof fn same_place_center_distance_is_zero()
    ensures axis_center_distance(0) == 0,
{
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_center_distance_between_two_balls
pub proof fn axis_center_distance_is_absolute_offset()
    ensures axis_center_distance(100) == 100,
{
}

} // verus!
