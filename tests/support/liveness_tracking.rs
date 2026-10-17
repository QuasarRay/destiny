//! Independent C++/CPython observations exercised against the production owner.
use crate::bubbles::{Ball, Tracking};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn integer(v: &Value) -> i64 {
    v.as_i64().unwrap()
}
fn groups(value: &[(i64, BTreeSet<i64>)]) -> Value {
    let map: BTreeMap<_, _> = value
        .iter()
        .filter(|(_, ids)| !ids.is_empty())
        .cloned()
        .collect();
    json!(map.into_iter().collect::<Vec<_>>())
}
#[test]
fn original_cpython_liveness_membership_and_all_mode_queries() {
    let data: Value = serde_json::from_str(include_str!(
        "../../verification/migration/liveness_oracle_samples.json"
    ))
    .unwrap();
    for scenario in data["scenarios"].as_array().unwrap() {
        let master = scenario["master"].as_bool().unwrap();
        let mut owner = Tracking::default();
        let mut flags = BTreeMap::<i64, (i64, bool)>::new();
        for (step, row) in scenario["rows"].as_array().unwrap().iter().enumerate() {
            let args = row["action"].as_array().unwrap();
            let id = args.get(1).map(integer).unwrap_or_default();
            match args[0].as_str().unwrap() {
                "seed" => {
                    flags.insert(id, (integer(&args[3]), args[4].as_bool().unwrap()));
                }
                "register" => owner.register_keep_alive(id, args[2].as_bool().unwrap()),
                "add" => owner.keep_alive_add(id, integer(&args[2])),
                "remove" => owner.keep_alive_remove(id, integer(&args[2])),
                "set_interactive" => owner.set_interactive_members(
                    id,
                    args[2].as_array().unwrap().iter().map(integer).collect(),
                ),
                "set_keep_alive" => owner.set_keep_alive_members(
                    id,
                    args[2].as_array().unwrap().iter().map(integer).collect(),
                ),
                "clear" => owner = Tracking::default(),
                "interactive" => {
                    let enabled = args[2].as_bool().unwrap();
                    let (bubble, was) = flags.get_mut(&id).unwrap();
                    if *was != enabled {
                        owner.interactive(id, *bubble, enabled);
                    }
                    *was = enabled;
                }
                "transition" => {
                    let new = integer(&args[3]);
                    let state = flags.get_mut(&id).unwrap();
                    state.0 = new;
                    owner.transition(Ball {
                        id,
                        old: integer(&args[2]),
                        new,
                        interactive: state.1,
                        global: false,
                        cloaked: false,
                        troll: false,
                    });
                }
                other => panic!("unhandled oracle action {other}"),
            }
            let actual = owner.liveness_state();
            let expected = &row["expected"];
            assert_eq!(
                json!(actual.registered),
                expected["registered"],
                "step {step}"
            );
            assert_eq!(
                groups(&actual.keep_alive),
                expected["keep_alive"],
                "step {step}"
            );
            let mut interactive = BTreeMap::<i64, BTreeSet<i64>>::new();
            for (bubble, id) in actual.interactives {
                interactive.entry(bubble).or_default().insert(id);
            }
            assert_eq!(
                json!(interactive.into_iter().collect::<Vec<_>>()),
                expected["interactives"],
                "step {step}"
            );
            for query in expected["queries"].as_array().unwrap() {
                let bubble = integer(&query[0]);
                let mode = integer(&query[1]) as u32;
                let count = owner.interactive_count(master, bubble);
                assert_eq!(count, query[3].as_u64().unwrap() as usize, "step {step}");
                assert_eq!(
                    crate::generated_liveness::is_dead(
                        master,
                        mode,
                        count,
                        owner.keep_alive_count(bubble)
                    ),
                    query[2].as_bool().unwrap(),
                    "step {step} bubble {bubble} mode {mode}"
                );
            }
        }
    }
}

#[test]
fn master_liveness_motion_decision_is_cached_until_tick_finish() {
    use crate::{CompatOptions, CompatRequest, CompatRuntime};
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .unwrap();
    let call = |runtime: &mut CompatRuntime,
                title: &str,
                operation: &str,
                target: &str,
                args: Vec<Value>| {
        runtime
            .dispatch(CompatRequest {
                title: title.into(),
                operation: operation.into(),
                target: Some(target.into()),
                args,
                kwargs: Default::default(),
            })
            .unwrap()
    };
    call(
        &mut runtime,
        "destiny.Ballpark.isMaster",
        "set",
        "park:0",
        vec![json!(true)],
    );
    call(
        &mut runtime,
        "destiny.Ballpark.AddBall",
        "call",
        "park:0",
        vec![
            json!(1),
            json!(13e6),
            json!(1.),
            json!(500.),
            json!(true),
            json!(false),
            json!(false),
            json!(false),
            json!(false),
            json!(1.),
            json!(1.),
            json!(1.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.9),
            json!(0.95),
        ],
    );
    call(
        &mut runtime,
        "destiny.Ballpark.Stop",
        "call",
        "park:0",
        vec![json!(1)],
    );
    call(
        &mut runtime,
        "destiny.Ballpark.SetBallVelocity",
        "call",
        "park:0",
        vec![json!(1), json!(20.), json!(0.), json!(0.)],
    );
    let bubble = call(
        &mut runtime,
        "destiny.Ball.newBubbleId",
        "get",
        "ball:1",
        vec![],
    )
    .as_i64()
    .unwrap();
    let position =
        |runtime: &mut CompatRuntime| call(runtime, "destiny.Ball.x", "get", "ball:1", vec![]);
    let initial = position(&mut runtime);
    runtime.begin_callback_tick(true).unwrap();
    runtime.add_keep_alive_member(99, bubble);
    assert!(!runtime.is_in_dead_bubble(1).unwrap());
    runtime.complete_callback_tick().unwrap();
    assert_eq!(
        position(&mut runtime),
        initial,
        "changing membership cannot recompute an earlier phase decision"
    );
    runtime.begin_callback_tick(true).unwrap();
    runtime.remove_keep_alive_member(99, bubble);
    assert!(runtime.is_in_dead_bubble(1).unwrap());
    runtime.complete_callback_tick().unwrap();
    let moved = position(&mut runtime);
    assert_ne!(moved, initial);
    runtime.begin_callback_tick(true).unwrap();
    runtime.complete_callback_tick().unwrap();
    assert_eq!(position(&mut runtime), moved);
}
