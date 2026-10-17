//! Original stop/warp fixture setup, order, modes and expected literals,
//! plus independent unchanged-C++ stop/integration bits, executed by the
//! production Rust/Bevy world without Python or Blue.
use avian3d::prelude::{LinearVelocity, MaxLinearSpeed, Position};
use destiny_bevy_compat::{
    CompatOptions, CompatRequest, CompatRuntime, DestinyBallId, DestinyMaxVelocity,
};
use serde_json::{Value, json};

fn park(runtime: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    runtime
        .dispatch(CompatRequest {
            title: format!("destiny.Ballpark.{name}"),
            operation: "call".into(),
            target: Some("park:0".into()),
            args,
            kwargs: Default::default(),
        })
        .unwrap()
}

fn property(runtime: &mut CompatRuntime, name: &str, value: Value) {
    runtime
        .dispatch(CompatRequest {
            title: format!("destiny.Ball.{name}"),
            operation: "set".into(),
            target: Some("ball:1".into()),
            args: vec![value],
            kwargs: Default::default(),
        })
        .unwrap();
}

fn world(iterative: bool) -> CompatRuntime {
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        ..Default::default()
    })
    .unwrap();
    // helpers.create_space_ball: first a zero-initialized non-free ball, then
    // SetBallFree and the original property assignments in their original order.
    park(
        &mut runtime,
        "AddBall",
        vec![
            json!(1),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(false),
            json!(false),
            json!(false),
            json!(false),
            json!(false),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
        ],
    );
    park(&mut runtime, "SetBallFree", vec![json!(1), json!(true)]);
    for (name, value) in [
        ("maxVelocity", json!(10.0)),
        ("Agility", json!(0.9)),
        ("mass", json!(13_000_000.0)),
        ("radius", json!(2.0)),
        ("isMassive", json!(true)),
        ("speedFraction", json!(0.95)),
    ] {
        property(&mut runtime, name, value);
    }
    let ecs = runtime.world_mut();
    let mut query = ecs.query::<(&DestinyBallId, &DestinyMaxVelocity, Option<&MaxLinearSpeed>)>();
    let (_, maximum, cap) = query.iter(ecs).find(|(id, _, _)| id.0 == 1).unwrap();
    assert_eq!(maximum.0, 10.0);
    assert!(
        cap.is_none(),
        "source parameter must not install an Avian speed cap"
    );
    runtime
}

fn stop_trace(iterative: bool) {
    let mut runtime = world(iterative);
    for (name, value) in [("vx", 10.0), ("vy", 20.0), ("vz", 30.0)] {
        property(&mut runtime, name, json!(value));
    }
    park(&mut runtime, "Stop", vec![json!(1)]);
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/original_stop_velocity_trace.json")).unwrap();
    for (step, expected) in fixture["value"].as_array().unwrap().iter().enumerate() {
        park(&mut runtime, "Evolve", vec![]);
        let world = runtime.world_mut();
        let mut query = world.query::<(&DestinyBallId, &LinearVelocity)>();
        let velocity = query.iter(world).find(|(id, _)| id.0 == 1).unwrap().1.0;
        for (axis, actual) in velocity.to_array().into_iter().enumerate() {
            let expected = expected[axis].as_f64().unwrap();
            // Original assertListOfPointsAlmostEqual compares each coordinate
            // rounded to four decimal places. Do not introduce a new epsilon.
            assert_eq!(
                format!("{actual:.4}"),
                format!("{expected:.4}"),
                "iterative={iterative}, step={step}, axis={axis}: {actual:?} versus {expected:?}"
            );
        }
    }
}

#[test]
fn stop_velocity_matches_original_ten_step_trace() {
    stop_trace(false);
}

#[test]
fn iterative_stop_velocity_matches_original_ten_step_trace() {
    stop_trace(true);
}

