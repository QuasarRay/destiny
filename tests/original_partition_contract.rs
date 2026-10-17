//! Actual headless world compared with preserved Original assertions and an
//! independently compiled, unchanged C++ partition/box/allocator source slice.
use avian3d::prelude::{LinearVelocity, Position};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{CompatOptions, CompatRequest, CompatRuntime, DestinyBallId};
use serde_json::{Value, json};

fn call(runtime: &mut CompatRuntime, title: &str, args: Vec<Value>) -> Value {
    runtime
        .dispatch(CompatRequest {
            title: title.into(),
            operation: "call".into(),
            target: Some("park:0".into()),
            args,
            kwargs: Default::default(),
        })
        .expect(title)
}

fn park(runtime: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    call(runtime, &format!("destiny.Ballpark.{name}"), args)
}

fn host(master: bool) -> CompatRuntime {
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .unwrap();
    runtime
        .dispatch(CompatRequest {
            title: "destiny.Ballpark.isMaster".into(),
            operation: "set".into(),
            target: Some("park:0".into()),
            args: vec![json!(master)],
            kwargs: Default::default(),
        })
        .unwrap();
    runtime
}

fn add(runtime: &mut CompatRuntime, id: i64, position: [f64; 3]) {
    // Original BallparkTestCase.add_balls: fixed zero-radius ball, followed by
    // each source fixture's explicit position/property mutations.
    park(
        runtime,
        "AddBall",
        vec![
            json!(id),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(false),
            json!(false),
            json!(false),
            json!(false),
            json!(false),
            json!(position[0]),
            json!(position[1]),
            json!(position[2]),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
        ],
    );
}

fn capture(runtime: &mut CompatRuntime) -> Value {
    let value = call(runtime, "dbc.compat.Ballpark.CaptureSnapshot", vec![]);
    serde_json::from_slice(&BASE64.decode(value["snapshot"].as_str().unwrap()).unwrap()).unwrap()
}

fn normalized_partition(runtime: &mut CompatRuntime) -> Value {
    let snapshot = capture(runtime);
    let partition = &snapshot["park"]["source_partition_v1"]["partition"];
    let address = |value: &Value| {
        json!([
            value["level"],
            value["key"][0],
            value["key"][1],
            value["key"][2]
        ])
    };
    let mut boxes: Vec<Value> = partition["boxes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let mut row = address(&entry["address"]).as_array().unwrap().clone();
            row.push(entry["bubble"].clone());
            json!(row)
        })
        .collect();
    boxes.sort_by_key(|value| {
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_i64().unwrap())
            .collect::<Vec<_>>()
    });
    let mut balls:Vec<Value>=partition["placements"].as_array().unwrap().iter().map(|entry| {
        let id=entry["id"].as_i64().unwrap();
        let bubble=runtime.dispatch(CompatRequest {title:"destiny.Ball.newBubbleId".into(),operation:"get".into(),target:Some(format!("ball:{id}")),args:vec![],kwargs:Default::default()}).unwrap();
        json!({"id":id,"bubble":bubble,"primary":address(&entry["primary"]),"owned":entry["owned"].as_array().unwrap().iter().map(&address).collect::<Vec<_>>()})
    }).collect();
    balls.sort_by_key(|value| value["id"].as_i64().unwrap());
    json!({"next_bubble":partition["next_bubble"],"boxes":boxes,"balls":balls})
}

fn property(runtime: &mut CompatRuntime, target: &str, name: &str, value: Value) {
    let class = if target.starts_with("ball:") {
        "Ball"
    } else {
        "Ballpark"
    };
    runtime
        .dispatch(CompatRequest {
            title: format!("destiny.{class}.{name}"),
            operation: "set".into(),
            target: Some(target.into()),
            args: vec![value],
            kwargs: Default::default(),
        })
        .unwrap();
}

