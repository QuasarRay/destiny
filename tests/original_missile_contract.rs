//! Source missile fixture assertions and independent C++ numeric samples,
//! executed in the production Bevy world without Python, Blue or transport.
use avian3d::prelude::{LinearVelocity, Position};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{
    CompatOptions, CompatRequest, CompatRuntime, DestinyBallId, DestinyBallMetadata,
};
use serde_json::{Value, json};

fn request(
    runtime: &mut CompatRuntime,
    title: &str,
    operation: &str,
    target: &str,
    args: Vec<Value>,
) -> Value {
    runtime
        .dispatch(CompatRequest {
            title: title.into(),
            operation: operation.into(),
            target: Some(target.into()),
            args,
            kwargs: Default::default(),
        })
        .expect(title)
}

fn park(runtime: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    request(
        runtime,
        &format!("destiny.Ballpark.{name}"),
        "call",
        "park:0",
        args,
    )
}

fn property(runtime: &mut CompatRuntime, id: i64, name: &str, value: Value) {
    request(
        runtime,
        &format!("destiny.Ball.{name}"),
        "set",
        &format!("ball:{id}"),
        vec![value],
    );
}

fn host(iterative: bool) -> CompatRuntime {
    CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        ..Default::default()
    })
    .unwrap()
}

fn empty_ball(runtime: &mut CompatRuntime, id: i64, x: f64, dynamic: bool) {
    let mut args = vec![
        json!(id),
        json!(0.0),
        json!(0.0),
        json!(0.0),
        json!(false),
        json!(false),
        json!(false),
        json!(false),
        json!(false),
        json!(x),
        json!(0.0),
        json!(0.0),
        json!(0.0),
        json!(0.0),
        json!(0.0),
        json!(0.0),
        json!(0.0),
    ];
    if dynamic {
        args.extend([json!(1.0), json!(0.9)]);
    }
    park(runtime, "AddBall", args);
}

fn space_ball(runtime: &mut CompatRuntime, id: i64, x: f64) {
    // Original helpers.create_space_ball, preserving command/property order.
    empty_ball(runtime, id, x, false);
    park(runtime, "SetBallFree", vec![json!(id), json!(true)]);
    for (name, value) in [
        ("maxVelocity", json!(10.0)),
        ("Agility", json!(0.9)),
        ("mass", json!(13_000_000.0)),
        ("radius", json!(2.0)),
        ("isMassive", json!(true)),
        ("speedFraction", json!(0.95)),
    ] {
        property(runtime, id, name, value);
    }
}

fn position_velocity(runtime: &mut CompatRuntime, id: i64) -> ([f64; 3], [f64; 3]) {
    let world = runtime.world_mut();
    let mut query = world.query::<(&DestinyBallId, &Position, &LinearVelocity)>();
    let (_, position, velocity) = query.iter(world).find(|(ball, _, _)| ball.0 == id).unwrap();
    (position.0.to_array(), velocity.0.to_array())
}

fn almost(actual: f64, expected: f64, places: usize) {
    // unittest.assertAlmostEqual rounds the difference, not each operand.
    assert_eq!(
        format!("{:.*}", places, (actual - expected).abs()),
        format!("{:.*}", places, 0.0),
        "{actual:?} versus {expected:?}"
    );
}

