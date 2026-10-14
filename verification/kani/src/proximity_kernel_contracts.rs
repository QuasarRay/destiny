//! Concrete production boolean/IEEE-754 kernels. Candidate enumeration,
//! observer invocation and whole original scenarios have separate obligations.
#[path = "../../../src/proximity.rs"]
mod production;

#[kani::proof]
fn proximity_flags_preserve_original_partition_branching() {
    let owner_cloaked: bool = kani::any();
    let free: bool = kani::any();
    let interactive: bool = kani::any();
    let only: bool = kani::any();
    let master: bool = kani::any();
    let same: bool = kani::any();
    let global: bool = kani::any();
    let actual = production::candidate_admitted(
        owner_cloaked,
        free,
        interactive,
        only,
        master,
        same,
        global,
    );
    if only {
        assert_eq!(actual, interactive);
    } else {
        assert_eq!(
            actual,
            (!master || same || global) && (owner_cloaked || free)
        );
    }
    kani::cover!(
        only && interactive && !free && master && !same && !global,
        "source interactive-only branch bypasses fixed and bubble filters"
    );
    kani::cover!(
        !only && owner_cloaked && !free && same,
        "cloaked owner can detect a fixed candidate"
    );
    kani::cover!(
        !only && !owner_cloaked && !free,
        "noncloaked owner rejects fixed candidate"
    );
    kani::cover!(
        !only && !master && free && !same && !global,
        "client branch admits free candidate across bubble labels"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_callback_gets_called_when_period_has_passed
// Only the timing conjunct is checked here; the existing scenario harness,
// actual observer/phase link and runtime correspondence remain required.
#[kani::proof]
fn sensor_elapsed_step_preserves_double_comparison_and_zero_reset() {
    let prior: f64 = kani::any();
    let dt: f64 = kani::any();
    let period: f64 = kani::any();
    let actual = production::sensor_elapsed_step(prior, dt, period);
    let valid =
        prior.is_finite() && prior >= 0.0 && dt.is_finite() && dt > 0.0 && period.is_finite();
    assert_eq!(actual.is_some(), valid);
    if valid {
        let (elapsed, due) = actual.unwrap();
        let sum = prior + dt;
        assert_eq!(due, !(sum < period));
        if due {
            assert_eq!(elapsed.to_bits(), 0.0_f64.to_bits());
        } else {
            assert_eq!(elapsed.to_bits(), sum.to_bits());
        }
    } else {
        assert!(actual.is_none());
    }
    let (first, called) = production::sensor_elapsed_step(0.0, 1.0, 2.0).unwrap();
    assert_eq!(first, 1.0);
    assert!(!called);
    let (second, called) = production::sensor_elapsed_step(first, 1.0, 2.0).unwrap();
    assert_eq!(second, 0.0);
    assert!(called);
    kani::cover!(
        prior == 0.0 && dt == 1.0 && period == 2.0,
        "source first tick waits"
    );
    kani::cover!(
        prior == 1.0 && dt == 1.0 && period == 2.0,
        "source second tick invokes"
    );
    kani::cover!(
        period == 0.0 && dt > 0.0 && prior == 0.0,
        "source zero period is due"
    );
    kani::cover!(
        prior.is_nan(),
        "native invalid elapsed is rejected before addition"
    );
    kani::cover!(
        dt.is_nan(),
        "native invalid time step is rejected before addition"
    );
    kani::cover!(
        period.is_nan(),
        "native invalid period is rejected before comparison"
    );
    kani::cover!(dt == 0.0, "native zero time step is rejected");
    kani::cover!(prior < 0.0, "native negative elapsed is rejected");
    kani::cover!(
        valid && prior == f64::MAX && dt == f64::MAX,
        "positive finite addition overflow still resets safely"
    );
}

#[kani::proof]
fn strict_range_boundary_and_notification_transitions_match_source() {
    let distance: f64 = kani::any();
    let reach: f64 = kani::any();
    let actual = production::strict_range_contains(distance, reach);
    let valid = distance.is_finite() && distance >= 0.0 && reach.is_finite() && reach >= 0.0;
    assert_eq!(actual.is_some(), valid);
    if valid {
        assert_eq!(actual, Some(distance < reach * reach));
    } else {
        assert!(actual.is_none());
    }
    assert_eq!(production::strict_range_contains(121.0, 11.0), Some(false));
    assert_eq!(production::strict_range_contains(1.0, 10.0), Some(true));
    let prior: bool = kani::any();
    let inside: bool = kani::any();
    let enabled: bool = kani::any();
    let (after, send) = production::notification_transition(prior, inside, enabled);
    assert_eq!(after, if enabled { inside } else { prior });
    assert_eq!(send, enabled && prior != inside);
    kani::cover!(
        distance == 121.0 && reach == 11.0,
        "original strict edge is excluded"
    );
    kani::cover!(!prior && inside && enabled, "entry notification transition");
    kani::cover!(prior && !inside && enabled, "exit notification transition");
    kani::cover!(!enabled, "disabled notification preserves prior state");
    kani::cover!(
        distance.is_nan(),
        "invalid distance rejected before comparison"
    );
    kani::cover!(reach.is_nan(), "invalid range rejected before square");
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_ball_on_edge_of_range
#[kani::proof]
fn sensor_inclusive_squared_boundary_preserves_original_geometry_comparison() {
    let distance: f64 = kani::any();
    let reach: f64 = kani::any();
    let actual = production::inclusive_range_contains(distance, reach);
    let valid = distance.is_finite() && distance >= 0.0 && reach.is_finite();
    assert_eq!(actual.is_some(), valid);
    if valid {
        assert_eq!(actual, Some(!(distance > reach * reach)));
    } else {
        assert!(actual.is_none());
    }
    assert_eq!(
        production::inclusive_range_contains(100.0, 10.0),
        Some(true)
    );
    assert_eq!(
        production::inclusive_range_contains(121.0, 10.0),
        Some(false)
    );
    kani::cover!(
        distance == 100.0 && reach == 10.0,
        "source inclusive sensor edge is admitted"
    );
    kani::cover!(
        valid && distance > reach * reach,
        "source out of range comparison"
    );
    kani::cover!(
        reach < 0.0 && reach.is_finite() && distance == 0.0,
        "source squared negative live reach"
    );
    kani::cover!(
        reach == f64::MAX && distance == f64::MAX,
        "source square overflows to infinity"
    );
    kani::cover!(
        reach.is_nan(),
        "invalid reach rejected before multiplication"
    );
    kani::cover!(
        distance.is_nan(),
        "invalid distance rejected before comparison"
    );
}

#[kani::proof]
fn follow_notification_annulus_preserves_both_strict_boundaries() {
    let distance: f64 = kani::any();
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    let actual = production::annular_range_contains(distance, hi, range);
    let valid = distance.is_finite()
        && distance >= 0.0
        && hi.is_finite()
        && range.is_finite()
        && range >= 0.0;
    assert_eq!(actual.is_some(), valid);
    if valid {
        let lower = hi - 2.0 * range;
        let lo = if lower < 0.0 { 0.0 } else { lower };
        assert_eq!(actual, Some(distance < hi * hi && distance > lo * lo));
    } else {
        assert!(actual.is_none());
    }
    assert_eq!(
        production::annular_range_contains(25.0, 5.0, 1.0),
        Some(false)
    );
    assert_eq!(
        production::annular_range_contains(9.0, 5.0, 1.0),
        Some(false)
    );
    assert_eq!(
        production::annular_range_contains(16.0, 5.0, 1.0),
        Some(true)
    );
    assert_eq!(
        production::annular_range_contains(0.0, 1.0, 2.0),
        Some(false)
    );
    kani::cover!(
        distance == 16.0 && hi == 5.0 && range == 1.0,
        "source annulus interior"
    );
    kani::cover!(
        distance == 25.0 && hi == 5.0 && range == 1.0,
        "source high edge excluded"
    );
    kani::cover!(
        distance == 9.0 && hi == 5.0 && range == 1.0,
        "source low edge excluded"
    );
    kani::cover!(
        hi == 1.0 && range == 2.0 && distance > 0.0 && distance < 1.0,
        "source lower radius clamps at zero"
    );
    kani::cover!(
        hi == f64::MAX && range == f64::MAX && distance == 1.0,
        "source finite arithmetic overflow retains comparison semantics"
    );
    kani::cover!(range.is_nan(), "invalid notification range rejected");
    kani::cover!(hi.is_nan(), "invalid upper radius rejected");
}

// These explicit partitions collectively retain the preceding full raw-f64
// claim. No desired result or implementation success is assumed. Original
// API nonfinite acceptance and the world/adapter relation remain separate.
#[kani::proof]
fn annular_invalid_inputs_are_rejected_before_arithmetic() {
    let distance: f64 = kani::any();
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    let valid = distance.is_finite()
        && distance >= 0.0
        && hi.is_finite()
        && range.is_finite()
        && range >= 0.0;
    kani::assume(!valid); // Exhaustive invalid-input partition, not an output assumption.
    assert_eq!(
        production::annular_range_contains(distance, hi, range),
        None
    );
    kani::cover!(
        distance.is_nan(),
        "invalid distance partition has a witness"
    );
    kani::cover!(hi.is_nan(), "invalid high radius partition has a witness");
    kani::cover!(range.is_nan(), "invalid range partition has a witness");
    kani::cover!(distance < 0.0, "negative distance partition has a witness");
    kani::cover!(range < 0.0, "negative range partition has a witness");
    kani::cover!(
        hi.is_infinite(),
        "infinite high radius partition has a witness"
    );
}

#[kani::proof]
fn annular_zero_lower_partition_matches_source() {
    let distance: f64 = kani::any();
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    kani::assume(
        distance.is_finite()
            && distance >= 0.0
            && hi.is_finite()
            && range.is_finite()
            && range >= 0.0,
    ); // Valid-input partition; rejection is checked above.
    let lower = hi - 2.0 * range;
    kani::assume(lower <= 0.0); // Source clamp partition includes either signed zero.
    assert_eq!(
        production::annular_range_contains(distance, hi, range),
        Some(distance < hi * hi && distance > 0.0)
    );
    kani::cover!(
        hi == 1.0 && range == 2.0 && distance == 0.25,
        "zero lower radius interior"
    );
    kani::cover!(
        hi == -1.0 && range == 0.0 && distance == 0.25,
        "source squared negative high radius"
    );
    kani::cover!(
        hi == 0.0 && range == 0.0 && distance == 0.0,
        "degenerate empty annulus"
    );
    kani::cover!(
        hi == f64::MAX && range == f64::MAX && distance == 1.0,
        "lower expression overflows negatively and clamps to zero"
    );
}

#[kani::proof]
fn annular_positive_lower_partition_matches_source() {
    let distance: f64 = kani::any();
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    kani::assume(
        distance.is_finite()
            && distance >= 0.0
            && hi.is_finite()
            && range.is_finite()
            && range >= 0.0,
    ); // Valid-input partition; rejection is checked above.
    let lower = hi - 2.0 * range;
    kani::assume(lower > 0.0); // Complement of the clamp/zero partition, not a result assumption.
    assert_eq!(
        production::annular_range_contains(distance, hi, range),
        Some(distance < hi * hi && distance > lower * lower)
    );
    kani::cover!(
        hi == 5.0 && range == 1.0 && distance == 16.0,
        "positive lower radius interior"
    );
    kani::cover!(
        hi == 5.0 && range == 1.0 && distance == 9.0,
        "strict lower edge excluded"
    );
    kani::cover!(
        hi == 5.0 && range == 1.0 && distance == 25.0,
        "strict upper edge excluded"
    );
    kani::cover!(
        hi == f64::MAX && range == 0.0 && distance == f64::MAX,
        "degenerate overflowing squares exclude every finite distance"
    );
}

#[kani::proof]
fn annular_partitions_exhaust_the_raw_input_domain() {
    let distance: f64 = kani::any();
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    let valid = distance.is_finite()
        && distance >= 0.0
        && hi.is_finite()
        && range.is_finite()
        && range >= 0.0;
    if valid {
        let lower = hi - 2.0 * range;
        assert!(!lower.is_nan());
        assert!((lower <= 0.0) != (lower > 0.0));
        if lower > 0.0 {
            assert!(lower.is_finite());
        }
        kani::cover!(lower <= 0.0, "valid clamp partition is feasible");
        kani::cover!(lower > 0.0, "valid positive-lower partition is feasible");
    }
    kani::cover!(!valid, "invalid partition is feasible");
}

// Contract checking uses the actual production lower-radius body on all inputs.
// The later compositional harness is conditional on this contract's current
// complete receipt; neither a declared contract nor stub alone is proof.
#[kani::proof_for_contract(production::annulus_lower_radius)]
fn annulus_lower_radius_contract_preserves_original_double_clamp() {
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    let actual = production::annulus_lower_radius(hi, range);
    let valid = hi.is_finite() && range.is_finite() && range >= 0.0;
    assert_eq!(actual.is_some(), valid);
    if valid {
        let lower = hi - 2.0 * range;
        assert_eq!(
            actual.unwrap().to_bits(),
            (if lower < 0.0 { 0.0 } else { lower }).to_bits()
        );
    }
    kani::cover!(
        hi == 5.0 && range == 1.0,
        "positive lower radius contract domain"
    );
    kani::cover!(
        hi == 1.0 && range == 2.0,
        "clamped lower radius contract domain"
    );
    kani::cover!(hi.is_nan(), "invalid high radius contract domain");
    kani::cover!(range.is_nan(), "invalid range contract domain");
    kani::cover!(
        hi == f64::MAX && range == f64::MAX,
        "original lower arithmetic overflow contract domain"
    );
}

#[kani::proof]
#[kani::stub_verified(production::annulus_lower_radius)]
fn annulus_composition_preserves_both_original_strict_comparisons() {
    let distance: f64 = kani::any();
    let hi: f64 = kani::any();
    let range: f64 = kani::any();
    let actual = production::annular_range_contains(distance, hi, range);
    let valid = distance.is_finite()
        && distance >= 0.0
        && hi.is_finite()
        && range.is_finite()
        && range >= 0.0;
    assert_eq!(actual.is_some(), valid);
    if valid {
        let lower = hi - 2.0 * range;
        let lo = if lower < 0.0 { 0.0 } else { lower };
        assert_eq!(actual, Some(distance < hi * hi && distance > lo * lo));
    }
    kani::cover!(
        distance == 16.0 && hi == 5.0 && range == 1.0,
        "compositional source interior"
    );
    kani::cover!(
        distance == 9.0 && hi == 5.0 && range == 1.0,
        "compositional lower edge"
    );
    kani::cover!(
        distance == 25.0 && hi == 5.0 && range == 1.0,
        "compositional high edge"
    );
    kani::cover!(distance.is_nan(), "compositional distance rejection");
}