fn motion_bits(runtime: &mut CompatRuntime) -> Value {
    let world = runtime.world_mut();
    let mut query = world.query::<(&DestinyBallId, &Position, &LinearVelocity)>();
    let mut rows=query.iter(world).map(|(id,position,velocity)|json!({"id":id.0,"position_bits":position.0.to_array().map(|v|format!("{:016x}",v.to_bits())),"velocity_bits":velocity.0.to_array().map(|v|format!("{:016x}",v.to_bits()))})).collect::<Vec<_>>();
    rows.sort_by_key(|row| row["id"].as_i64().unwrap());
    json!(rows)
}

fn assert_source_state(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => {
            assert_eq!(
                actual.keys().collect::<Vec<_>>(),
                expected.keys().collect::<Vec<_>>(),
                "{path}: object fields"
            );
            for (key, value) in expected {
                assert_source_state(&actual[key], value, &format!("{path}.{key}"));
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len(), "{path}: array length");
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                assert_source_state(actual, expected, &format!("{path}[{index}]"));
            }
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}

#[test]
fn client_motion_and_partition_match_original_nonunit_dt_phases() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/partition_dynamic_oracle_samples.json"
    ))
    .unwrap();
    for (case, scenario) in corpus["scenarios"].as_array().unwrap().iter().enumerate() {
        let mut runtime = host(false);
        for (index, row) in scenario["rows"].as_array().unwrap().iter().enumerate() {
            let action = &row["action"];
            let id = action[1].as_i64().unwrap();
            let values = action[2].as_array().unwrap();
            match action[0].as_u64().unwrap() {
                0 => add(
                    &mut runtime,
                    id,
                    std::array::from_fn(|axis| values[axis].as_f64().unwrap()),
                ),
                1 | 3 => {
                    let mut args = vec![json!(id)];
                    args.extend(values.iter().cloned());
                    park(
                        &mut runtime,
                        if action[0] == 1 {
                            "SetBallPosition"
                        } else {
                            "SetBallVelocity"
                        },
                        args,
                    );
                }
                2 => {
                    park(
                        &mut runtime,
                        "SetBallRadius",
                        vec![json!(id), values[0].clone()],
                    );
                }
                6 => {
                    property(&mut runtime, "park:0", "friction", values[0].clone());
                    property(
                        &mut runtime,
                        "park:0",
                        "tickInterval",
                        json!((values[1].as_f64().unwrap() * 1000.) as u64),
                    );
                }
                7 => {
                    for (name, value) in
                        ["mass", "Agility", "speedFraction"].into_iter().zip(values)
                    {
                        property(&mut runtime, &format!("ball:{id}"), name, value.clone());
                    }
                    park(&mut runtime, "SetBallFree", vec![json!(id), json!(true)]);
                    park(&mut runtime, "Stop", vec![json!(id)]);
                }
                8 => {
                    park(
                        &mut runtime,
                        "SetMaxSpeed",
                        vec![json!(id), values[0].clone()],
                    );
                }
                9 => {
                    park(&mut runtime, "Evolve", vec![]);
                }
                _ => panic!("unreviewed dynamic oracle action"),
            }
            let mut actual = normalized_partition(&mut runtime);
            actual["motion"] = motion_bits(&mut runtime);
            assert_source_state(
                &actual,
                &row["expected"],
                &format!("case={case}, transition={index}, action={action}"),
            );
        }
    }
}

