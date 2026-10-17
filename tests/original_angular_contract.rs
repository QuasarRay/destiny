//! Direct Rust hosts of the actual Bevy runtime; no Python/Blue/transport.
//! Angular expectations come from the unchanged C++ oracle corpus. Replay and
//! bit preservation are state-contract properties, not universal parity proofs.
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{
    CompatOptions, CompatRequest, CompatRuntime, DestinyAngularState, DestinyBallId,
    DestinyBallMetadata,
};
use serde_json::{Value, json};

fn request(runtime: &mut CompatRuntime, title: &str, operation: &str, args: Vec<Value>) -> Value {
    runtime
        .dispatch(CompatRequest {
            title: title.into(),
            operation: operation.into(),
            target: Some("park:0".into()),
            args,
            kwargs: Default::default(),
        })
        .expect(title)
}

fn host(free: bool) -> CompatRuntime {
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .unwrap();
    request(
        &mut runtime,
        "destiny.Ballpark.AddBall",
        "call",
        vec![
            json!(1),
            json!(13_000_000.0),
            json!(2.0),
            json!(10.0),
            json!(free),
            json!(false),
            json!(false),
            json!(true),
            json!(false),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.0),
            json!(0.9),
            json!(0.95),
        ],
    );
    runtime
}

fn capture(runtime: &mut CompatRuntime) -> (Value, Value) {
    let result = request(
        runtime,
        "dbc.compat.Ballpark.CaptureSnapshot",
        "call",
        vec![],
    );
    let encoded = result["snapshot"].clone();
    let decoded =
        serde_json::from_slice(&BASE64.decode(encoded.as_str().unwrap()).unwrap()).unwrap();
    (encoded, decoded)
}

#[test]
fn velocity_command_reaches_source_owned_angular_state() {
    let mut runtime = host(false);
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/angular_oracle_samples.json"
    ))
    .unwrap();
    let row = &corpus["rows"][0];
    let input = row["input"].as_array().unwrap();
    request(
        &mut runtime,
        "destiny.Ballpark.SetBallVelocity",
        "call",
        vec![
            json!(1),
            input[1].clone(),
            input[2].clone(),
            input[3].clone(),
        ],
    );
    let (_, state) = capture(&mut runtime);
    for (name, expected) in row["expected_bits"].as_object().unwrap() {
        let expected = u64::from_str_radix(expected.as_str().unwrap(), 16).unwrap();
        assert_eq!(
            state["balls"][0]["source_angles"][name].as_u64().unwrap(),
            expected,
            "{name}"
        );
    }
    for name in ["yaw", "pitch", "roll"] {
        let result = runtime
            .dispatch(CompatRequest {
                title: format!("destiny.Ball.{name}"),
                operation: "get".into(),
                target: Some("ball:1".into()),
                args: vec![],
                kwargs: Default::default(),
            })
            .unwrap()
            .as_f64()
            .unwrap();
        let expected =
            u64::from_str_radix(row["expected_bits"][name].as_str().unwrap(), 16).unwrap();
        assert_eq!(result.to_bits(), expected, "getter {name}");
    }
}

#[test]
fn restored_angles_and_roll_inputs_produce_the_same_future() {
    let mut first = host(true);
    for (name, value) in [
        ("rollAcceleration", 2.5),
        ("rollDecay", -0.75),
        ("rollSpeedAcceleration", -4.25),
        ("rollSpeedDecay", -0.2),
    ] {
        request(
            &mut first,
            &format!("destiny.Ballpark.{name}"),
            "set",
            vec![json!(value)],
        );
    }
    request(
        &mut first,
        "destiny.Ballpark.SetBallVelocity",
        "call",
        vec![json!(1), json!(1.0), json!(2.0), json!(3.0)],
    );
    request(
        &mut first,
        "destiny.Ballpark.GotoPoint",
        "call",
        vec![json!(1), json!(-100.0), json!(50.0), json!(75.0)],
    );
    for _ in 0..4 {
        request(&mut first, "destiny.Ballpark.Evolve", "call", vec![]);
    }
    let (encoded, before) = capture(&mut first);
    assert_ne!(before["balls"][0]["source_angles"]["roll"], json!(0));
    let mut second = host(false);
    request(
        &mut second,
        "dbc.compat.Ballpark.Deserialize",
        "call",
        vec![encoded, json!(0)],
    );
    assert_eq!(capture(&mut second).1, before);
    for _ in 0..12 {
        request(&mut first, "destiny.Ballpark.Evolve", "call", vec![]);
        request(&mut second, "destiny.Ballpark.Evolve", "call", vec![]);
        assert_eq!(capture(&mut first).1, capture(&mut second).1);
    }
}

