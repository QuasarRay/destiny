//! Concrete source command decision kernel, without assuming finite inputs.
//! Complete command publication, sensor/rotation/events and ABI remain callers.
#[path = "../../../src/warp_entry.rs"]
mod production;

#[cfg(kani)]
#[kani::proof]
fn total_warp_command_action() {
    let distance: f64 = kani::any();
    let warping: bool = kani::any();
    let same: bool = kani::any();
    let action = production::command_action(distance, warping, same);
    assert_eq!(action == 1, distance < 1e10);
    assert_eq!(action == 0, !(distance < 1e10) && warping && same);
    assert_eq!(action == 3, !(distance < 1e10) && (!warping || !same));
    kani::cover!(
        distance == 1e10 && action == 3,
        "exact distance threshold starts Warp"
    );
    kani::cover!(distance == 0.0 && action == 1, "zero distance becomes Goto");
    kani::cover!(
        distance.is_nan() && action == 3,
        "NaN derived distance follows source comparison"
    );
    kani::cover!(
        distance == f64::INFINITY && action == 0,
        "redundant infinite derived distance"
    );
    kani::cover!(
        action == 0 && distance.is_finite(),
        "finite redundant Warp command"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestEntityWarpIn::test_warps_immediately
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn original_entity_immediate_warp_decision_and_fields() {
    use destiny_original_spec::Vec3;
    let position = Vec3 {
        x: 1e10,
        y: 1e10,
        z: 1e10,
    };
    let target = Vec3::ZERO;
    assert_eq!(
        production::command_action(position.sub(target).norm_squared(), false, false),
        3
    );
    let fields = production::arrival(target, position, target, 0.0, 0)
        .expect("source time0 is representable");
    assert_eq!(fields.0, target);
    assert_eq!(fields.1, Vec3::ZERO);
    assert_eq!(fields.2, 0);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original immediate Warp command and arrival fields feasible"
    );
}