#[test]
fn collision_motion_ignores_client_numeric_bubble_labels() {
    // Original GetNearbyBalls applies bubble equality only for isMaster.
    // A distant fixed, nonmassive ball makes the contacting group label 1;
    // it cannot enter this pair's source spatial/collision candidate set.
    for iterative in [false, true] {
        let mut worlds = Vec::new();
        for label_one in [false, true] {
            let mut runtime = CompatRuntime::new(CompatOptions {
                enable_network_components: false,
                use_iterative_collision: iterative,
                ..Default::default()
            })
            .unwrap();
            if label_one {
                add(&mut runtime, 0, [-1e9, 1., 1.]);
            }
            for (id, x, vx) in [(1, 0., 20.), (2, 8., -20.)] {
                add(&mut runtime, id, [x, 0., 0.]);
                for (name, value) in [
                    ("mass", 13e6),
                    ("Agility", 0.9),
                    ("radius", 2.),
                    ("maxVelocity", 10.),
                ] {
                    property(&mut runtime, &format!("ball:{id}"), name, json!(value));
                }
                park(&mut runtime, "SetBallFree", vec![json!(id), json!(true)]);
                property(
                    &mut runtime,
                    &format!("ball:{id}"),
                    "isMassive",
                    json!(true),
                );
                park(&mut runtime, "Stop", vec![json!(id)]);
                park(
                    &mut runtime,
                    "SetBallVelocity",
                    vec![json!(id), json!(vx), json!(0.), json!(0.)],
                );
            }
            park(&mut runtime, "InitializeBubbles", vec![]);
            worlds.push(runtime);
        }
        for tick in 0..5 {
            for world in &mut worlds {
                park(world, "Evolve", vec![]);
            }
            let baseline = motion_bits(&mut worlds[0]);
            let shifted = motion_bits(&mut worlds[1]);
            let shifted: Vec<_> = shifted
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["id"] != 0)
                .cloned()
                .collect();
            assert_eq!(
                baseline,
                json!(shifted),
                "iterative={iterative}, tick={tick}"
            );
        }
    }
}

fn original_box_fixture(z: f64, expected_z: f64) {
    let mut runtime = host(false);
    add(&mut runtime, 1, [0.; 3]);
    for (name, value) in [("x", 1.), ("y", 1.), ("z", z)] {
        runtime
            .dispatch(CompatRequest {
                title: format!("destiny.Ball.{name}"),
                operation: "set".into(),
                target: Some("ball:1".into()),
                args: vec![json!(value)],
                kwargs: Default::default(),
            })
            .unwrap();
    }
    let value = park(&mut runtime, "GetBallBox", vec![json!(1)]);
    // Every assertion in each unchanged TestBallBox fixture is retained.
    assert_eq!(value[0], json!(480.));
    for (actual, expected) in value[1]
        .as_array()
        .unwrap()
        .iter()
        .zip([240., 240., expected_z])
    {
        assert!((actual.as_f64().unwrap() - expected).abs() < 0.00000005);
    }
}

#[test]
fn original_box_zero_zero_zero() {
    original_box_fixture(1., 240.);
}
#[test]
fn original_box_zero_zero_one() {
    original_box_fixture(481., 720.);
}

fn bubble_property(runtime: &mut CompatRuntime, id: i64, name: &str) -> Value {
    runtime
        .dispatch(CompatRequest {
            title: format!("destiny.Ball.{name}"),
            operation: "get".into(),
            target: Some(format!("ball:{id}")),
            args: vec![],
            kwargs: Default::default(),
        })
        .unwrap()
}

#[test]
fn original_can_initialize_bubbles() {
    let mut runtime = host(false);
    add(&mut runtime, 1, [0.; 3]);
    assert_eq!(bubble_property(&mut runtime, 1, "newBubbleId"), json!(-1));
    park(&mut runtime, "InitializeBubbles", vec![]);
    assert_ne!(bubble_property(&mut runtime, 1, "newBubbleId"), json!(-1));
    assert_eq!(bubble_property(&mut runtime, 1, "oldBubbleId"), json!(-1));
}

#[test]
fn original_can_initialize_bubble_for_specific_ball() {
    let mut runtime = host(false);
    add(&mut runtime, 1, [0.; 3]);
    add(&mut runtime, 2, [0.; 3]);
    assert_eq!(bubble_property(&mut runtime, 1, "newBubbleId"), json!(-1));
    park(&mut runtime, "InitializeBubbles", vec![json!(1)]);
    assert_ne!(bubble_property(&mut runtime, 1, "newBubbleId"), json!(-1));
    assert_eq!(bubble_property(&mut runtime, 1, "oldBubbleId"), json!(-1));
    assert_eq!(bubble_property(&mut runtime, 2, "newBubbleId"), json!(-1));
}

