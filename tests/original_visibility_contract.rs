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

fn runtime() -> CompatRuntime {
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
        .expect("construct");
    runtime
}

fn add_ball(
    runtime: &mut CompatRuntime,
    id: i64,
    x: f64,
    y: f64,
    z: f64,
    radius: f64,
    massive: bool,
) {
    runtime
        .dispatch(request(
            "destiny.Ballpark.AddBall",
            "call",
            Some("park:0"),
            vec![
                json!(id),
                json!(10.0),
                json!(radius),
                json!(100.0),
                json!(false),
                json!(false),
                json!(massive),
                json!(true),
                json!(false),
                json!(x),
                json!(y),
                json!(z),
                json!(0.0),
                json!(0.0),
                json!(0.0),
                json!(0.5),
                json!(1.0),
            ],
        ))
        .expect("add ball");
}

fn ball_state(runtime: &mut CompatRuntime, id: i64) -> Value {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

    let captured = runtime
        .dispatch(request(
            "dbc.compat.Ballpark.CaptureSnapshot",
            "call",
            Some("park:0"),
            vec![json!(-1)],
        ))
        .expect("snapshot");
    let encoded = captured["snapshot"].as_str().expect("base64 snapshot");
    let bytes = BASE64.decode(encoded).expect("decode");
    let payload: Value = serde_json::from_slice(&bytes).expect("snapshot json");
    payload["balls"]
        .as_array()
        .expect("balls")
        .iter()
        .find(|ball| ball["id"] == json!(id))
        .expect("ball")
        .clone()
}

// Original tests:
// - test_returns_zero_when_there_is_no_occlusion
// - test_returns_the_id_of_the_occluding_ball_when_there_is_an_occlusion
// - test_ball_that_is_not_massive_is_not_an_occlusion
// - test_ball_that_is_cloaked_is_not_an_occlusion
#[test]
fn original_visibility_occlusion_filters_match_public_runtime() {
    let mut runtime = runtime();
    add_ball(&mut runtime, 1, 0.0, 0.0, 0.0, 2.0, false);
    add_ball(&mut runtime, 2, 100.0, 0.0, 0.0, 2.0, false);

    let visible = runtime
        .dispatch(request(
            "destiny.Ballpark.CheckVisibility",
            "call",
            Some("park:0"),
            vec![json!(1), json!(2)],
        ))
        .expect("visibility");
    assert_eq!(visible, json!(0));

    add_ball(&mut runtime, 3, 50.0, 0.0, 0.0, 25.0, true);
    let blocked = runtime
        .dispatch(request(
            "destiny.Ballpark.CheckVisibility",
            "call",
            Some("park:0"),
            vec![json!(1), json!(2)],
        ))
        .expect("blocked visibility");
    assert_eq!(blocked, json!(3));

    runtime
        .dispatch(request(
            "destiny.Ballpark.SetBallMassive",
            "call",
            Some("park:0"),
            vec![json!(3), json!(false)],
        ))
        .expect("nonmassive");
    assert_eq!(
        runtime
            .dispatch(request(
                "destiny.Ballpark.CheckVisibility",
                "call",
                Some("park:0"),
                vec![json!(1), json!(2)],
            ))
            .expect("visibility"),
        json!(0)
    );

    runtime
        .dispatch(request(
            "destiny.Ballpark.SetBallMassive",
            "call",
            Some("park:0"),
            vec![json!(3), json!(true)],
        ))
        .expect("massive");
    runtime
        .dispatch(request(
            "destiny.Ballpark.CloakBall",
            "call",
            Some("park:0"),
            vec![json!(3), json!(1)],
        ))
        .expect("cloak");
    assert_eq!(
        runtime
            .dispatch(request(
                "destiny.Ballpark.CheckVisibility",
                "call",
                Some("park:0"),
                vec![json!(1), json!(2)],
            ))
            .expect("visibility"),
        json!(0)
    );
}

