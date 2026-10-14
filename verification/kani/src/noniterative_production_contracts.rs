// Copyright © 2015, 2023 CCP ehf. Original Destiny MIT license retained.
//! Complete original stopped-overlap coordinate trace on the actual native
//! gradient kernel. Scope: 2 source fixture bodies,10ticks, dt1, friction1e6.
//! Allocation/partition, IO, orientation, native adapters, concurrent ownership
//! and every other operation remain separate correspondence obligations.
use crate::{iterative_collision::Body, noniterative_collision};
use destiny_original_spec::{OriginalMode, Vec3, python_rounded_equal};

fn body(id: i64) -> Body {
    let mass = 13_000_000.0;
    let agility = 0.9_f32;
    Body {
        id,
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        acceleration: Vec3::ZERO,
        collision_acceleration: Vec3::ZERO,
        mass,
        agility,
        radius: 2.0,
        max_velocity: 10.0,
        time_factor: (-1_000_000.0 / (mass * f64::from(agility))).exp(),
        is_free: true,
        is_massive: true,
        is_cloaked: false,
        is_global: false,
        is_space_junk: false,
        is_moribund: false,
        bubble: 0,
        mode: OriginalMode::Stop,
        owner_id: 0,
        follow_id: 0,
    }
}

fn assert_source_points(actual: &[Vec3], expected: &[Vec3]) {
    let limit = actual.len().min(expected.len());
    for index in 0..limit {
        if actual[index] != expected[index] {
            for (a, b) in [
                (actual[index].x, expected[index].x),
                (actual[index].y, expected[index].y),
                (actual[index].z, expected[index].z),
            ] {
                assert_eq!(python_rounded_equal(a, b, 4), Some(true));
            }
        }
    }
    assert_eq!(actual.len(), expected.len());
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCollision::test_stopped_balls_with_same_location
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(257))]
#[cfg_attr(not(kani), test)]
fn original_stopped_overlap_full_trace_uses_production_gradient() {
    let mut bodies = [body(1), body(2)];
    let mut a = Vec::new();
    let mut b = Vec::new();
    for _ in 0..10 {
        for body in &mut bodies {
            body.velocity = destiny_original_spec::original_stop_velocity(body.velocity);
        }
        let result = noniterative_collision::resolve(&bodies, &[], 1_000_000.0, 1.0, false);
        assert!(result.is_ok());
        let result = result.unwrap();
        for index in 0..2 {
            let (position, velocity) = result[index].expect("original live body result");
            bodies[index].position = position;
            bodies[index].velocity = velocity;
        }
        a.push(bodies[0].position);
        b.push(bodies[1].position);
    }
    let expected = [
        1.121144335565424,
        3.2401005010556143,
        5.185473058408457,
        6.9714818718687654,
        8.61118192299853,
        10.116558737162107,
        11.498615992731432,
        12.76745595339809,
        13.93235331151902,
        15.001822982259961,
    ];
    let expected_a: Vec<_> = expected
        .iter()
        .map(|&x| Vec3 { x, y: 0.0, z: 0.0 })
        .collect();
    let expected_b: Vec<_> = expected
        .iter()
        .map(|&x| Vec3 {
            x: -x,
            y: 0.0,
            z: 0.0,
        })
        .collect();
    assert_source_points(&a, &expected_a);
    assert_source_points(&b, &expected_b);
    #[cfg(kani)]
    kani::cover!(
        a.len() == 10 && b.len() == 10,
        "all original overlap points and source helper comparisons completed"
    );
}
