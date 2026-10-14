//! Actual identity cast and harmonic mode-action kernels. Complete mode event,
//! registry/generation/lease/snapshot/Python/ABI correspondences remain callers.
#[path = "../../../src/harmonic.rs"]
mod production;

#[cfg(kani)]
#[kani::proof]
fn total_harmonic_identity_and_action() {
    let harmonic: i64 = kani::any();
    let corporation: i32 = kani::any();
    let alliance: i32 = kani::any();
    let mode: u8 = kani::any();
    let field: bool = kani::any();
    let fields = production::fields(harmonic, corporation, alliance);
    assert_eq!(fields[0], harmonic);
    assert_eq!(fields[1], corporation as i64);
    assert_eq!(fields[2], alliance as i64);
    let action = production::action(mode, field);
    assert_eq!(action == 10, field);
    assert_eq!(action == 2, !field && mode == 10);
    assert_eq!(action == 255, !field && mode != 10);
    kani::cover!(
        corporation == i32::MIN && alliance == i32::MAX,
        "signed32identity boundaries preserved"
    );
    kani::cover!(field && mode == 1, "Follow to Field needs Stop then Field");
    kani::cover!(!field && mode == 10, "remove Field requests Stop");
    kani::cover!(
        !field && mode == 4,
        "non-Field mode preserved without a Stop"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_harmonic_make_forcefield
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn original_harmonic_field_values_and_action() {
    let fields = production::fields(1, 2, 3);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:153
    assert_eq!(fields[0], 1);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:154
    assert_eq!(fields[1], 2);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:155
    assert_eq!(fields[2], 3);
    // Original156mode assertion additionally requires native Stop/Field
    // callback publication; this checks only the consumed action decision.
    assert_eq!(production::action(2, true), 10);
    #[cfg(kani)]
    kani::cover!(true, "original harmonic fields and Field action feasible");
}