#[test]
fn stopped_ball_preserves_original_zero_position_each_tick() {
    let mut runtime = world(false);
    for _ in 0..10 {
        park(&mut runtime, "Evolve", vec![]);
        let world = runtime.world_mut();
        let mut query = world.query::<(&DestinyBallId, &Position)>();
        let position = query.iter(world).find(|(id, _)| id.0 == 1).unwrap().1.0;
        assert_eq!(position.to_array(), [0.0; 3]);
    }
}

#[test]
fn stop_motion_matches_unchanged_cpp_position_and_velocity_bits() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/stop_oracle_samples.json"
    ))
    .unwrap();
    for iterative in [false, true] {
        for (case, row) in corpus["rows"].as_array().unwrap().iter().enumerate() {
            let mut runtime = world(iterative);
            for (name, value) in [
                ("mass", row["mass"].clone()),
                ("Agility", row["agility"].clone()),
                ("maxVelocity", row["max_velocity"].clone()),
            ] {
                property(&mut runtime, name, value);
            }
            for (name, value) in [
                ("friction", row["friction"].clone()),
                ("tickInterval", json!(row["dt"].as_f64().unwrap() * 1000.0)),
            ] {
                runtime
                    .dispatch(CompatRequest {
                        title: format!("destiny.Ballpark.{name}"),
                        operation: "set".into(),
                        target: Some("park:0".into()),
                        args: vec![value],
                        kwargs: Default::default(),
                    })
                    .unwrap();
            }
            for (command, field) in [
                ("SetBallPosition", "position"),
                ("SetBallVelocity", "velocity"),
            ] {
                let mut args = vec![json!(1)];
                args.extend(row[field].as_array().unwrap().iter().cloned());
                park(&mut runtime, command, args);
            }
            park(&mut runtime, "Evolve", vec![]);
            let world = runtime.world_mut();
            let mut query = world.query::<(&DestinyBallId, &Position, &LinearVelocity)>();
            let (_, position, velocity) = query.iter(world).find(|(id, _, _)| id.0 == 1).unwrap();
            let state = position
                .0
                .to_array()
                .into_iter()
                .chain(velocity.0.to_array());
            for (lane, (actual, expected)) in state
                .zip(row["expected_bits"].as_array().unwrap())
                .enumerate()
            {
                let expected = u64::from_str_radix(expected.as_str().unwrap(), 16).unwrap();
                assert_eq!(
                    actual.to_bits(),
                    expected,
                    "iterative={iterative}, case={case}, lane={lane}: {actual:?} versus {:?}",
                    f64::from_bits(expected)
                );
            }
        }
    }
}

#[test]
fn warp_alignment_matches_original_ten_step_positions_and_mode() {
    let mut runtime = world(false);
    for (name, value) in [("vx", 1.0), ("vy", 2.0), ("vz", 3.0)] {
        property(&mut runtime, name, json!(value));
    }
    park(
        &mut runtime,
        "WarpTo",
        vec![
            json!(1),
            json!(100_000.0),
            json!(200_000.0),
            json!(300_000.0),
        ],
    );
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/original_warp_alignment_trace.json")).unwrap();
    for (step, expected) in fixture["value"].as_array().unwrap().iter().enumerate() {
        park(&mut runtime, "Evolve", vec![]);
        let mode = runtime
            .dispatch(CompatRequest {
                title: "destiny.Ball.mode".into(),
                operation: "get".into(),
                target: Some("ball:1".into()),
                args: vec![],
                kwargs: Default::default(),
            })
            .unwrap();
        // Original DSTBALL_WARP is 3; the original fixture requires it every tick.
        assert_eq!(mode, json!(3), "step {step}");
        let world = runtime.world_mut();
        let mut query = world.query::<(&DestinyBallId, &Position)>();
        let position = query.iter(world).find(|(id, _)| id.0 == 1).unwrap().1.0;
        for (axis, actual) in position.to_array().into_iter().enumerate() {
            let expected = expected[axis].as_f64().unwrap();
            assert_eq!(
                format!("{actual:.4}"),
                format!("{expected:.4}"),
                "step={step}, axis={axis}: {actual:?} versus {expected:?}"
            );
        }
    }
}
