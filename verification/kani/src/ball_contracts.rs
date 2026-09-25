use destiny_original_spec::{
    MAX_ORIGINAL_FORMATION_SLOTS, MotionMode, ORIGINAL_TEN_BILLION,
    child_count_after_capsule_attempt, child_count_after_successful_add,
    identity_rotated_vector, mini_capsule_radius_accepted,
    next_sequential_formation_slot, original_ball_defaults,
    proximity_sensor_accepted, slot_reused_after_free,
};

// original-test: python/destiny/test/test_ball.py::test_can_construct
#[cfg(kani)]
#[kani::proof]
fn original_ball_construction_has_a_total_default_state() {
    let _ = original_ball_defaults();
}

// original-test: python/destiny/test/test_ball.py::test_default_values
#[cfg(kani)]
#[kani::proof]
fn original_ball_default_values_are_exact() {
    let d = original_ball_defaults();
    assert_eq!(d.id, 0);
    assert_eq!(d.mass.to_bits(), 0.0_f64.to_bits());
    assert_eq!(d.radius.to_bits(), 0.0_f64.to_bits());
    assert_eq!(d.max_velocity.to_bits(), 0.0_f64.to_bits());
    assert!(!d.is_free && !d.is_global && !d.is_massive && !d.is_interactive);
    assert!(!d.is_cloaked && !d.is_moribund);
    assert_eq!(d.harmonic, -1);
    assert_eq!(d.corporation_id, -1);
    assert_eq!(d.alliance_id, -1);
    for coordinate in d.position {
        assert_eq!(coordinate.to_bits(), ORIGINAL_TEN_BILLION.to_bits());
    }
    for velocity in d.velocity {
        assert_eq!(velocity.to_bits(), 0.0_f64.to_bits());
    }
    assert_eq!(d.yaw.to_bits(), 0.0_f64.to_bits());
    assert_eq!(d.pitch.to_bits(), 0.0_f64.to_bits());
    assert_eq!(d.roll.to_bits(), 0.0_f64.to_bits());
    assert_eq!(d.agility.to_bits(), 1.0_f64.to_bits());
    assert_eq!(d.speed_fraction.to_bits(), 1.0_f64.to_bits());
    assert_eq!(d.mode, MotionMode::Stop);
    assert_eq!(d.goto, [0.0; 3]);
    assert_eq!(d.follow_id, 0);
    assert_eq!(d.follow_range.to_bits(), 10.0_f64.to_bits());
    assert_eq!(d.owner_id, 0);
    assert_eq!(d.effect_stamp, 0);
    assert_eq!(d.new_bubble_id, -1);
    assert_eq!(d.old_bubble_id, -1);
    assert_eq!(d.formation_id, 255);
    assert_eq!(d.mini_ball_count, 0);
    assert!(!d.has_ballpark);
}

// original-test: python/destiny/test/test_ball.py::test_can_add_miniball
#[cfg(kani)]
#[kani::proof]
fn adding_a_miniball_increments_child_count_once() {
    let current: u16 = kani::any();
    kani::assume(current < u16::MAX);
    assert_eq!(child_count_after_successful_add(current), Some(current + 1));
}

// original-test: python/destiny/test/test_ball.py::test_can_add_minicapsule
#[cfg(kani)]
#[kani::proof]
fn adding_a_positive_radius_minicapsule_increments_child_count_once() {
    let current: u16 = kani::any();
    let radius: f64 = kani::any();
    kani::assume(current < u16::MAX);
    kani::assume(radius.is_finite() && radius > 0.0);
    assert!(mini_capsule_radius_accepted(radius));
    assert_eq!(
        child_count_after_capsule_attempt(current, radius),
        Some(current + 1)
    );
}

// original-test: python/destiny/test/test_ball.py::test_can_not_add_minicapsule_with_negative_radius
#[cfg(kani)]
#[kani::proof]
fn non_positive_minicapsule_radius_is_rejected_without_count_change() {
    let current: u16 = kani::any();
    let radius: f64 = kani::any();
    kani::assume(radius.is_finite() && radius <= 0.0);
    assert!(!mini_capsule_radius_accepted(radius));
    assert_eq!(child_count_after_capsule_attempt(current, radius), Some(current));
}

// original-test: python/destiny/test/test_ball.py::test_get_rotated_vector_returns_original_vector_if_there_is_no_rotation
#[cfg(kani)]
#[kani::proof]
fn identity_rotation_preserves_every_vector_bit_pattern() {
    let vector = [kani::any::<u64>(), kani::any::<u64>(), kani::any::<u64>()];
    assert_eq!(identity_rotated_vector(vector), vector);
}

// original-test: python/destiny/test/test_ball.py::test_add_proximity_sensor
#[cfg(kani)]
#[kani::proof]
fn original_test_proximity_sensor_arguments_are_accepted() {
    assert!(proximity_sensor_accepted(0.0, 5.0));
}

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot_returns_negative_one_when_ball_is_not_set_up_properly
#[cfg(kani)]
#[kani::proof]
fn formation_slot_reservation_rejects_unconfigured_ball() {
    let reserved: u8 = kani::any();
    let slots: u8 = kani::any();
    assert_eq!(
        next_sequential_formation_slot(reserved, slots, false, false, false),
        -1
    );
}

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot
#[cfg(kani)]
#[kani::proof]
fn first_slot_of_valid_nonempty_formation_is_zero() {
    let slots: u8 = kani::any();
    kani::assume(slots > 0 && slots <= MAX_ORIGINAL_FORMATION_SLOTS);
    assert_eq!(next_sequential_formation_slot(0, slots, true, true, true), 0);
}

// original-test: python/destiny/test/test_ball.py::test_slots_are_reserved_in_incremental_order
#[cfg(kani)]
#[kani::proof]
fn formation_slots_are_reserved_in_incremental_order() {
    let slots: u8 = kani::any();
    let reserved: u8 = kani::any();
    kani::assume(slots > 0 && slots <= MAX_ORIGINAL_FORMATION_SLOTS);
    kani::assume(reserved < slots);
    assert_eq!(
        next_sequential_formation_slot(reserved, slots, true, true, true),
        reserved as i8
    );
}

// original-test: python/destiny/test/test_ball.py::test_reserving_too_many_formation_slots_fails
#[cfg(kani)]
#[kani::proof]
fn formation_reservation_fails_when_all_slots_are_reserved() {
    let slots: u8 = kani::any();
    kani::assume(slots <= MAX_ORIGINAL_FORMATION_SLOTS);
    assert_eq!(
        next_sequential_formation_slot(slots, slots, true, true, true),
        -1
    );
}

// original-test: python/destiny/test/test_ball.py::test_free_formation_slot
#[cfg(kani)]
#[kani::proof]
fn freed_formation_slot_is_reused() {
    let slots: u8 = kani::any();
    let freed: u8 = kani::any();
    kani::assume(slots > 0 && slots <= MAX_ORIGINAL_FORMATION_SLOTS);
    kani::assume(freed < slots);
    assert_eq!(
        slot_reused_after_free(freed, slots, true, true, true),
        freed as i8
    );
}