fn original_fixture(aimed: bool, massive: bool, moving: bool) {
    for iterative in [false, true] {
        let mut runtime = host(iterative);
        space_ball(&mut runtime, 1, 1e6);
        space_ball(&mut runtime, 2, 0.0);
        space_ball(&mut runtime, 3, 2e3);
        property(&mut runtime, 3, "radius", json!(1.9e3));
        space_ball(&mut runtime, 4, 0.0);
        for (name, value) in [
            ("maxVelocity", 300.0),
            ("Agility", 0.01),
            ("maxAngularVelocity", 2.0),
            ("RotationalAgility", 0.5),
            ("mass", 1e4),
        ] {
            property(&mut runtime, 4, name, json!(value));
        }
        if moving {
            property(&mut runtime, 2, "vz", json!(10.0));
        }
        park(
            &mut runtime,
            "LaunchMissile",
            vec![json!(4), json!(1), json!(2), json!(aimed), json!(massive)],
        );
        if moving {
            property(&mut runtime, 2, "vz", json!(0.0));
        }
        if !massive {
            let initial = position_velocity(&mut runtime, 4).1;
            for (actual, expected) in initial.into_iter().zip(if aimed {
                [1.0, 0.0, 0.0]
            } else if moving {
                [0.0, 0.0, 160.0]
            } else {
                [0.0, 0.0, -150.0]
            }) {
                almost(actual, expected, 7);
            }
        }
        for _ in 0..20 {
            park(&mut runtime, "Evolve", vec![]);
        }
        let (missile_position, velocity) = position_velocity(&mut runtime, 4);
        let places = if !aimed && !moving { 2 } else { 4 };
        almost(velocity[0], 300.0, places);
        for actual in &velocity[1..] {
            if !aimed && !moving {
                assert!(actual.abs() <= 0.5);
            } else {
                almost(*actual, 0.0, if aimed { 4 } else { 0 });
            }
        }
        let collider = position_velocity(&mut runtime, 3).0;
        for (actual, expected) in collider.into_iter().zip([2e3, 0.0, 0.0]) {
            almost(actual, expected, 7);
        }
        if aimed && !massive {
            assert!(missile_position[0] > collider[0]);
        }
    }
}

#[test]
fn original_non_aimed_stationary_missile_fixture() {
    original_fixture(false, false, false);
}
#[test]
fn original_non_aimed_moving_missile_fixture() {
    original_fixture(false, false, true);
}
#[test]
fn original_aimed_nonmassive_missile_fixture() {
    original_fixture(true, false, false);
}
#[test]
fn original_aimed_massive_missile_fixture() {
    original_fixture(true, true, false);
}

fn capture(runtime: &mut CompatRuntime) -> (Value, Value) {
    let result = request(
        runtime,
        "dbc.compat.Ballpark.CaptureSnapshot",
        "call",
        "park:0",
        vec![],
    );
    let encoded = result["snapshot"].clone();
    let decoded =
        serde_json::from_slice(&BASE64.decode(encoded.as_str().unwrap()).unwrap()).unwrap();
    (encoded, decoded)
}

#[test]
fn missile_motion_matches_unchanged_cpp_bits_with_real_clock_and_friction() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/missile_oracle_samples.json"
    ))
    .unwrap();
    for iterative in [false, true] {
        for (case, row) in corpus["rows"].as_array().unwrap().iter().enumerate() {
            let input = row["input"].as_array().unwrap();
            let mut runtime = host(iterative);
            empty_ball(&mut runtime, 1, 0.0, false);
            empty_ball(&mut runtime, 2, 0.0, false);
            for (name, value) in [
                ("mass", input[12].clone()),
                ("Agility", input[13].clone()),
                ("maxVelocity", input[14].clone()),
                ("radius", input[15].clone()),
            ] {
                property(&mut runtime, 1, name, value);
            }
            property(&mut runtime, 2, "radius", input[16].clone());
            park(&mut runtime, "SetBallFree", vec![json!(1), json!(true)]);
            for (id, position, velocity) in [
                (1, &input[0..3], &input[3..6]),
                (2, &input[6..9], &input[9..12]),
            ] {
                for (method, values) in
                    [("SetBallPosition", position), ("SetBallVelocity", velocity)]
                {
                    let mut args = vec![json!(id)];
                    args.extend_from_slice(values);
                    park(&mut runtime, method, args);
                }
            }
            for (name, value) in [
                ("friction", input[17].clone()),
                ("tickInterval", row["tick_ms"].clone()),
            ] {
                request(
                    &mut runtime,
                    &format!("destiny.Ballpark.{name}"),
                    "set",
                    "park:0",
                    vec![value],
                );
            }
            park(
                &mut runtime,
                "MissileFollow",
                vec![json!(1), json!(2), json!(3)],
            );
            let world = runtime.world_mut();
            let mut query = world.query::<(&DestinyBallId, &mut DestinyBallMetadata)>();
            let (_, mut metadata) = query.iter_mut(world).find(|(id, _)| id.0 == 1).unwrap();
            metadata.effect_stamp = -row["elapsed_ticks"].as_i64().unwrap();
            park(&mut runtime, "Evolve", vec![]);
            let (position, velocity) = position_velocity(&mut runtime, 1);
            let snapshot = capture(&mut runtime).1;
            let destination = snapshot["balls"][0]["motion"]["destination"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_f64().unwrap());
            for (lane, (actual, expected)) in destination
                .chain(position)
                .chain(velocity)
                .zip(row["expected_bits"].as_array().unwrap().iter().skip(3))
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

fn stored_bits(runtime: &mut CompatRuntime) -> u32 {
    let world = runtime.world_mut();
    let mut query = world.query::<(&DestinyBallId, &DestinyBallMetadata)>();
    query
        .iter(world)
        .find(|(id, _)| id.0 == 1)
        .unwrap()
        .1
        .angular_agility
        .to_bits()
}

#[test]
fn raw_rotational_agility_preserves_cpp_f32_state_and_snapshot_bits() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/missile_oracle_samples.json"
    ))
    .unwrap();
    {
        let mut runtime = host(false);
        empty_ball(&mut runtime, 1, 0.0, false);
        for row in corpus["scalar_rows"].as_array().unwrap() {
            let bits = row["input_bits"].as_u64().unwrap() as u32;
            let expected = row["expected_bits"].as_u64().unwrap() as u32;
            property(
                &mut runtime,
                1,
                "RotationalAgility",
                json!({"source_f32_bits_v1":bits}),
            );
            assert_eq!(stored_bits(&mut runtime), expected, "raw {bits:08x}");
            let (encoded, snapshot) = capture(&mut runtime);
            let mut restored = host(false);
            request(
                &mut restored,
                "dbc.compat.Ballpark.Deserialize",
                "call",
                "park:0",
                vec![encoded, json!(0)],
            );
            assert_eq!(stored_bits(&mut restored), expected, "restored {bits:08x}");
            assert_eq!(capture(&mut restored).1, snapshot);
        }
        property(&mut runtime, 1, "RotationalAgility", json!(0.1));
        assert_eq!(stored_bits(&mut runtime), 0.1_f32.to_bits());
        let getter = request(
            &mut runtime,
            "destiny.Ball.RotationalAgility",
            "get",
            "ball:1",
            vec![],
        );
        assert_eq!(
            getter.as_f64().unwrap().to_bits(),
            f64::from(0.1_f32).to_bits()
        );
    }
}

