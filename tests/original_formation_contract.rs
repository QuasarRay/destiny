use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{CompatOptions, CompatRequest, CompatRuntime};
use serde_json::{Value, json};

fn dispatch(runtime: &mut CompatRuntime, title: &str, target: &str, args: Vec<Value>) -> Value {
    runtime
        .dispatch(CompatRequest {
            title: title.to_owned(),
            operation: if title.ends_with("formationID") {
                "get"
            } else {
                "call"
            }
            .to_owned(),
            target: Some(target.to_owned()),
            args,
            kwargs: Default::default(),
        })
        .expect(title)
}

fn park(runtime: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    dispatch(runtime, &format!("destiny.Ballpark.{name}"), "park:0", args)
}

fn ball(runtime: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    dispatch(runtime, &format!("destiny.Ball.{name}"), "ball:1", args)
}

fn fixture() -> Value {
    serde_json::from_str::<Value>(include_str!("fixtures/original_formations.json"))
        .expect("extracted original fixture")["value"]
        .clone()
}

fn new_runtime() -> CompatRuntime {
    let mut runtime = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        ..Default::default()
    })
    .expect("runtime");
    park(
        &mut runtime,
        "AddBall",
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
    );
    runtime
}

fn configured_runtime() -> CompatRuntime {
    let mut runtime = new_runtime();
    park(&mut runtime, "LoadFormations", vec![fixture()]);
    park(&mut runtime, "SetBallFormation", vec![json!(1), json!(1)]);
    runtime
}

fn capture(runtime: &mut CompatRuntime) -> (Value, Value) {
    let captured = dispatch(
        runtime,
        "dbc.compat.Ballpark.CaptureSnapshot",
        "park:0",
        vec![json!(-1)],
    );
    let encoded = captured["snapshot"].clone();
    let bytes = BASE64
        .decode(encoded.as_str().expect("encoded snapshot"))
        .expect("decode");
    (encoded, serde_json::from_slice(&bytes).expect("snapshot"))
}

// The original detached-Ball construction adapter is still missing; this
// bridge checks the same unassigned-formation invariant on an in-park ball.
#[test]
fn original_unassigned_in_park_ball_cannot_reserve_formation_slots() {
    let mut runtime = new_runtime();
    assert_eq!(ball(&mut runtime, "formationID", vec![]), json!(255));
    assert_eq!(
        ball(&mut runtime, "ReserveFormationSlot", vec![]),
        json!(-1)
    );
}

macro_rules! original_formation_case {
    ($name:ident, |$runtime:ident| $body:block) => {
        #[test]
        fn $name() {
            let mut $runtime = configured_runtime();
            $body
        }
    };
}

// original-test: python/destiny/test/test_ball.py::test_reserve_formation_slot
original_formation_case!(original_first_formation_slot_is_zero, |runtime| {
    assert_eq!(ball(&mut runtime, "ReserveFormationSlot", vec![]), json!(0));
});

// original-test: python/destiny/test/test_ball.py::test_slots_are_reserved_in_incremental_order
original_formation_case!(
    original_formation_slots_are_reserved_in_incremental_order,
    |runtime| {
        for expected in 0..fixture()[1][1]
            .as_array()
            .expect("original Arrow offsets")
            .len()
        {
            assert_eq!(
                ball(&mut runtime, "ReserveFormationSlot", vec![]),
                json!(expected)
            );
        }
    }
);

// original-test: python/destiny/test/test_ball.py::test_reserving_too_many_formation_slots_fails
original_formation_case!(
    original_exhausted_formation_returns_negative_one,
    |runtime| {
        for _ in 0..fixture()[1][1]
            .as_array()
            .expect("original Arrow offsets")
            .len()
        {
            ball(&mut runtime, "ReserveFormationSlot", vec![]);
        }
        assert_eq!(
            ball(&mut runtime, "ReserveFormationSlot", vec![]),
            json!(-1)
        );
    }
);

// original-test: python/destiny/test/test_ball.py::test_free_formation_slot
original_formation_case!(original_freed_formation_slot_two_is_reused, |runtime| {
    for _ in 0..16 {
        ball(&mut runtime, "ReserveFormationSlot", vec![]);
    }
    ball(&mut runtime, "FreeFormationSlot", vec![json!(2)]);
    assert_eq!(ball(&mut runtime, "ReserveFormationSlot", vec![]), json!(2));
});

// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_valid_formation_gets_set
// original-test: python/destiny/test/ballpark/test_getters_and_setters.py::test_formation_out_of_range_does_not_get_set
#[test]
fn original_formation_assignment_accepts_valid_ids_and_ignores_out_of_range_ids() {
    let mut runtime = new_runtime();
    park(&mut runtime, "LoadFormations", vec![fixture()]);
    park(&mut runtime, "SetBallFormation", vec![json!(1), json!(3)]);
    assert_eq!(ball(&mut runtime, "formationID", vec![]), json!(255));
    park(&mut runtime, "SetBallFormation", vec![json!(1), json!(0)]);
    assert_eq!(ball(&mut runtime, "formationID", vec![]), json!(0));
    ball(&mut runtime, "ReserveFormationSlot", vec![]);
    park(&mut runtime, "SetBallFormation", vec![json!(1), json!(3)]);
    assert_eq!(ball(&mut runtime, "formationID", vec![]), json!(0));
    assert_eq!(ball(&mut runtime, "ReserveFormationSlot", vec![]), json!(1));
    park(&mut runtime, "SetBallFormation", vec![json!(99), json!(0)]);
}

