use destiny_original_spec::{
    ParkState, add_ball_is_permitted, apply_non_negative_setter, clamp_speed_fraction,
};

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

#[cfg(kani)]
#[kani::proof]
fn speed_fraction_below_zero_clamps_to_zero() {
    let requested: f64 = kani::any();
    kani::assume(requested.is_finite());
    kani::assume(requested < 0.0);
    assert_eq!(clamp_speed_fraction(requested).to_bits(), 0.0_f64.to_bits());
}

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