#[test]
fn retained_rotational_agility_mutates_the_original_detached_generation() {
    let mut runtime = host(false);
    empty_ball(&mut runtime, 1, 0.0, false);
    let generation = request(
        &mut runtime,
        "dbc.compat.Ballpark.ObjectGeneration",
        "call",
        "park:0",
        vec![json!(1)],
    );
    request(
        &mut runtime,
        "dbc.compat.Ballpark.RetainBall",
        "call",
        "park:0",
        vec![json!(1), generation.clone()],
    );
    park(&mut runtime, "RemoveBall", vec![json!(1)]);
    empty_ball(&mut runtime, 1, 0.0, false);
    let target = format!("ball:0:{}:1", generation.as_u64().unwrap());
    let value = json!({"source_f32_bits_v1":0xffc04321_u32});
    request(
        &mut runtime,
        "destiny.Ball.RotationalAgility",
        "set",
        &target,
        vec![value.clone()],
    );
    assert_eq!(
        request(
            &mut runtime,
            "destiny.Ball.RotationalAgility",
            "get",
            &target,
            vec![]
        ),
        value
    );
    assert_eq!(stored_bits(&mut runtime), 0.9_f32.to_bits());
}

#[test]
fn dynamic_constructor_normalizes_only_the_source_nonpositive_domain() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/missile_oracle_samples.json"
    ))
    .unwrap();
    for row in corpus["scalar_rows"].as_array().unwrap() {
        let mut runtime = host(false);
        // Exercise the existing dynamic constructor state boundary only.
        // Dynamic evolution and snapshot restore still reject the unsupported
        // world configuration; this does not claim those mechanics work.
        request(
            &mut runtime,
            "dbc.compat.Runtime.Configure",
            "call",
            "park:0",
            vec![
                json!(20),
                json!(false),
                json!(false),
                json!(true),
                json!(false),
            ],
        );
        let args = vec![
            json!(1),
            json!(1e4),
            json!(2.0),
            json!(300.0),
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
            json!(0.01),
            json!(1.0),
            json!(2.0),
            json!({"source_f32_bits_v1":row["input_bits"]}),
        ];
        park(&mut runtime, "AddBall", args);
        assert_eq!(
            stored_bits(&mut runtime),
            row["constructor_expected_bits"].as_u64().unwrap() as u32,
            "constructor input {}",
            row["input_bits"]
        );
    }
}