#[test]
fn original_client_proximity_assertions_without_bubble_initialization() {
    // All eight TestProximitySensors fixtures and all nine original event-list
    // assertions, preserving helper defaults, ordering, period and range.
    for case in [
        "period", "called", "fixed", "only", "removed", "outside", "edge", "leave",
    ] {
        let mut runtime = host(false);
        let point = 1e10;
        add(&mut runtime, 3, [point; 3]);
        park(
            &mut runtime,
            "AddProximitySensor",
            vec![
                json!(3),
                json!(10.),
                json!(if case == "period" { 2. } else { 0. }),
                json!(0), // Original False through the existing integer shuffle ABI.
                json!(case == "only"),
            ],
        );
        if case == "removed" {
            park(&mut runtime, "RemoveProximitySensor", vec![json!(3)]);
        }
        let dx = if case == "outside" {
            11.
        } else if case == "edge" {
            10.
        } else {
            0.
        };
        add(&mut runtime, 4, [point + dx, point, point]);
        park(
            &mut runtime,
            "SetBallFree",
            vec![json!(4), json!(case != "fixed")],
        );
        let read = |runtime: &mut CompatRuntime| {
            let events = call(runtime, "dbc.compat.Proximity.DrainEvents", vec![]);
            events
                .as_array()
                .unwrap()
                .iter()
                .map(|event| {
                    assert_eq!(event["owner_id"], json!(3));
                    json!([
                        event["other_id"],
                        i64::from(event["entering"].as_bool().unwrap())
                    ])
                })
                .collect::<Vec<_>>()
        };
        park(&mut runtime, "Evolve", vec![]);
        let mut observed = read(&mut runtime);
        if case == "period" {
            assert_eq!(observed, Vec::<Value>::new(), "{case}: before period");
            park(&mut runtime, "Evolve", vec![]);
            observed.extend(read(&mut runtime));
        }
        if case == "leave" {
            property(&mut runtime, "ball:4", "x", json!(point + 11.));
            park(&mut runtime, "Evolve", vec![]);
            observed.extend(read(&mut runtime));
        }
        let expected = match case {
            "period" | "called" | "edge" => vec![json!([4, 1])],
            "leave" => vec![json!([4, 1]), json!([4, 0])],
            _ => vec![],
        };
        assert_eq!(observed, expected, "{case}");
        assert_eq!(bubble_property(&mut runtime, 3, "newBubbleId"), json!(-1));
        assert_eq!(bubble_property(&mut runtime, 4, "newBubbleId"), json!(-1));
    }
}

#[test]
fn partition_ownership_matches_unchanged_cpp_after_every_mutation() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../verification/migration/partition_oracle_samples.json"
    ))
    .unwrap();
    for scenario in corpus["scenarios"].as_array().unwrap() {
        let mut runtime = host(scenario["master"].as_bool().unwrap());
        for (index, row) in scenario["rows"].as_array().unwrap().iter().enumerate() {
            let action = &row["action"];
            let id = action[1].as_i64().unwrap();
            let values = action[2].as_array().unwrap();
            match action[0].as_u64().unwrap() {
                0 => add(
                    &mut runtime,
                    id,
                    std::array::from_fn(|axis| values[axis].as_f64().unwrap()),
                ),
                1 | 3 => {
                    let name = if action[0] == 1 {
                        "SetBallPosition"
                    } else {
                        "SetBallVelocity"
                    };
                    let mut args = vec![json!(id)];
                    args.extend(values.iter().cloned());
                    park(&mut runtime, name, args);
                }
                2 => {
                    park(
                        &mut runtime,
                        "SetBallRadius",
                        vec![json!(id), values[0].clone()],
                    );
                }
                4 => {
                    park(&mut runtime, "RemoveBall", vec![json!(id)]);
                }
                5 => {
                    park(&mut runtime, "ClearAll", vec![]);
                }
                _ => panic!("unreviewed oracle action"),
            }
            assert_eq!(
                normalized_partition(&mut runtime),
                row["expected"],
                "master={} transition={index} action={action}",
                scenario["master"]
            );
        }
    }
}

