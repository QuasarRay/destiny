use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{CompatOptions, CompatRequest, CompatRuntime};
use serde_json::{Value, json};

fn request(title: &str, operation: &str, target: Option<&str>, args: Vec<Value>) -> CompatRequest {
    CompatRequest {
        title: title.to_owned(),
        operation: operation.to_owned(),
        target: target.map(str::to_owned),
        args,
        kwargs: serde_json::Map::new(),
    }
}

fn runtime_with_ball() -> CompatRuntime {
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .expect("runtime");

    runtime
        .dispatch(request(
            "destiny.Ballpark.__init__",
            "construct",
            None,
            vec![json!(false)],
        ))
        .expect("construct ballpark");

    runtime
        .dispatch(request(
            "destiny.Ballpark.AddBall",
            "call",
            Some("park:0"),
            vec![
                json!(1),
                json!(10.0),
                json!(2.0),
                json!(100.0),
                json!(false),
                json!(false),
                json!(true),
                json!(true),
                json!(false),
                json!(0.0),
                json!(0.0),
                json!(0.0),
                json!(0.0),
                json!(0.0),
                json!(0.0),
                json!(0.5),
                json!(1.0),
            ],
        ))
        .expect("add ball");

    runtime
}

fn snapshot(runtime: &mut CompatRuntime) -> Value {
    let captured = runtime
        .dispatch(request(
            "dbc.compat.Ballpark.CaptureSnapshot",
            "call",
            Some("park:0"),
            vec![json!(-1)],
        ))
        .expect("capture snapshot");
    let encoded = captured["snapshot"].as_str().expect("snapshot base64");
    let bytes = BASE64.decode(encoded).expect("decode snapshot");
    serde_json::from_slice(&bytes).expect("snapshot json")
}

fn only_ball(snapshot: &Value) -> &Value {
    let balls = snapshot["balls"].as_array().expect("balls");
    assert_eq!(balls.len(), 1);
    &balls[0]
}

// Original tests:
// - python/destiny/test/test_ball.py::test_can_add_miniball
// - python/destiny/test/test_ball.py::test_can_add_minicapsule
// - python/destiny/test/test_ball.py::test_get_rotated_vector_returns_original_vector_if_there_is_no_rotation
#[test]
fn original_ball_child_shapes_and_identity_rotation_match_public_contract() {
    let mut runtime = runtime_with_ball();

    let rotated = runtime
        .dispatch(request(
            "destiny.Ball.GetRotatedVector",
            "call",
            Some("ball:1"),
            vec![json!([1.0, 2.0, 3.0])],
        ))
        .expect("rotate vector");
    assert_eq!(rotated, json!([1.0, 2.0, 3.0]));

    runtime
        .dispatch(request(
            "destiny.Ball.AddMiniBall",
            "call",
            Some("ball:1"),
            vec![json!(1.0), json!(2.0), json!(3.0), json!(1.0)],
        ))
        .expect("add mini ball");

    runtime
        .dispatch(request(
            "destiny.Ball.AddMiniCapsule",
            "call",
            Some("ball:1"),
            vec![
                json!(1.0),
                json!(0.0),
                json!(0.0),
                json!(2.0),
                json!(0.0),
                json!(0.0),
                json!(1.0),
            ],
        ))
        .expect("add mini capsule");

    let state = snapshot(&mut runtime);
    let minis = only_ball(&state)["minis"].as_array().expect("minis");
    assert_eq!(minis.len(), 2);
    assert_eq!(minis[0]["kind"], json!("sphere"));
    assert_eq!(minis[1]["kind"], json!("capsule"));
}

// Original test:
// python/destiny/test/test_ball.py::test_can_not_add_minicapsule_with_negative_radius
#[test]
fn original_ball_nonpositive_minicapsule_radius_is_rejected_transactionally() {
    let mut runtime = runtime_with_ball();

    for radius in [-1.0, 0.0] {
        assert!(
            runtime
                .dispatch(request(
                    "destiny.Ball.AddMiniCapsule",
                    "call",
                    Some("ball:1"),
                    vec![
                        json!(1.0),
                        json!(0.0),
                        json!(0.0),
                        json!(2.0),
                        json!(0.0),
                        json!(0.0),
                        json!(radius),
                    ],
                ))
                .is_err()
        );
    }

    let state = snapshot(&mut runtime);
    assert!(
        only_ball(&state)["minis"]
            .as_array()
            .expect("minis")
            .is_empty()
    );
}

// Original test:
// python/destiny/test/test_ball.py::test_add_proximity_sensor
#[test]
fn original_ball_proximity_sensor_arguments_are_stored() {
    let mut runtime = runtime_with_ball();

    runtime
        .dispatch(request(
            "destiny.Ball.AddProximitySensor",
            "call",
            Some("ball:1"),
            vec![json!(5.0), json!(10.0), json!(1), json!(false)],
        ))
        .expect("add proximity sensor");

    let state = snapshot(&mut runtime);
    let sensors = only_ball(&state)["sensors"].as_array().expect("sensors");
    assert_eq!(sensors.len(), 1);
    assert_eq!(sensors[0]["range"], json!(5.0));
    assert_eq!(sensors[0]["period"], json!(10.0));
    assert_eq!(sensors[0]["only_interactives"], json!(false));
}
