// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// Copyright © 2023 CCP ehf. Original Destiny MIT license retained.
//! Actual native kernel, concrete unchanged original substep assertions.
//! Scope: 3 spheres, all 20 configured passes, dt=1 and original fixture values.
//! Caller/adapters, partition enumeration, native publication, shapes, libm and
//! concurrent operation ownership remain separate obligations. No full-case
//! production parity is asserted by this kernel harness alone.

use crate::iterative_collision as production;
use destiny_original_spec::{OriginalMode, Vec3, python_almost_equal};

fn body(id: i64, z: f64, vz: f64, dt: f64) -> production::Body {
    // helpers.create_space_ball defaults, including the f32 agility field.
    let mass = 13_000_000.0;
    let agility = 0.9_f32;
    production::Body {
        id,
        position: Vec3 { x: 0.0, y: 0.0, z },
        velocity: Vec3 {
            x: 0.0,
            y: 0.0,
            z: vz,
        },
        acceleration: Vec3::ZERO,
        collision_acceleration: Vec3::ZERO,
        mass,
        agility,
        radius: 2.0,
        max_velocity: 100.0,
        time_factor: (-1e-10 * dt / (mass * f64::from(agility))).exp(),
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
        cached_dead: false,
    }
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_substep_collision
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(257))]
#[cfg_attr(not(kani), test)]
fn original_three_sphere_substeps_use_actual_ordered_kernel() {
    // Original creation order: a, c, b. Retain their IDs when sorted, so the
    // solver cannot satisfy the claim by processing geometric order instead.
    let mut bodies = vec![
        body(1, 0.0, 20.0, 1.0),
        body(2, 10.0, 0.0, 1.0),
        body(3, 5.0, 0.0, 1.0),
    ];
    let result = production::resolve(&mut bodies, &[], 1e-10, 1.0, 20, false, &mut Vec::new());
    assert_eq!(result.len(), 3);
    let (a, av) = result[0].expect("original a remains live");
    let (c, cv) = result[1].expect("original c remains live");
    let (b, bv) = result[2].expect("original b remains live");
    // Exact original unittest.assertAlmostEqual(..., places=0), six assertions.
    assert_eq!(python_almost_equal(av.z, 0.0, 0), Some(true));
    assert_eq!(python_almost_equal(a.z, 1.0, 0), Some(true));
    assert_eq!(python_almost_equal(bv.z, 0.0, 0), Some(true));
    assert_eq!(python_almost_equal(b.z, 6.0, 0), Some(true));
    assert_eq!(python_almost_equal(cv.z, 20.0, 0), Some(true));
    assert_eq!(python_almost_equal(c.z, 28.0, 0), Some(true));
    #[cfg(kani)]
    kani::cover!(
        python_almost_equal(a.z, 1.0, 0) == Some(true)
            && python_almost_equal(c.z, 28.0, 0) == Some(true),
        "original three-sphere transfer reached after all source assertions"
    );
}

#[test]
fn native_collection_is_not_limited_to_eight_bodies_and_preserves_dt() {
    let dt = 0.5;
    let mut bodies = vec![
        body(1, 0.0, 20.0, dt),
        body(2, 10.0, 0.0, dt),
        body(3, 5.0, 0.0, dt),
    ];
    for id in 4..=12 {
        bodies.push(body(id, 1000.0 * id as f64, 0.0, dt));
    }
    let result = production::resolve(&mut bodies, &[], 1e-10, dt, 32, true, &mut Vec::new());
    assert_eq!(result.len(), 12);
    let (a, av) = result[0].unwrap();
    let (c, cv) = result[1].unwrap();
    let (b, bv) = result[2].unwrap();
    assert!((a.z - 1.0).abs() < 1e-8 && av.z.abs() < 1e-8);
    assert!((b.z - 6.0).abs() < 1e-8 && bv.z.abs() < 1e-8);
    assert!((c.z - 18.0).abs() < 1e-8 && (cv.z - 20.0).abs() < 1e-8);
    for index in 3..12 {
        // Original integration still evaluates its floating-point expression
        // for stationary bodies; it can round a coordinate by one ULP. The
        // frame relation is the source free-flight result, not ideal reals.
        let initial = bodies[index];
        let expected = destiny_original_spec::original_integrate_duration(
            initial.position,
            initial.velocity,
            Vec3::ZERO,
            initial.mass * f64::from(initial.agility),
            1e-10,
            initial.time_factor,
            dt,
            dt,
        );
        assert_eq!(result[index], Some(expected));
    }
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_from_left
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(257))]
#[cfg_attr(not(kani), test)]
fn original_capsule_left_keeps_every_tick_and_assertion() {
    use destiny_original_spec::{
        OriginalCapsule, OriginalParkBall, OriginalStaticCollisionShape, original_goto_thrust,
    };
    let a = Vec3 {
        x: 100.0,
        y: 100.0,
        z: 0.0,
    };
    let b = Vec3 {
        x: 200.0,
        y: 100.0,
        z: 0.0,
    };
    let center = a.add(b).scale(0.5);
    let capsule = OriginalCapsule {
        id: 2,
        parent_id: 0,
        a,
        b,
        radius: 10.0,
        ab: b.sub(a),
        position: center,
        old_position: center,
    };
    let geometry = [production::StaticGeometry {
        owner: 0,
        shape: OriginalStaticCollisionShape::Capsule(capsule),
    }];
    let mut moving = body(1, 0.0, 0.0, 1.0);
    moving.position = Vec3 {
        x: 80.0,
        y: 100.0,
        z: 0.0,
    };
    moving.max_velocity = 10.0;
    moving.mode = OriginalMode::Goto;
    moving.time_factor = (-1_000_000.0 / (moving.mass * f64::from(moving.agility))).exp();
    let mut coordinates = Vec::new();
    for _ in 0..20 {
        let input = OriginalParkBall {
            id: 1,
            in_park: true,
            is_free: true,
            is_massive: true,
            position: moving.position,
            velocity: moving.velocity,
            radius: moving.radius,
            mass: moving.mass,
            agility: moving.agility,
            max_velocity: moving.max_velocity,
            speed_fraction: 0.95_f32,
            time_factor: moving.time_factor,
            mode: OriginalMode::Goto,
            ..OriginalParkBall::default()
        };
        moving.acceleration = original_goto_thrust(
            &input,
            Vec3 {
                x: 300.0,
                y: 100.0,
                z: 0.0,
            },
            1_000_000.0,
            1.0,
            false,
        );
        moving.collision_acceleration = Vec3::ZERO;
        let mut bodies = [moving];
        let result = production::resolve(
            &mut bodies,
            &geometry,
            1_000_000.0,
            1.0,
            20,
            false,
            &mut Vec::new(),
        );
        let (position, velocity) = result[0].expect("original capsule tick succeeds");
        coordinates.push(position);
        moving.position = position;
        moving.velocity = velocity;
    }
    assert_eq!(coordinates.len(), 20);
    // Original evaluates these assertions after saving the full twenty points.
    for point in coordinates {
        let distance = point.sub(a).length();
        assert!(1.11 * distance > f64::from(moving.radius) + f64::from(capsule.radius));
        assert_eq!(python_almost_equal(point.y, 100.0, 4), Some(true));
        assert_eq!(python_almost_equal(point.z, 0.0, 2), Some(true));
    }
    #[cfg(kani)]
    kani::cover!(
        true,
        "all twenty original capsule-left points and comparisons executed"
    );
}