#[test]
fn world_snapshot_preserves_each_angular_bit_pattern() {
    let mut first = host(false);
    let mut words = json!({});
    for (name, value) in [
        ("yaw", 0x8000000000000000_u64),
        ("pitch", 0x7ff0000000000000),
        ("roll", 0xfff0000000000000),
        ("old_yaw", 0x7ff8000000001234),
        ("old_pitch", 0x7ff0000000004321),
        ("old_roll", 0xfff8000000008765),
        ("roll_speed", 1),
        ("old_roll_speed", 0x10000000000000),
        ("yaw_delta", 0x7fefffffffffffff),
    ] {
        words[name] = json!(value);
    }
    let angles: DestinyAngularState = serde_json::from_value(words.clone()).unwrap();
    let world = first.world_mut();
    let mut query = world.query::<(&DestinyBallId, &mut DestinyBallMetadata)>();
    for (id, mut metadata) in query.iter_mut(world) {
        if id.0 == 1 {
            metadata.source_angles = angles;
        }
    }
    let (encoded, state) = capture(&mut first);
    assert_eq!(state["balls"][0]["source_angles"], words);
    let mut second = host(false);
    request(
        &mut second,
        "dbc.compat.Ballpark.Deserialize",
        "call",
        vec![encoded, json!(0)],
    );
    assert_eq!(capture(&mut second).1["balls"][0]["source_angles"], words);
}

fn formation_trace(moving: bool, fixture_text: &str) {
    // Original TestFormation.setUp and helpers.create_space_ball ordering are
    // significant: AddBall starts non-free, then each source property is set.
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .unwrap();
    let formations: Value =
        serde_json::from_str(include_str!("fixtures/original_formations.json")).unwrap();
    request(
        &mut runtime,
        "destiny.Ballpark.LoadFormations",
        "call",
        vec![formations["value"].clone()],
    );
    for (id, x) in [(1, 0.0), (2, -5.0)] {
        request(
            &mut runtime,
            "destiny.Ballpark.AddBall",
            "call",
            vec![
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
            ],
        );
        request(
            &mut runtime,
            "destiny.Ballpark.SetBallFree",
            "call",
            vec![json!(id), json!(true)],
        );
        for (name, value) in [
            ("maxVelocity", json!(10.0)),
            ("Agility", json!(0.9)),
            ("mass", json!(13_000_000.0)),
            ("radius", json!(2.0)),
            ("isMassive", json!(true)),
            ("speedFraction", json!(0.95)),
        ] {
            runtime
                .dispatch(CompatRequest {
                    title: format!("destiny.Ball.{name}"),
                    operation: "set".into(),
                    target: Some(format!("ball:{id}")),
                    args: vec![value],
                    kwargs: Default::default(),
                })
                .unwrap();
        }
    }
    request(
        &mut runtime,
        "destiny.Ballpark.SetBallFormation",
        "call",
        vec![json!(2), json!(1)],
    );
    let slot = runtime
        .dispatch(CompatRequest {
            title: "destiny.Ball.ReserveFormationSlot".into(),
            operation: "call".into(),
            target: Some("ball:2".into()),
            args: vec![],
            kwargs: Default::default(),
        })
        .unwrap();
    request(
        &mut runtime,
        "destiny.Ballpark.FormationFollow",
        "call",
        vec![json!(1), json!(2), slot],
    );
    if moving {
        request(
            &mut runtime,
            "destiny.Ballpark.GotoPoint",
            "call",
            vec![json!(2), json!(0.0), json!(100.0), json!(0.0)],
        );
    }
    let fixture: Value = serde_json::from_str(fixture_text).unwrap();
    for (step, expected) in fixture["value"].as_array().unwrap().iter().enumerate() {
        request(&mut runtime, "destiny.Ballpark.Evolve", "call", vec![]);
        let world = runtime.world_mut();
        let mut query = world.query::<(&DestinyBallId, &avian3d::prelude::Position)>();
        let position = query.iter(world).find(|(id, _)| id.0 == 1).unwrap().1.0;
        for (axis, actual) in position.to_array().into_iter().enumerate() {
            let expected = expected[axis].as_f64().unwrap();
            // Original helper compares each coordinate rounded to four decimal
            // places. Preserve that rule, rather than introducing a new epsilon.
            assert_eq!(
                format!("{actual:.4}"),
                format!("{expected:.4}"),
                "step {step}, axis {axis}: {actual:?} versus {expected:?}"
            );
        }
    }
}

#[test]
fn stopped_formation_matches_original_ten_step_trace() {
    formation_trace(
        false,
        include_str!("fixtures/original_formation_stopped_trace.json"),
    );
}

#[test]
fn moving_formation_matches_original_ten_step_trace() {
    formation_trace(
        true,
        include_str!("fixtures/original_formation_moving_trace.json"),
    );
}
