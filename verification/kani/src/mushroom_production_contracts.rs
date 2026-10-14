//! Complete original radius trace on the actual creation/time/radius kernels.
//! Owner lookup, object insertion, publication, leases and engine/FFI boundary
//! are separate required relations. powf uses the real platform float operation.
use destiny_original_spec::{Vec3, python_almost_equal};
#[path = "../../../src/mushroom.rs"]
mod production;

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAddMushroom::test_mushroom_expands_correctly
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn original_mushroom_all_seven_radius_assertions() {
    let initial = production::initial_ball(Vec3 {
        x: 1e10,
        y: 1e10,
        z: 1e10,
    });
    let mut radius = f64::from(initial.radius);
    assert_eq!(radius, 0.0);
    radius = f64::from(
        production::radius_at_tick(0, 0, 1000, 10.0, 5.0)
            .unwrap()
            .unwrap(),
    );
    assert_eq!(radius, 0.0);
    radius = f64::from(
        production::radius_at_tick(1, 0, 1000, 10.0, 5.0)
            .unwrap()
            .unwrap(),
    );
    assert_eq!(
        python_almost_equal(radius, 6.687403202056885, 7),
        Some(true)
    );
    radius = f64::from(
        production::radius_at_tick(2, 0, 1000, 10.0, 5.0)
            .unwrap()
            .unwrap(),
    );
    assert_eq!(
        python_almost_equal(radius, 7.952707290649414, 7),
        Some(true)
    );
    radius = f64::from(
        production::radius_at_tick(3, 0, 1000, 10.0, 5.0)
            .unwrap()
            .unwrap(),
    );
    assert_eq!(
        python_almost_equal(radius, 8.801117897033691, 7),
        Some(true)
    );
    radius = f64::from(
        production::radius_at_tick(4, 0, 1000, 10.0, 5.0)
            .unwrap()
            .unwrap(),
    );
    assert_eq!(
        python_almost_equal(radius, 9.457415580749512, 7),
        Some(true)
    );
    // The final Evolve removes the object and keeps its prior radius through
    // a retained lease. The kernel emits removal, not an invented final radius.
    assert_eq!(production::radius_at_tick(5, 0, 1000, 10.0, 5.0), Ok(None));
    assert_eq!(
        python_almost_equal(radius, 9.457415580749512, 7),
        Some(true)
    );
    #[cfg(kani)]
    kani::cover!(
        true,
        "all original mushroom radius assertions and expiration reached"
    );
}
