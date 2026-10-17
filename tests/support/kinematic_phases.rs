//! Exercise the same mutable staging/publication helpers reached by Evolve.
//! Expected state comes from independent unchanged C++ writes/swap/query.
use super::*;
fn call(r: &mut CompatRuntime, name: &str, args: Vec<Value>) {
    r.dispatch(CompatRequest {
        title: format!("destiny.Ballpark.{name}"),
        operation: "call".into(),
        target: Some("park:0".into()),
        args,
        kwargs: Default::default(),
    })
    .unwrap();
}
fn number(bits: &Value) -> f64 {
    f64::from_bits(u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap())
}
#[test]
fn source_old_new_pose_commands_and_callback_publication_match_original_cpp() {
    let report: Value = serde_json::from_str(include_str!(
        "../../verification/migration/kinematic_oracle_samples.json"
    ))
    .unwrap();
    for case in report["cases"].as_array().unwrap() {
        for iterative in [false, true] {
            let mut r = CompatRuntime::new(CompatOptions {
                enable_network_components: false,
                use_iterative_collision: iterative,
                tick_interval_ms: case["dt"].as_f64().unwrap() * 1000.,
                ..Default::default()
            })
            .unwrap();
            if let Some(friction) = case.get("friction") {
                r.dispatch(CompatRequest {
                    title: "destiny.Ballpark.friction".into(),
                    operation: "set".into(),
                    target: Some("park:0".into()),
                    args: vec![friction.clone()],
                    kwargs: Default::default(),
                })
                .unwrap();
            }
            for (step, action) in case["actions"].as_array().unwrap().iter().enumerate() {
                let id = action[0].as_i64().unwrap() + 1;
                let op = action[1].as_u64().unwrap();
                let values = action[2].as_array().unwrap();
                match op {
                    0 | 1 => call(
                        &mut r,
                        "AddBall",
                        vec![
                            json!(id),
                            json!(10.),
                            json!(0.5),
                            json!(500.),
                            json!(true),
                            json!(false),
                            json!(false),
                            json!(false),
                            json!(false),
                            values[0].clone(),
                            values[1].clone(),
                            values[2].clone(),
                            values[3].clone(),
                            values[4].clone(),
                            values[5].clone(),
                            json!(1.),
                            json!(0.),
                        ],
                    ),
                    2 | 3 => call(
                        &mut r,
                        if op == 2 {
                            "SetBallPosition"
                        } else {
                            "SetBallVelocity"
                        },
                        vec![
                            json!(id),
                            values[0].clone(),
                            values[1].clone(),
                            values[2].clone(),
                        ],
                    ),
                    4 => r
                        .stage_source_motion(Some(&[PreparedMotion {
                            id,
                            position: OriginalVec3 {
                                x: values[0].as_f64().unwrap(),
                                y: values[1].as_f64().unwrap(),
                                z: values[2].as_f64().unwrap(),
                            },
                            velocity: OriginalVec3 {
                                x: values[3].as_f64().unwrap(),
                                y: values[4].as_f64().unwrap(),
                                z: values[5].as_f64().unwrap(),
                            },
                            in_warp: false,
                        }]))
                        .unwrap(),
                    5 => r.publish_source_pose(id).unwrap(),
                    7 => call(&mut r, "Evolve", vec![]),
                    _ => panic!("unreviewed source operation"),
                }
                for (index, expected) in case["observations"][step]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    if expected.is_null() {
                        continue;
                    }
                    let id = index as i64 + 1;
                    let current = r.source_current_pose(id).unwrap();
                    let old = r.source_old_pose(id).unwrap();
                    let lanes: Vec<_> = current
                        .position
                        .into_iter()
                        .chain(current.velocity)
                        .chain(old.position)
                        .chain(old.velocity)
                        .chain(r.source_acceleration(id).unwrap())
                        .collect();
                    let expected: Vec<_> = expected["state_bits"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .chain(expected["acceleration_bits"].as_array().unwrap())
                        .map(number)
                        .collect();
                    assert_eq!(lanes.len(), expected.len());
                    for (lane, (actual, expected)) in lanes.into_iter().zip(expected).enumerate() {
                        assert_eq!(
                            actual.to_bits(),
                            expected.to_bits(),
                            "{} step={step} id={id} lane={lane}",
                            case["name"]
                        );
                    }
                }
            }
        }
    }
}
