use destiny_original_spec::{
    MotionMode, ParkState, add_ball_is_permitted, apply_non_negative_setter,
    apply_positive_setter, clamp_speed_fraction, follow_allowed, missile_follow_range,
    non_negative_setter_accepts, orbit_allowed, positive_setter_accepts,
    proximity_eligible, stopped_mode, uncloak_restores_massive, visibility_occluder,
};

// original-test: python/destiny/test/ballpark/evolve/test_add.py::test_balls_can_not_be_added_during_evolve
#[cfg(kani)]
#[kani::proof]
fn original_add_during_evolve_contract_rejects_every_evolving_state() {
    let running: bool = kani::any();
    let time: i64 = kani::any();
    let park = ParkState {
        evolving: true,
        running,
        time,
    };
    assert!(!add_ball_is_permitted(&park));
}

#[cfg(kani)]
#[kani::proof]
fn original_add_outside_evolve_contract_permits_every_non_evolving_state() {
    let running: bool = kani::any();
    let time: i64 = kani::any();
    let park = ParkState {
        evolving: false,
        running,
        time,
    };
    assert!(add_ball_is_permitted(&park));
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_ball_radius_to_negative_value_is_ineffective
// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_max_speed_to_negative_value_is_ineffective
#[cfg(kani)]
#[kani::proof]
fn negative_non_negative_setter_requests_are_bitwise_noops() {
    let previous: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(previous.is_finite());
    kani::assume(requested.is_finite());
    kani::assume(requested < 0.0);

    let observed = apply_non_negative_setter(previous, requested);
    assert_eq!(observed.to_bits(), previous.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_ball_mass_to_a_negative_value_is_ineffective
// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_agility_to_negative_value_is_ineffective
#[cfg(kani)]
#[kani::proof]
fn non_positive_positive_setter_requests_are_bitwise_noops() {
    let previous: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(previous.is_finite());
    kani::assume(requested.is_finite());
    kani::assume(requested <= 0.0);

    let observed = apply_positive_setter(previous, requested);
    assert_eq!(observed.to_bits(), previous.to_bits());
}

#[cfg(kani)]
#[kani::proof]
fn setter_acceptance_predicates_match_original_thresholds() {
    let requested: f64 = kani::any();
    kani::assume(requested.is_finite());
    assert_eq!(non_negative_setter_accepts(requested), requested >= 0.0);
    assert_eq!(positive_setter_accepts(requested), requested > 0.0);
}

#[cfg(kani)]
#[kani::proof]
fn non_negative_setter_requests_are_accepted_exactly() {
    let previous: f64 = kani::any();
    let requested: f64 = kani::any();
    kani::assume(previous.is_finite());
    kani::assume(requested.is_finite());
    kani::assume(requested >= 0.0);

    let observed = apply_non_negative_setter(previous, requested);
    assert_eq!(observed.to_bits(), requested.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_speed_fraction_clamps_lowe_bound_at_zero
#[cfg(kani)]
#[kani::proof]
fn speed_fraction_below_zero_clamps_to_zero() {
    let requested: f64 = kani::any();
    kani::assume(requested.is_finite());
    kani::assume(requested < 0.0);
    assert_eq!(clamp_speed_fraction(requested).to_bits(), 0.0_f64.to_bits());
}

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_setting_speed_fraction_clamps_upper_bound_at_one
#[cfg(kani)]
#[kani::proof]
fn speed_fraction_above_one_clamps_to_one() {
    let requested: f64 = kani::any();
    kani::assume(requested.is_finite());
    kani::assume(requested > 1.0);
    assert_eq!(clamp_speed_fraction(requested).to_bits(), 1.0_f64.to_bits());
}

#[cfg(kani)]
#[kani::proof]
fn speed_fraction_inside_closed_unit_interval_is_unchanged() {
    let requested: f64 = kani::any();
    kani::assume(requested.is_finite());
    kani::assume(requested >= 0.0);
    kani::assume(requested <= 1.0);
    assert_eq!(
        clamp_speed_fraction(requested).to_bits(),
        requested.to_bits()
    );
}


// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_ball_can_not_follow_self
#[cfg(kani)]
#[kani::proof]
fn follow_rejects_self_for_every_ball_id() {
    let id: i64 = kani::any();
    let moribund: bool = kani::any();
    assert!(!follow_allowed(id, id, moribund));
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_ball_can_not_follow_moribund_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_follow_moribund_ball
#[cfg(kani)]
#[kani::proof]
fn follow_rejects_every_moribund_target() {
    let src: i64 = kani::any();
    let dst: i64 = kani::any();
    assert!(!follow_allowed(src, dst, true));
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_self
#[cfg(kani)]
#[kani::proof]
fn orbit_rejects_self() {
    let id: i64 = kani::any();
    let range: f64 = kani::any();
    let cloaked: bool = kani::any();
    let same_bubble: bool = kani::any();
    assert!(!orbit_allowed(id, id, range, cloaked, same_bubble));
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_at_nan_range
#[cfg(kani)]
#[kani::proof]
fn orbit_rejects_nan_range() {
    let src: i64 = kani::any();
    let dst: i64 = kani::any();
    let same_bubble: bool = kani::any();
    assert!(!orbit_allowed(src, dst, f64::NAN, false, same_bubble));
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_cloaked_ball
#[cfg(kani)]
#[kani::proof]
fn orbit_rejects_cloaked_targets() {
    let src: i64 = kani::any();
    let dst: i64 = kani::any();
    let range: f64 = kani::any();
    let same_bubble: bool = kani::any();
    assert!(!orbit_allowed(src, dst, range, true, same_bubble));
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_can_not_orbit_ball_in_different_bubble
#[cfg(kani)]
#[kani::proof]
fn orbit_rejects_cross_bubble_targets() {
    let src: i64 = kani::any();
    let dst: i64 = kani::any();
    let range: f64 = kani::any();
    let cloaked: bool = kani::any();
    assert!(!orbit_allowed(src, dst, range, cloaked, false));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_not_massive_is_not_an_occlusion
#[cfg(kani)]
#[kani::proof]
fn non_massive_ball_is_never_an_occluder() {
    let cloaked: bool = kani::any();
    assert!(!visibility_occluder(false, cloaked));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_ball_that_is_cloaked_is_not_an_occlusion
#[cfg(kani)]
#[kani::proof]
fn cloaked_ball_is_never_an_occluder() {
    let massive: bool = kani::any();
    assert!(!visibility_occluder(massive, true));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_makes_a_ball_massive
#[cfg(kani)]
#[kani::proof]
fn uncloak_restores_massive_when_not_warping() {
    assert!(uncloak_restores_massive(false));
}

// original-test: python/destiny/test/ballpark/test_visibility.py::test_uncloaking_a_ball_in_warp_does_not_make_it_massive
#[cfg(kani)]
#[kani::proof]
fn uncloak_does_not_restore_massive_in_warp() {
    assert!(!uncloak_restores_massive(true));
}

// original-test: python/destiny/test/ballpark/test_callbacks.py::test_ignores_balls_that_are_not_free
#[cfg(kani)]
#[kani::proof]
fn proximity_rejects_non_free_owners() {
    let target_interactive: bool = kani::any();
    let only_interactives: bool = kani::any();
    assert!(!proximity_eligible(false, target_interactive, only_interactives));
}

// original-test: python/destiny/test/ballpark/test_callbacks.py::test_only_interactives_ignores_non_interactive_balls
#[cfg(kani)]
#[kani::proof]
fn proximity_only_interactives_rejects_noninteractive_targets() {
    assert!(!proximity_eligible(true, false, true));
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_stopped_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_following_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_orbiting_ball
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_missile
// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_stop_ball_in_formation
#[cfg(kani)]
#[kani::proof]
fn stop_sets_every_mode_to_stop() {
    let tag: u8 = kani::any();
    let mode = match tag % 8 {
        0 => MotionMode::Stop,
        1 => MotionMode::GotoDirection,
        2 => MotionMode::GotoPoint,
        3 => MotionMode::Follow,
        4 => MotionMode::FormationFollow,
        5 => MotionMode::Orbit,
        6 => MotionMode::Missile,
        _ => MotionMode::Warp,
    };
    assert_eq!(stopped_mode(mode), MotionMode::Stop);
}

// original-test: python/destiny/test/ballpark/test_movement_controls.py::test_sets_follow_range_as_negative_sum_of_src_and_dst_radii
#[cfg(kani)]
#[kani::proof]
fn missile_follow_range_is_negative_radius_sum_for_finite_safe_inputs() {
    let src: f64 = kani::any();
    let dst: f64 = kani::any();
    kani::assume(src.is_finite() && dst.is_finite());
    kani::assume(src >= 0.0 && dst >= 0.0);
    kani::assume(src <= f64::MAX - dst);
    let expected = -(src + dst);
    assert_eq!(missile_follow_range(src, dst).to_bits(), expected.to_bits());
}
