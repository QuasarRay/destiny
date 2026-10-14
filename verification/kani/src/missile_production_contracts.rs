//! Actual source-backed MissileFollow admission/fields. This verifies kernels,
//! retaining registry, ordered Stop/mode events, adapters and integration gaps.
#[path = "../../../src/missile.rs"]
mod production;
use destiny_original_spec::Vec3;

#[cfg(kani)]
#[kani::proof]
fn total_missile_admission() {
    let src: i64 = kani::any();
    let target: i64 = kani::any();
    let owner: i64 = kani::any();
    let src_exists: bool = kani::any();
    let target_exists: bool = kani::any();
    let moribund: bool = kani::any();
    let accepted = production::admitted(src, target, owner, src_exists, target_exists, moribund);
    assert_eq!(
        accepted,
        src_exists
            && target_exists
            && !moribund
            && src != target
            && src != owner
            && target != owner
    );
    kani::cover!(accepted && src < 0, "valid negative missile ID");
    kani::cover!(accepted && owner < 0, "negative owner ID is permitted");
    kani::cover!(
        moribund && src_exists && target_exists,
        "moribund target rejected"
    );
    kani::cover!(!src_exists, "missing missile rejected");
    kani::cover!(!target_exists, "missing target rejected");
    kani::cover!(src == owner, "missile-owner alias rejected");
    kani::cover!(src == target, "missile-target alias rejected");
    kani::cover!(owner == target, "owner-target alias rejected");
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_sets_ids_correctly
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn original_missile_follow_ids_fields() {
    // helpers.BallparkTestCase.add_balls(3), IDs1/2/3, zero radii and velocity.
    assert!(production::admitted(1, 2, 3, true, true, false));
    let fields = production::follow_fields(2, 3, 0, 0.0, 0.0, Vec3::ZERO);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:41
    assert_eq!(fields.follow_id, 2);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:42
    assert_eq!(fields.owner_id, 3);
    assert_eq!(fields.effect_stamp, 0);
    assert_eq!(fields.follow_range.to_bits(), (-0.0f32).to_bits());
    assert_eq!(fields.destination, Vec3::ZERO);
    assert_eq!(fields.speed_fraction, 1.0);
    #[cfg(kani)]
    kani::cover!(true, "original missile IDs field scenario reached");
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_direction_of_non_aimed_missiles
// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_additional_velocity_of_missile_is_the_same_as_the_max_velocity_of_its_owner_if_above_limit
// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_initial_velocity_of_owner_has_impact_of_non_aimed_missile_direction_and_speed
// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_direction_of_aimed_missiles
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(test, test)]
fn original_launcher_velocity_examples() {
    // Source helpers place owner at1e10, target at1e10+10000 on X;
    // default identity orientation rotates (0,0,-1) to itself. Quaternion and
    // position adapters are separate obligations; no Kani assumptions are used.
    let offset = Some(Vec3 {
        x: 10000.0,
        y: 0.0,
        z: 0.0,
    });
    let stationary = Vec3 {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };
    let (velocity, maximum) =
        production::launch_velocity(Vec3::ZERO, 0.0, stationary, offset, 0, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:313
    assert_eq!(velocity.x, 0.0);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:314
    assert_eq!(velocity.y, 0.0);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:315
    assert_eq!(
        destiny_original_spec::python_almost_equal(velocity.z, -150.0, 4),
        Some(true)
    );
    assert_eq!(maximum, 0.0);
    let (velocity, maximum) =
        production::launch_velocity(Vec3::ZERO, 1234.0, stationary, offset, 0, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:324
    assert_eq!(velocity.x, 0.0);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:325
    assert_eq!(velocity.y, 0.0);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:326
    assert_eq!(
        destiny_original_spec::python_almost_equal(velocity.z, -1234.0, 3),
        Some(true)
    );
    assert_eq!(maximum, 1234.0);
    let (velocity, _) = production::launch_velocity(
        Vec3 {
            x: 100.0,
            y: 0.0,
            z: 0.0,
        },
        0.0,
        stationary,
        offset,
        0,
        false,
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:335
    assert_eq!(
        velocity,
        Vec3 {
            x: 250.0,
            y: 0.0,
            z: 0.0
        }
    );
    let (velocity, _) = production::launch_velocity(Vec3::ZERO, 0.0, stationary, offset, 1, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:343
    assert_eq!(
        velocity,
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    );
    #[cfg(kani)]
    kani::cover!(true, "all four original launcher velocity examples reached");
}
