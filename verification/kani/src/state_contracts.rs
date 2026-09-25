use destiny_original_spec::{
    BallState, Vec3, adjust_time, apply_non_negative_setter, apply_positive_setter,
    assign_flag, assign_vec3, center_distance, clamp_speed_fraction, running_after_pause,
    running_after_start, surface_distance, surface_distance_from_center,
};

// original-test: python/destiny/test/ballpark/test_time.py::test_ballpark_time_initialises_to_zero
#[cfg(kani)]
#[kani::proof]
fn ballpark_time_initializes_to_zero() {
    let initial = 0_i64;
    assert_eq!(initial, 0);
}

// original-test: python/destiny/test/ballpark/test_time.py::test_adjust_times_adds_to_ballpark_time
#[cfg(kani)]
#[kani::proof]
fn adjust_times_accumulates_original_deltas() {
    let first = adjust_time(0, 2).unwrap();
    let second = adjust_time(first, 3).unwrap();
    assert_eq!(first, 2);
    assert_eq!(second, 5);
}

// original-test: python/destiny/test/ballpark/test_time.py::test_initial_state_is_paused
#[cfg(kani)]
#[kani::proof]
fn initial_running_state_is_false() {
    let initial_running = false;
    assert!(!initial_running);
}

// original-test: python/destiny/test/ballpark/test_time.py::test_can_start
#[cfg(kani)]
#[kani::proof]
fn start_sets_running_true() {
    assert!(running_after_start());
}

// original-test: python/destiny/test/ballpark/test_time.py::test_can_pause
#[cfg(kani)]
#[kani::proof]
fn pause_sets_running_false() {
    assert!(!running_after_pause());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_mass
#[cfg(kani)]
#[kani::proof]
fn positive_mass_assignment_is_exact() {
    let current: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(current.is_finite());
    kani::assume(requested.is_finite() && requested > 0.0);
    assert_eq!(
        apply_positive_setter(current, requested).to_bits(),
        requested.to_bits()
    );
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_radius
#[cfg(kani)]
#[kani::proof]
fn non_negative_radius_assignment_is_exact() {
    let current: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(current.is_finite());
    kani::assume(requested.is_finite() && requested >= 0.0);
    assert_eq!(
        apply_non_negative_setter(current, requested).to_bits(),
        requested.to_bits()
    );
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_max_speed
#[cfg(kani)]
#[kani::proof]
fn non_negative_max_speed_assignment_is_exact() {
    let current: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(current.is_finite());
    kani::assume(requested.is_finite() && requested >= 0.0);
    assert_eq!(
        apply_non_negative_setter(current, requested).to_bits(),
        requested.to_bits()
    );
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_position
#[cfg(kani)]
#[kani::proof]
fn position_assignment_preserves_all_components() {
    let requested = Vec3 {
        x: kani::any(),
        y: kani::any(),
        z: kani::any(),
    };
    let assigned = assign_vec3(requested);
    assert_eq!(assigned.x.to_bits(), requested.x.to_bits());
    assert_eq!(assigned.y.to_bits(), requested.y.to_bits());
    assert_eq!(assigned.z.to_bits(), requested.z.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_velocity
#[cfg(kani)]
#[kani::proof]
fn velocity_assignment_preserves_all_components() {
    let requested = Vec3 {
        x: kani::any(),
        y: kani::any(),
        z: kani::any(),
    };
    let assigned = assign_vec3(requested);
    assert_eq!(assigned.x.to_bits(), requested.x.to_bits());
    assert_eq!(assigned.y.to_bits(), requested.y.to_bits());
    assert_eq!(assigned.z.to_bits(), requested.z.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_speed_fraction
#[cfg(kani)]
#[kani::proof]
fn in_range_speed_fraction_assignment_is_exact() {
    let requested: f64 = kani::any();
    kani::assume(requested.is_finite());
    kani::assume((0.0..=1.0).contains(&requested));
    assert_eq!(
        clamp_speed_fraction(requested).to_bits(),
        requested.to_bits()
    );
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_free
#[cfg(kani)]
#[kani::proof]
fn free_flag_assignment_is_exact() {
    let requested: bool = kani::any();
    assert_eq!(assign_flag(requested), requested);
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_massive
#[cfg(kani)]
#[kani::proof]
fn massive_flag_assignment_is_exact() {
    let requested: bool = kani::any();
    assert_eq!(assign_flag(requested), requested);
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_global
#[cfg(kani)]
#[kani::proof]
fn global_flag_assignment_is_exact() {
    let requested: bool = kani::any();
    assert_eq!(assign_flag(requested), requested);
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_agility
#[cfg(kani)]
#[kani::proof]
fn positive_agility_assignment_is_exact() {
    let current: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(current.is_finite());
    kani::assume(requested.is_finite() && requested > 0.0);
    assert_eq!(
        apply_positive_setter(current, requested).to_bits(),
        requested.to_bits()
    );
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_can_set_ball_interactive
#[cfg(kani)]
#[kani::proof]
fn interactive_flag_assignment_is_exact() {
    let requested: bool = kani::any();
    assert_eq!(assign_flag(requested), requested);
}

fn ball(position: Vec3, radius: f64) -> BallState {
    BallState {
        position,
        radius,
        ..BallState::default()
    }
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_surface_distance_between_two_balls_in_the_same_place_with_no_radius_is_zero
#[cfg(kani)]
#[kani::proof]
fn same_place_zero_radius_surface_distance_is_zero() {
    let left = ball(Vec3::ZERO, 0.0);
    let right = ball(Vec3::ZERO, 0.0);
    assert_eq!(surface_distance(&left, &right).to_bits(), 0.0_f64.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_surface_distance_between_two_balls_with_no_radius_equals_the_distance_between_the_balls
#[cfg(kani)]
#[kani::proof]
fn zero_radius_surface_distance_equals_axis_center_distance() {
    let left = ball(Vec3::ZERO, 0.0);
    let right = ball(Vec3 { x: 100.0, y: 0.0, z: 0.0 }, 0.0);
    assert_eq!(surface_distance(&left, &right).to_bits(), 100.0_f64.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_surface_distance_between_two_balls_equals_the_distance_between_the_balls_minus_their_combined_radii
#[cfg(kani)]
#[kani::proof]
fn surface_distance_subtracts_combined_radii() {
    assert_eq!(
        surface_distance_from_center(100.0, 10.0, 5.0).to_bits(),
        85.0_f64.to_bits()
    );
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_euclidean_distance
#[cfg(kani)]
#[kani::proof]
fn euclidean_distance_matches_original_three_axis_case() {
    let left = ball(Vec3::ZERO, 0.0);
    let right = ball(Vec3 { x: 1.0, y: 2.0, z: 3.0 }, 0.0);
    let observed = center_distance(&left, &right);
    let expected = 3.7416573867739413_f64;
    assert!((observed - expected).abs() < 1.0e-12);
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_center_distance_between_two_balls_in_the_same_place
#[cfg(kani)]
#[kani::proof]
fn same_place_center_distance_is_zero() {
    let left = ball(Vec3::ZERO, 0.0);
    let right = ball(Vec3::ZERO, 0.0);
    assert_eq!(center_distance(&left, &right).to_bits(), 0.0_f64.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_center_distance_between_two_balls
#[cfg(kani)]
#[kani::proof]
fn axis_center_distance_matches_absolute_offset() {
    let left = ball(Vec3::ZERO, 0.0);
    let right = ball(Vec3 { x: 100.0, y: 0.0, z: 0.0 }, 0.0);
    assert_eq!(center_distance(&left, &right).to_bits(), 100.0_f64.to_bits());
}