#[test]
fn stationary_launch_and_retained_rotation_match_original_cpp_math_bits() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/missile_oracle_samples.json"
    ))
    .unwrap();
    for (case, row) in corpus["rotation"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut runtime = host(false);
        for id in 1..=3 {
            empty_ball(&mut runtime, id, if id == 2 { 1e6 } else { 0.0 }, false);
        }
        let input = row["input"].as_array().unwrap();
        property(&mut runtime, 3, "maxVelocity", input[3].clone());
        let world = runtime.world_mut();
        let mut query = world.query::<(&DestinyBallId, &mut DestinyBallMetadata)>();
        let (_, mut metadata) = query.iter_mut(world).find(|(id, _)| id.0 == 3).unwrap();
        metadata.source_angles.yaw = input[0].as_f64().unwrap();
        metadata.source_angles.pitch = input[1].as_f64().unwrap();
        metadata.source_angles.roll = input[2].as_f64().unwrap();
        park(
            &mut runtime,
            "LaunchMissile",
            vec![json!(1), json!(2), json!(3), json!(false), json!(false)],
        );
        let velocity = position_velocity(&mut runtime, 1).1;
        for (lane, actual) in velocity.into_iter().enumerate() {
            let expected =
                u64::from_str_radix(row["expected_bits"][lane].as_str().unwrap(), 16).unwrap();
            assert_eq!(
                actual.to_bits(),
                expected,
                "launch case={case}, lane={lane}"
            );
        }
        let generation = request(
            &mut runtime,
            "dbc.compat.Ballpark.ObjectGeneration",
            "call",
            "park:0",
            vec![json!(3)],
        );
        request(
            &mut runtime,
            "dbc.compat.Ballpark.RetainBall",
            "call",
            "park:0",
            vec![json!(3), generation.clone()],
        );
        for target in [
            "ball:3".to_owned(),
            format!("ball:0:{}:3", generation.as_u64().unwrap()),
        ] {
            if target != "ball:3" {
                park(&mut runtime, "RemoveBall", vec![json!(3)]);
            }
            let rotated = request(
                &mut runtime,
                "destiny.Ball.GetRotatedVector",
                "call",
                &target,
                vec![json!([0.0, 0.0, -1.0])],
            );
            for (lane, actual) in rotated.as_array().unwrap().iter().enumerate() {
                let expected =
                    u64::from_str_radix(row["expected_bits"][lane + 3].as_str().unwrap(), 16)
                        .unwrap();
                assert_eq!(
                    actual.as_f64().unwrap().to_bits(),
                    expected,
                    "rotation case={case}, lane={lane}, target={target}"
                );
            }
        }
    }
}

#[test]
fn restored_missile_preserves_tracking_and_twenty_future_ticks() {
    for iterative in [false, true] {
        let mut first = host(iterative);
        for (id, x) in [(1, 1e6), (2, 0.0), (3, 0.0)] {
            space_ball(&mut first, id, x);
        }
        for (name, value) in [
            ("maxVelocity", 300.0),
            ("Agility", 0.01),
            ("RotationalAgility", 0.5),
            ("mass", 1e4),
        ] {
            property(&mut first, 3, name, json!(value));
        }
        park(
            &mut first,
            "LaunchMissile",
            vec![json!(3), json!(1), json!(2), json!(true), json!(false)],
        );
        for _ in 0..5 {
            park(&mut first, "Evolve", vec![]);
        }
        let (encoded, before) = capture(&mut first);
        let mut second = host(iterative);
        request(
            &mut second,
            "dbc.compat.Ballpark.Deserialize",
            "call",
            "park:0",
            vec![encoded, json!(0)],
        );
        assert_eq!(capture(&mut second).1, before);
        for step in 0..20 {
            park(&mut first, "Evolve", vec![]);
            park(&mut second, "Evolve", vec![]);
            assert_eq!(
                capture(&mut first).1,
                capture(&mut second).1,
                "iterative={iterative}, replay step={step}"
            );
        }
    }
}