// Original tests:
// - test_cloaked_balls_are_cloaked
// - test_cloaked_balls_are_not_massive
// - test_uncloaked_balls_are_not_cloaked
// - test_uncloaking_makes_a_ball_massive
#[test]
fn original_visibility_cloak_and_nonwarp_uncloak_transitions_match() {
    let mut runtime = runtime();
    add_ball(&mut runtime, 1, 0.0, 0.0, 0.0, 2.0, true);

    runtime
        .dispatch(request(
            "destiny.Ballpark.CloakBall",
            "call",
            Some("park:0"),
            vec![json!(1), json!(1)],
        ))
        .expect("cloak");
    let cloaked = ball_state(&mut runtime, 1);
    assert_eq!(cloaked["is_cloaked"], json!(1));
    assert_eq!(cloaked["is_massive"], json!(false));

    runtime
        .dispatch(request(
            "destiny.Ballpark.UncloakBall",
            "call",
            Some("park:0"),
            vec![json!(1)],
        ))
        .expect("uncloak");
    let uncloaked = ball_state(&mut runtime, 1);
    assert_eq!(uncloaked["is_cloaked"], json!(0));
    assert_eq!(uncloaked["is_massive"], json!(true));

    runtime
        .dispatch(request(
            "destiny.Ballpark.SetBallMassive",
            "call",
            Some("park:0"),
            vec![json!(1), json!(false)],
        ))
        .expect("nonmassive");
    runtime
        .dispatch(request(
            "destiny.Ballpark.UncloakBall",
            "call",
            Some("park:0"),
            vec![json!(1)],
        ))
        .expect("uncloak already-uncloaked ball");
    assert_eq!(ball_state(&mut runtime, 1)["is_massive"], json!(true));
}

// Original tests:
// - test_scan_cone_finds_nothing_when_there_is_nothing_to_be_found
// - test_scan_cone_finds_ball_when_it_is_in_the_cone
// - test_scan_cone_excludes_balls_not_in_cone
#[test]
fn original_visibility_scan_cone_axis_examples_match() {
    let mut empty = runtime();
    add_ball(&mut empty, 1, 0.0, 0.0, 0.0, 2.0, false);
    assert_eq!(
        empty
            .dispatch(request(
                "destiny.Ballpark.ScanCone",
                "call",
                Some("park:0"),
                vec![
                    json!(1),
                    json!(core::f64::consts::FRAC_PI_2),
                    json!(100.0),
                    json!(1.0),
                    json!(0.0),
                    json!(0.0),
                ],
            ))
            .expect("empty scan"),
        json!([])
    );

    let mut inside = runtime();
    add_ball(&mut inside, 1, 0.0, 0.0, 0.0, 2.0, false);
    add_ball(&mut inside, 2, 50.0, 0.0, 0.0, 2.0, false);
    assert_eq!(
        inside
            .dispatch(request(
                "destiny.Ballpark.ScanCone",
                "call",
                Some("park:0"),
                vec![
                    json!(1),
                    json!(core::f64::consts::FRAC_PI_2),
                    json!(100.0),
                    json!(1.0),
                    json!(0.0),
                    json!(0.0),
                ],
            ))
            .expect("inside scan"),
        json!([2])
    );

    let mut outside = runtime();
    add_ball(&mut outside, 1, 0.0, 0.0, 0.0, 2.0, false);
    add_ball(&mut outside, 2, -50.0, 0.0, 0.0, 2.0, false);
    add_ball(&mut outside, 3, 0.0, 50.0, 0.0, 2.0, false);
    add_ball(&mut outside, 4, 0.0, -50.0, 0.0, 2.0, false);
    add_ball(&mut outside, 5, 0.0, 0.0, 50.0, 2.0, false);
    add_ball(&mut outside, 6, 0.0, 0.0, -50.0, 2.0, false);
    assert_eq!(
        outside
            .dispatch(request(
                "destiny.Ballpark.ScanCone",
                "call",
                Some("park:0"),
                vec![
                    json!(1),
                    json!(core::f64::consts::FRAC_PI_2),
                    json!(100.0),
                    json!(1.0),
                    json!(0.0),
                    json!(0.0),
                ],
            ))
            .expect("outside scan"),
        json!([])
    );
}
