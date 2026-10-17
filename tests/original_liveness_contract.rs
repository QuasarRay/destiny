//! Source liveness semantics in the actual headless Bevy runtime.
use avian3d::prelude::{AngularVelocity, LinearVelocity, Position, RigidBody, Rotation};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{CompatOptions, CompatRequest, CompatRuntime, DestinyBallId};
use serde_json::{Value, json};
use std::collections::BTreeSet;
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
fn get(runtime: &mut CompatRuntime, id: i64, name: &str) -> Value {
    request(
        runtime,
        &format!("destiny.Ball.{name}"),
        "get",
        &format!("ball:{id}"),
        vec![],
    )
}
fn host(master: bool, iterative: bool) -> CompatRuntime {
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        ..Default::default()
    })
    .unwrap();
    request(
        &mut runtime,
        "destiny.Ballpark.isMaster",
        "set",
        "park:0",
        vec![json!(master)],
    );
    runtime
}
fn add(runtime: &mut CompatRuntime, id: i64, x: f64, free: bool) {
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
            json!(x),
            json!(1.),
            json!(1.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
        ],
    );
    if free {
        park(runtime, "SetBallFree", vec![json!(id), json!(true)]);
    }
}
fn moving(master: bool, iterative: bool) -> CompatRuntime {
    let mut runtime = host(master, iterative);
    add(&mut runtime, 1, 479., true);
    for (name, value) in [
        ("mass", 13e6),
        ("Agility", 0.9),
        ("maxVelocity", 500.),
        ("speedFraction", 0.95),
        ("radius", 2.),
    ] {
        property(&mut runtime, 1, name, json!(value));
    }
    park(&mut runtime, "Stop", vec![json!(1)]);
    park(
        &mut runtime,
        "SetBallVelocity",
        vec![json!(1), json!(20.), json!(3.), json!(-4.)],
    );
    property(&mut runtime, 1, "wx", json!(0.125));
    property(&mut runtime, 1, "wy", json!(-0.25));
    runtime
}
fn pose(runtime: &mut CompatRuntime) -> Vec<u64> {
    let ecs = runtime.world_mut();
    let mut q = ecs.query::<(
        &DestinyBallId,
        &Position,
        &LinearVelocity,
        &AngularVelocity,
        &Rotation,
        &RigidBody,
    )>();
    let (_, p, v, w, r, body) = q.iter(ecs).find(|(id, ..)| id.0 == 1).unwrap();
    assert_eq!(*body, RigidBody::Dynamic);
    p.0.to_array()
        .into_iter()
        .chain(v.0.to_array())
        .chain(w.0.to_array())
        .chain(r.0.to_array())
        .map(f64::to_bits)
        .collect()
}
fn snapshot(runtime: &mut CompatRuntime) -> (Value, Value) {
    let wire = request(
        runtime,
        "dbc.compat.Ballpark.CaptureSnapshot",
        "call",
        "park:0",
        vec![],
    );
    let value = serde_json::from_slice(&BASE64.decode(wire["snapshot"].as_str().unwrap()).unwrap())
        .unwrap();
    (wire, value)
}
fn restore(runtime: &mut CompatRuntime, wire: &Value) {
    request(
        runtime,
        "dbc.compat.Ballpark.Deserialize",
        "call",
        "park:0",
        vec![wire["snapshot"].clone()],
    );
}
#[test]
fn master_dead_bubble_freezes_source_phases() {
    for iterative in [false, true] {
        let mut master = moving(true, iterative);
        let mut client = moving(false, iterative);
        assert!(master.is_in_dead_bubble(1).unwrap());
        assert!(!client.is_in_dead_bubble(1).unwrap());
        let before = pose(&mut master);
        let (_, initial) = snapshot(&mut master);
        for _ in 0..5 {
            park(&mut master, "Evolve", vec![]);
            park(&mut client, "Evolve", vec![]);
            assert_eq!(pose(&mut master), before);
        }
        assert_ne!(pose(&mut client), before);
        let (_, final_state) = snapshot(&mut master);
        assert_eq!(
            initial["balls"][0]["source_angles"],
            final_state["balls"][0]["source_angles"]
        );
        assert_eq!(
            initial["park"]["source_partition_v1"]["partition"],
            final_state["park"]["source_partition_v1"]["partition"]
        );
        let bubble = get(&mut master, 1, "newBubbleId").as_i64().unwrap();
        master.register_keep_alive(1, true);
        assert!(master.is_in_dead_bubble(1).unwrap());
        master.add_keep_alive_member(999, bubble);
        assert!(!master.is_in_dead_bubble(1).unwrap());
        park(&mut master, "Evolve", vec![]);
        assert_ne!(pose(&mut master), before);
    }
}
#[test]
fn source_interactive_same_flag_and_keep_alive_registry_are_distinct() {
    let mut runtime = moving(true, true);
    let bubble = get(&mut runtime, 1, "newBubbleId").as_i64().unwrap();
    park(
        &mut runtime,
        "SetBallInteractive",
        vec![json!(1), json!(true)],
    );
    assert!(!runtime.is_in_dead_bubble(1).unwrap());
    runtime.set_interactive_members(bubble, BTreeSet::new());
    park(
        &mut runtime,
        "SetBallInteractive",
        vec![json!(1), json!(true)],
    );
    assert!(
        runtime.is_in_dead_bubble(1).unwrap(),
        "same flag does not restore source dictionary membership"
    );
    park(
        &mut runtime,
        "SetBallInteractive",
        vec![json!(1), json!(false)],
    );
    park(
        &mut runtime,
        "SetBallInteractive",
        vec![json!(1), json!(true)],
    );
    assert!(!runtime.is_in_dead_bubble(1).unwrap());
    park(
        &mut runtime,
        "SetBallInteractive",
        vec![json!(1), json!(false)],
    );
    runtime.register_keep_alive(1, true);
    assert!(runtime.is_in_dead_bubble(1).unwrap());
    runtime.add_keep_alive_member(1, bubble);
    runtime.register_keep_alive(1, false);
    assert!(!runtime.is_in_dead_bubble(1).unwrap());
    runtime.remove_keep_alive_member(1, bubble);
    assert!(runtime.is_in_dead_bubble(1).unwrap());
    park(
        &mut runtime,
        "WarpTo",
        vec![json!(1), json!(100000.), json!(200000.), json!(300000.)],
    );
    assert_eq!(get(&mut runtime, 1, "mode"), json!(3));
    assert!(!runtime.is_in_dead_bubble(1).unwrap());
}
#[test]
fn source_liveness_snapshot_and_future_replay_have_no_model_capacity_cap() {
    for iterative in [false, true] {
        let mut runtime = moving(true, iterative);
        let bubble = get(&mut runtime, 1, "newBubbleId").as_i64().unwrap();
        for id in 1000..1100 {
            runtime.register_keep_alive(id, true);
        }
        for group in 100..180 {
            runtime.set_keep_alive_members(group, (1000..1100).collect());
        }
        runtime.set_keep_alive_members(bubble, (1000..1100).collect());
        let expected = serde_json::to_value(runtime.liveness_state()).unwrap();
        let (wire, _) = snapshot(&mut runtime);
        let mut resumed = host(false, iterative);
        restore(&mut resumed, &wire);
        assert_eq!(
            serde_json::to_value(resumed.liveness_state()).unwrap(),
            expected
        );
        assert!(!resumed.is_in_dead_bubble(1).unwrap());
        for _ in 0..5 {
            park(&mut runtime, "Evolve", vec![]);
            park(&mut resumed, "Evolve", vec![]);
            assert_eq!(pose(&mut runtime), pose(&mut resumed));
        }
        park(&mut resumed, "ClearAll", vec![]);
        let cleared = resumed.liveness_state();
        assert!(cleared.registered.is_empty());
        assert!(cleared.keep_alive.is_empty());
        assert!(cleared.interactives.is_empty());
    }
}
#[test]
fn source_dead_sensor_final_pass_clears_members_then_live_sensor_reenters() {
    let mut runtime = host(true, true);
    add(&mut runtime, 3, 1e10, false);
    add(&mut runtime, 4, 1e10, true);
    let bubble = get(&mut runtime, 3, "newBubbleId").as_i64().unwrap();
    runtime.add_keep_alive_member(99, bubble);
    park(
        &mut runtime,
        "AddProximitySensor",
        vec![json!(3), json!(10.), json!(2.), json!(0), json!(false)],
    );
    let drain = |r: &mut CompatRuntime| {
        request(
            r,
            "dbc.compat.Proximity.DrainEvents",
            "call",
            "park:0",
            vec![],
        )
    };
    park(&mut runtime, "Evolve", vec![]);
    assert_eq!(drain(&mut runtime), json!([]));
    park(&mut runtime, "Evolve", vec![]);
    let entered = drain(&mut runtime);
    assert_eq!(entered.as_array().unwrap().len(), 1);
    assert_eq!(entered[0]["other_id"], json!(4));
    assert_eq!(entered[0]["entering"], json!(true));
    runtime.remove_keep_alive_member(99, bubble);
    assert!(runtime.is_in_dead_bubble(3).unwrap());
    park(&mut runtime, "Evolve", vec![]);
    assert_eq!(drain(&mut runtime), json!([]));
    let (_, state) = snapshot(&mut runtime);
    let owner = state["balls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["id"] == 3)
        .unwrap();
    assert_eq!(owner["sensors"][0]["members"], json!([]));
    park(&mut runtime, "Evolve", vec![]);
    assert_eq!(drain(&mut runtime), json!([]));
    runtime.add_keep_alive_member(99, bubble);
    park(&mut runtime, "Evolve", vec![]);
    assert_eq!(drain(&mut runtime), json!([]));
    park(&mut runtime, "Evolve", vec![]);
    let again = drain(&mut runtime);
    assert_eq!(again.as_array().unwrap().len(), 1);
    assert_eq!(again[0]["other_id"], json!(4));
    assert_eq!(again[0]["entering"], json!(true));
}
