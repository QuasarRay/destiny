//! Original rejection result and in/out parameter frame. The numeric pow/sqrt
//! body, admission/caller, geometry, adapters and complete query remain separate.
#[path = "../../../src/accuracy.rs"]
mod production;

#[cfg(kani)]
#[kani::proof]
fn total_rejected_accuracy_fields() {
    let optimal: f64 = kani::any();
    let tracking: f64 = kani::any();
    let fields = production::rejected(optimal, tracking);
    assert_eq!(fields[0], 0.0);
    assert_eq!(fields[1].to_bits(), optimal.to_bits());
    assert_eq!(fields[2].to_bits(), tracking.to_bits());
    kani::cover!(
        optimal.is_nan() && tracking.is_nan(),
        "nonfinite in/out values preserved"
    );
    kani::cover!(
        optimal.to_bits() == (-0.0f64).to_bits(),
        "signed-zero optimal range preserved"
    );
    kani::cover!(
        tracking == f64::INFINITY,
        "infinite tracking input preserved"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_accuracy_is_zero_when_ball_targets_itself
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn original_self_target_rejection_fields() {
    let fields = production::rejected(1.0, 1.0);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:189
    assert_eq!(fields[0], 0.0);
    assert_eq!(fields[1], 1.0);
    assert_eq!(fields[2], 1.0);
    #[cfg(kani)]
    kani::cover!(true, "original self-target rejection fields feasible");
}
