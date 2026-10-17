//! Persistent source controller/collision force effects in the actual engine.
use destiny_bevy_compat::{CompatOptions, CompatRequest, CompatRuntime};
use serde_json::{Value, json};
fn request(
    r: &mut CompatRuntime,
    title: &str,
    operation: &str,
    target: &str,
    args: Vec<Value>,
) -> Value {
    r.dispatch(CompatRequest {
        title: title.into(),
        operation: operation.into(),
        target: Some(target.into()),
        args,
        kwargs: Default::default(),
    })
    .expect(title)
}
fn park(r: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    request(
        r,
        &format!("destiny.Ballpark.{name}"),
        "call",
        "park:0",
        args,
    )
}
fn property(r: &mut CompatRuntime, id: i64, name: &str, value: Value) {
    request(
        r,
        &format!("destiny.Ball.{name}"),
        "set",
        &format!("ball:{id}"),
        vec![value],
    );
}
fn get(r: &mut CompatRuntime, id: i64, name: &str) -> Value {
    request(
        r,
        &format!("destiny.Ball.{name}"),
        "get",
        &format!("ball:{id}"),
        vec![],
    )
}
fn value(bits: &Value) -> f64 {
    f64::from_bits(u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap())
}
fn scenario(iterative: bool, case: &Value) -> CompatRuntime {
    let (dead, live) = if iterative { (2, 1) } else { (1, 2) };
    let p = case["parameters"].as_array().unwrap();
    let mut r = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        tick_interval_ms: p[1].as_f64().unwrap() * 1000.,
        ..Default::default()
    })
    .unwrap();
    request(
        &mut r,
        "destiny.Ballpark.friction",
        "set",
        "park:0",
        vec![p[0].clone()],
    );
    for id in [1, 2] {
        park(
            &mut r,
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
                json!(0.),
                json!(1.),
                json!(1.),
                json!(0.),
                json!(0.),
                json!(0.),
                json!(0.),
                json!(0.),
            ],
        );
        park(&mut r, "SetBallFree", vec![json!(id), json!(true)]);
        for (name, index) in [
            ("mass", 2),
            ("Agility", 3),
            ("radius", 4),
            ("maxVelocity", 5),
        ] {
            property(&mut r, id, name, p[index].clone());
        }
        property(&mut r, id, "speedFraction", json!(1.));
    }
    // A real client controller tick establishes the retained force. No force
    // expected value is computed by the port or installed into a test double.
    park(
        &mut r,
        "GotoPoint",
        vec![
            json!(dead),
            case["goto"][0].clone(),
            case["goto"][1].clone(),
            case["goto"][2].clone(),
        ],
    );
    park(&mut r, "Evolve", vec![]);
    for id in [1, 2] {
        let base = if id == dead { 0 } else { 12 };
        park(
            &mut r,
            "SetBallPosition",
            vec![
                json!(id),
                case["initial"][base].clone(),
                case["initial"][base + 1].clone(),
                case["initial"][base + 2].clone(),
            ],
        );
        park(
            &mut r,
            "SetBallVelocity",
            vec![json!(id), json!(0.), json!(0.), json!(0.)],
        );
        property(&mut r, id, "isMassive", json!(true));
    }
    property(&mut r, dead, "isGlobal", json!(true));
    // Initialize only the live owner; switching master does not repair the
    // other sentinel. In iterative mode the live ID precedes the dead ID,
    // preserving GetNextCollisionFree's source pair-selection rule.
    park(&mut r, "InitializeBubbles", vec![json!(live)]);
    assert_eq!(get(&mut r, dead, "newBubbleId"), json!(-1));
    let bubble = get(&mut r, live, "newBubbleId").as_i64().unwrap();
    request(
        &mut r,
        "destiny.Ballpark.isMaster",
        "set",
        "park:0",
        vec![json!(true)],
    );
    r.add_keep_alive_member(999, bubble);
    assert!(r.is_in_dead_bubble(dead).unwrap());
    assert!(!r.is_in_dead_bubble(live).unwrap());
    r
}
#[test]
fn dead_global_force_changes_live_gradient_prediction_against_original_cpp() {
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    for case in report["cases"].as_array().unwrap() {
        let mut r = scenario(false, case);
        for (step, expected) in case["variants"]["retained"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            park(&mut r, "Evolve", vec![]);
            assert_eq!(
                serde_json::to_value(r.source_contact_ids(2).unwrap()).unwrap(),
                case["contact_variants"]["retained"][step],
                "source Potential contact order, {} step={step}",
                case["name"]
            );
            for (id, offset) in [(1, 0), (2, 12)] {
                for (axis, index) in [
                    ("x", 0),
                    ("y", 1),
                    ("z", 2),
                    ("vx", 3),
                    ("vy", 4),
                    ("vz", 5),
                ] {
                    assert_eq!(
                        get(&mut r, id, axis).as_f64().unwrap().to_bits(),
                        value(&expected[offset + index]).to_bits(),
                        "{} id={id} {axis}",
                        case["name"]
                    );
                }
                let forces = r.source_force_state(id).unwrap();
                for (index, actual) in forces.last_g.into_iter().chain(forces.last_c).enumerate() {
                    assert_eq!(
                        actual.to_bits(),
                        value(&expected[offset + 6 + index]).to_bits(),
                        "{} id={id} force {index}",
                        case["name"]
                    );
                }
            }
        }
    }
}