original_formation_case!(
    original_clearing_a_formation_resets_its_reserved_slots,
    |runtime| {
        ball(&mut runtime, "ReserveFormationSlot", vec![]);
        park(&mut runtime, "SetBallFormation", vec![json!(1), json!(-1)]);
        assert_eq!(
            ball(&mut runtime, "ReserveFormationSlot", vec![]),
            json!(-1)
        );
        park(&mut runtime, "SetBallFormation", vec![json!(1), json!(1)]);
        assert_eq!(ball(&mut runtime, "ReserveFormationSlot", vec![]), json!(0));
    }
);

original_formation_case!(
    original_invalid_frees_preserve_other_formation_slots,
    |runtime| {
        ball(&mut runtime, "ReserveFormationSlot", vec![]);
        for slot in [-1, 4, 16, i64::MAX] {
            ball(&mut runtime, "FreeFormationSlot", vec![json!(slot)]);
        }
        assert_eq!(ball(&mut runtime, "ReserveFormationSlot", vec![]), json!(1));
    }
);

#[test]
fn original_formations_with_more_than_sixteen_slots_cannot_reserve_slots() {
    let mut runtime = new_runtime();
    park(
        &mut runtime,
        "LoadFormations",
        vec![json!([["oversized", vec![[0.0; 3]; 17]]])],
    );
    park(&mut runtime, "SetBallFormation", vec![json!(1), json!(0)]);
    assert_eq!(
        ball(&mut runtime, "ReserveFormationSlot", vec![]),
        json!(-1)
    );
}

original_formation_case!(
    original_reloading_definitions_preserves_existing_reservations,
    |runtime| {
        ball(&mut runtime, "ReserveFormationSlot", vec![]);
        park(&mut runtime, "LoadFormations", vec![json!([])]);
        assert_eq!(
            ball(&mut runtime, "ReserveFormationSlot", vec![]),
            json!(-1)
        );
        park(&mut runtime, "LoadFormations", vec![fixture()]);
        assert_eq!(ball(&mut runtime, "ReserveFormationSlot", vec![]), json!(1));
    }
);

original_formation_case!(
    partial_formation_restore_rejects_changed_definitions_atomically,
    |runtime| {
        ball(&mut runtime, "ReserveFormationSlot", vec![]);
        let (_, mut changed) = capture(&mut runtime);
        let before = changed.clone();
        changed["park"]["formations"][1][0][0] = json!(555.0);
        let encoded = BASE64.encode(serde_json::to_vec(&changed).expect("changed snapshot"));
        assert!(
            runtime
                .dispatch(CompatRequest {
                    title: "dbc.compat.Ballpark.Deserialize".into(),
                    operation: "call".into(),
                    target: Some("park:0".into()),
                    args: vec![json!(encoded), json!(2)],
                    kwargs: Default::default(),
                })
                .is_err()
        );
        assert_eq!(capture(&mut runtime).1, before);
    }
);

#[test]
fn original_sixteen_slot_formation_includes_the_last_bit_and_reuses_it() {
    let mut runtime = new_runtime();
    park(
        &mut runtime,
        "LoadFormations",
        vec![json!([["full", vec![[0.0; 3]; 16]]])],
    );
    park(&mut runtime, "SetBallFormation", vec![json!(1), json!(0)]);
    for slot in 0..16 {
        assert_eq!(
            ball(&mut runtime, "ReserveFormationSlot", vec![]),
            json!(slot)
        );
    }
    assert_eq!(
        ball(&mut runtime, "ReserveFormationSlot", vec![]),
        json!(-1)
    );
    ball(&mut runtime, "FreeFormationSlot", vec![json!(15)]);
    assert_eq!(
        ball(&mut runtime, "ReserveFormationSlot", vec![]),
        json!(15)
    );
}

original_formation_case!(
    original_formation_snapshot_roundtrip_preserves_the_next_available_slot,
    |runtime| {
        ball(&mut runtime, "ReserveFormationSlot", vec![]);
        let (encoded, snapshot) = capture(&mut runtime);
        assert_eq!(snapshot["balls"][0]["formation_slots"], json!(1));
        let mut restored = new_runtime();
        dispatch(
            &mut restored,
            "dbc.compat.Ballpark.Deserialize",
            "park:0",
            vec![encoded, json!(0)],
        );
        assert_eq!(ball(&mut restored, "formationID", vec![]), json!(1));
        assert_eq!(
            ball(&mut restored, "ReserveFormationSlot", vec![]),
            json!(1)
        );
    }
);

original_formation_case!(
    original_formation_fields_default_when_loading_an_older_snapshot,
    |runtime| {
        let (_, mut snapshot) = capture(&mut runtime);
        snapshot["park"]
            .as_object_mut()
            .expect("park")
            .remove("formations");
        let object = snapshot["balls"][0].as_object_mut().expect("ball");
        object.remove("formation_id");
        object.remove("formation_slots");
        let encoded = BASE64.encode(serde_json::to_vec(&snapshot).expect("old snapshot"));
        dispatch(
            &mut runtime,
            "dbc.compat.Ballpark.Deserialize",
            "park:0",
            vec![json!(encoded), json!(0)],
        );
        assert_eq!(ball(&mut runtime, "formationID", vec![]), json!(255));
        assert_eq!(
            ball(&mut runtime, "ReserveFormationSlot", vec![]),
            json!(-1)
        );
    }
);

original_formation_case!(
    original_malformed_formation_load_is_rejected_without_replacing_definitions,
    |runtime| {
        let before = capture(&mut runtime).1;
        assert!(
            runtime
                .dispatch(CompatRequest {
                    title: "destiny.Ballpark.LoadFormations".into(),
                    operation: "call".into(),
                    target: Some("park:0".into()),
                    args: vec![json!([["bad", [[1.0, 2.0]]]])],
                    kwargs: Default::default(),
                })
                .is_err()
        );
        assert_eq!(capture(&mut runtime).1, before);
    }
);