#[test]
fn source_partition_is_unbounded_and_retains_stale_ownership_in_snapshots() {
    let mut runtime = host(false);
    for id in 1..=100 {
        add(&mut runtime, id, [id as f64 * 1e7, 1., 1.]);
    }
    let before = capture(&mut runtime);
    assert!(
        before["park"]["source_partition_v1"]["partition"]["boxes"]
            .as_array()
            .unwrap()
            .len()
            > 64
    );
    park(
        &mut runtime,
        "SetBallVelocity",
        vec![json!(1), json!(1e9), json!(0.), json!(0.)],
    );
    let snapshot = capture(&mut runtime);
    assert_eq!(
        snapshot["park"]["source_partition_v1"],
        before["park"]["source_partition_v1"]
    );
    let mut restored = host(false);
    let encoded = BASE64.encode(serde_json::to_vec(&snapshot).unwrap());
    call(
        &mut restored,
        "dbc.compat.Ballpark.Deserialize",
        vec![json!(encoded)],
    );
    assert_eq!(
        normalized_partition(&mut runtime),
        normalized_partition(&mut restored)
    );
    park(&mut restored, "ClearAll", vec![]);
    assert_eq!(
        park(&mut restored, "GetBallBox", vec![json!(1)]),
        Value::Null
    );
    assert!(
        capture(&mut restored)["park"]["source_partition_v1"]["partition"]["boxes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn malformed_partition_snapshots_reject_without_mutation() {
    let mut runtime = host(false);
    add(&mut runtime, 1, [1.; 3]);
    let original = capture(&mut runtime);
    for case in 0..4 {
        let mut bad = original.clone();
        let partition = &mut bad["park"]["source_partition_v1"]["partition"];
        match case {
            0 => partition["boxes"][0]["address"]["level"] = json!(8),
            1 => {
                let copy = partition["boxes"][0].clone();
                partition["boxes"].as_array_mut().unwrap().push(copy);
            }
            2 => partition["placements"][0]["id"] = json!(999),
            3 => {
                partition["boxes"].as_array_mut().unwrap().remove(0);
            }
            _ => unreachable!(),
        }
        let result = runtime.dispatch(CompatRequest {
            title: "dbc.compat.Ballpark.Deserialize".into(),
            operation: "call".into(),
            target: Some("park:0".into()),
            args: vec![json!(BASE64.encode(serde_json::to_vec(&bad).unwrap()))],
            kwargs: Default::default(),
        });
        assert!(result.is_err(), "malformed partition case {case}");
        assert_eq!(capture(&mut runtime), original);
    }
}

#[test]
fn retained_rust_extreme_coordinates_keep_explicit_partition_limits() {
    let mut runtime = host(false);
    add(&mut runtime, 1, [1.; 3]);
    park(
        &mut runtime,
        "SetBallPosition",
        vec![json!(1), json!(5e99), json!(0.), json!(0.)],
    );
    let snapshot = capture(&mut runtime);
    assert_eq!(
        snapshot["park"]["source_partition_v1"]["partition"]["undefined_coordinates"],
        json!([1])
    );
    let mut restored = host(false);
    call(
        &mut restored,
        "dbc.compat.Ballpark.Deserialize",
        vec![json!(BASE64.encode(serde_json::to_vec(&snapshot).unwrap()))],
    );
    let result = restored.dispatch(CompatRequest {
        title: "destiny.Ballpark.GetBallBox".into(),
        operation: "call".into(),
        target: Some("park:0".into()),
        args: vec![json!(1)],
        kwargs: Default::default(),
    });
    assert!(result.is_err());
    park(
        &mut restored,
        "SetBallPosition",
        vec![json!(1), json!(1.), json!(1.), json!(1.)],
    );
    assert_eq!(
        park(&mut restored, "GetBallBox", vec![json!(1)]),
        json!([480., [240., 240., 240.]])
    );
}