fn capture(r: &mut CompatRuntime) -> Value {
    request(
        r,
        "dbc.compat.Ballpark.CaptureSnapshot",
        "call",
        "park:0",
        vec![],
    )
}
fn restore(r: &mut CompatRuntime, wire: &Value) {
    request(
        r,
        "dbc.compat.Ballpark.Deserialize",
        "call",
        "park:0",
        vec![wire["snapshot"].clone()],
    );
}
fn force_bits(r: &CompatRuntime, id: i64) -> Vec<u64> {
    let f = r.source_force_state(id).unwrap();
    f.last_g
        .into_iter()
        .chain(f.last_c)
        .map(f64::to_bits)
        .collect()
}
#[test]
fn dead_candidate_receives_iterative_overlap_force_without_pose_publication() {
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    for case in report["cases"].as_array().unwrap() {
        let mut r = scenario(true, case);
        park(
            &mut r,
            "SetBallPosition",
            vec![json!(1), json!(0.5), json!(1.), json!(1.)],
        );
        park(&mut r, "Evolve", vec![]);
        for (id, offset) in [(2, 0), (1, 12)] {
            let expected: Vec<_> = (6..12)
                .map(|i| value(&case["overlap_expected_bits"][offset + i]).to_bits())
                .collect();
            assert_eq!(force_bits(&r, id), expected, "{} id={id}", case["name"]);
        }
        assert_eq!(get(&mut r, 2, "x"), json!(0.));
    }
}
#[test]
fn source_force_snapshot_replays_both_actual_collision_paths() {
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    let case = &report["cases"][1];
    for iterative in [false, true] {
        let mut original = scenario(iterative, case);
        park(&mut original, "Evolve", vec![]);
        let wire = capture(&mut original);
        let mut restored = CompatRuntime::new(CompatOptions {
            enable_network_components: false,
            ..Default::default()
        })
        .unwrap();
        restore(&mut restored, &wire);
        for id in [1, 2] {
            assert_eq!(force_bits(&original, id), force_bits(&restored, id));
            assert_eq!(
                original.source_contact_ids(id).unwrap(),
                restored.source_contact_ids(id).unwrap()
            );
        }
        for _ in 0..8 {
            park(&mut original, "Evolve", vec![]);
            park(&mut restored, "Evolve", vec![]);
            for id in [1, 2] {
                assert_eq!(force_bits(&original, id), force_bits(&restored, id));
                assert_eq!(
                    original.source_contact_ids(id).unwrap(),
                    restored.source_contact_ids(id).unwrap()
                );
                for axis in ["x", "y", "z", "vx", "vy", "vz"] {
                    assert_eq!(get(&mut original, id, axis), get(&mut restored, id, axis));
                }
            }
        }
    }
}
#[test]
fn source_force_reset_occurs_only_after_dead_body_becomes_live() {
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    let mut r = scenario(true, &report["cases"][1]);
    park(
        &mut r,
        "SetBallPosition",
        vec![json!(1), json!(0.5), json!(1.), json!(1.)],
    );
    park(&mut r, "Evolve", vec![]);
    assert!(force_bits(&r, 2).iter().any(|&bits| bits != 0));
    // A direct valid source dictionary set can contain the sentinel key, while
    // AddKeepAliveBall deliberately ignores that key. This wakes the old body.
    r.set_keep_alive_members(-1, [777].into_iter().collect());
    property(&mut r, 1, "isMassive", json!(false));
    park(&mut r, "Stop", vec![json!(2)]);
    park(&mut r, "Evolve", vec![]);
    assert_eq!(force_bits(&r, 2), vec![0; 6]);
}
#[test]
fn force_snapshot_bits_survive_nonfinite_values_and_old_snapshot_defaults() {
    use avian3d::prelude::Position;
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use destiny_bevy_compat::{DestinyBallId, DestinyBallMetadata, DestinyForceState};
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    let mut r = scenario(false, &report["cases"][1]);
    let bits = [
        0x8000000000000000,
        0x7ff8000000000035,
        0x7ff0000000000000,
        0xfff0000000000000,
        1,
        0x7ff0000000000039,
    ];
    {
        let w = r.world_mut();
        let mut q = w.query::<(&DestinyBallId, &mut DestinyBallMetadata, &Position)>();
        let (_, mut metadata, _) = q.iter_mut(w).find(|(id, ..)| id.0 == 1).unwrap();
        metadata.source_forces = DestinyForceState {
            last_g: [bits[0], bits[1], bits[2]].map(f64::from_bits),
            last_c: [bits[3], bits[4], bits[5]].map(f64::from_bits),
        };
    }
    let wire = capture(&mut r);
    let mut restored = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .unwrap();
    restore(&mut restored, &wire);
    assert_eq!(force_bits(&restored, 1), bits);
    let mut document: Value =
        serde_json::from_slice(&BASE64.decode(wire["snapshot"].as_str().unwrap()).unwrap())
            .unwrap();
    for ball in document["balls"].as_array_mut().unwrap() {
        ball.as_object_mut().unwrap().remove("source_forces");
    }
    let old = json!({"snapshot":BASE64.encode(serde_json::to_vec(&document).unwrap())});
    restore(&mut restored, &old);
    for id in [1, 2] {
        assert_eq!(force_bits(&restored, id), vec![0; 6]);
    }
}
