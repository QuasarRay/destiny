//! Source contact ownership in the actual headless engine. Numeric expectations
//! are retained from unchanged C++ methods, independently of the Rust emitter.
use avian3d::prelude::CollidingEntities;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use destiny_bevy_compat::{CompatOptions, CompatRequest, CompatRuntime, DestinyBallId};
use serde_json::{Value, json};

fn park(r: &mut CompatRuntime, name: &str, args: Vec<Value>) -> Value {
    r.dispatch(CompatRequest {
        title: format!("destiny.Ballpark.{name}"),
        operation: "call".into(),
        target: Some("park:0".into()),
        args,
        kwargs: Default::default(),
    })
    .unwrap()
}
fn host(iterative: bool) -> CompatRuntime {
    let mut r = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        ..Default::default()
    })
    .unwrap();
    park(
        &mut r,
        "AddBall",
        vec![
            json!(1),
            json!(13_000_000.),
            json!(2.),
            json!(10.),
            json!(false),
            json!(false),
            json!(false),
            json!(true),
            json!(false),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.9),
            json!(0.95),
        ],
    );
    park(&mut r, "SetBallFree", vec![json!(1), json!(true)]);
    park(
        &mut r,
        "WarpTo",
        vec![json!(1), json!(200000.), json!(300000.), json!(-100000.)],
    );
    assert_eq!(
        r.dispatch(CompatRequest {
            title: "destiny.Ball.mode".into(),
            operation: "get".into(),
            target: Some("ball:1".into()),
            args: vec![],
            kwargs: Default::default()
        })
        .unwrap(),
        json!(3)
    );
    r
}
fn angular(r: &mut CompatRuntime) -> Value {
    park(
        r,
        "SetBallVelocity",
        vec![json!(1), json!(1.), json!(2.), json!(3.)],
    );
    let w = r.world_mut();
    let mut q = w.query::<(&DestinyBallId, &destiny_bevy_compat::DestinyBallMetadata)>();
    serde_json::to_value(q.iter(w).find(|(id, _)| id.0 == 1).unwrap().1.source_angles).unwrap()
}
#[test]
fn avian_contacts_do_not_change_source_warp_angle_admission() {
    for iterative in [false, true] {
        let mut clean = host(iterative);
        let mut avian = host(iterative);
        let w = avian.world_mut();
        let mut q = w.query::<(bevy::prelude::Entity, &DestinyBallId)>();
        let entity = q.iter(w).find(|(_, id)| id.0 == 1).unwrap().0;
        w.entity_mut(entity)
            .insert(CollidingEntities([entity].into_iter().collect()));
        assert_eq!(
            angular(&mut avian),
            angular(&mut clean),
            "source mCollisions is empty in both worlds; iterative={iterative}"
        );
    }
}

fn set_contacts(r: &mut CompatRuntime, ids: Vec<i64>) {
    let w = r.world_mut();
    let mut q = w.query::<(
        &DestinyBallId,
        &mut destiny_bevy_compat::DestinyBallMetadata,
    )>();
    q.iter_mut(w)
        .find(|(id, _)| id.0 == 1)
        .unwrap()
        .1
        .source_contacts = ids;
}
#[test]
fn actual_source_contacts_select_cpp_warp_angles_independently_of_avian() {
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    for iterative in [false, true] {
        for collided in [false, true] {
            let mut r = host(iterative);
            if collided {
                set_contacts(&mut r, vec![0, -7, 0]);
            }
            let actual = angular(&mut r);
            for (name, expected) in report["contact_angles"]["states"][usize::from(collided)]
                .as_object()
                .unwrap()
            {
                assert_eq!(
                    actual[name].as_u64().unwrap(),
                    u64::from_str_radix(expected.as_str().unwrap(), 16).unwrap(),
                    "{name}, iterative={iterative}, collided={collided}"
                );
            }
        }
    }
}
fn capture(r: &mut CompatRuntime) -> Value {
    r.dispatch(CompatRequest {
        title: "dbc.compat.Ballpark.CaptureSnapshot".into(),
        operation: "call".into(),
        target: Some("park:0".into()),
        args: vec![],
        kwargs: Default::default(),
    })
    .unwrap()
}
fn restore(r: &mut CompatRuntime, snapshot: Value) {
    r.dispatch(CompatRequest {
        title: "dbc.compat.Ballpark.Deserialize".into(),
        operation: "call".into(),
        target: Some("park:0".into()),
        args: vec![snapshot],
        kwargs: Default::default(),
    })
    .unwrap();
}
#[test]
fn contacts_snapshot_preserves_order_duplicates_full_ids_and_old_defaults() {
    let mut r = host(false);
    let ids: Vec<_> = [i64::MIN, 0, i64::MAX, -1, -2, 0]
        .into_iter()
        .cycle()
        .take(300)
        .collect();
    set_contacts(&mut r, ids.clone());
    let wire = capture(&mut r);
    let mut other = host(true);
    restore(&mut other, wire["snapshot"].clone());
    assert_eq!(other.source_contact_ids(1).unwrap(), ids);
    let mut document: Value =
        serde_json::from_slice(&BASE64.decode(wire["snapshot"].as_str().unwrap()).unwrap())
            .unwrap();
    for ball in document["balls"].as_array_mut().unwrap() {
        ball.as_object_mut().unwrap().remove("source_contacts");
    }
    restore(
        &mut other,
        json!(BASE64.encode(serde_json::to_vec(&document).unwrap())),
    );
    assert!(other.source_contact_ids(1).unwrap().is_empty());
    assert!(other.source_contact_ids(999).is_err());
    assert!(other.source_collision_location_ids(999).is_err());
}
#[test]
fn contacts_reset_only_after_live_free_behavior_admission() {
    for iterative in [false, true] {
        let mut r = host(iterative);
        park(&mut r, "Stop", vec![json!(1)]);
        park(&mut r, "SetBallFree", vec![json!(1), json!(false)]);
        set_contacts(&mut r, vec![7, 7, 0]);
        park(&mut r, "Evolve", vec![]);
        assert_eq!(r.source_contact_ids(1).unwrap(), vec![7, 7, 0]);
        park(&mut r, "SetBallFree", vec![json!(1), json!(true)]);
        r.dispatch(CompatRequest {
            title: "destiny.Ballpark.isMaster".into(),
            operation: "set".into(),
            target: Some("park:0".into()),
            args: vec![json!(true)],
            kwargs: Default::default(),
        })
        .unwrap();
        assert!(r.is_in_dead_bubble(1).unwrap());
        park(&mut r, "Evolve", vec![]);
        assert_eq!(r.source_contact_ids(1).unwrap(), vec![7, 7, 0]);
        r.set_keep_alive_members(-1, [999].into_iter().collect());
        park(&mut r, "Evolve", vec![]);
        assert!(r.source_contact_ids(1).unwrap().is_empty());
    }
}
fn property(r: &mut CompatRuntime, id: i64, name: &str, value: Value) {
    r.dispatch(CompatRequest {
        title: format!("destiny.Ball.{name}"),
        operation: "set".into(),
        target: Some(format!("ball:{id}")),
        args: vec![value],
        kwargs: Default::default(),
    })
    .unwrap();
}
fn collision_host(iterative: bool, obstacles: impl IntoIterator<Item = i64>) -> CompatRuntime {
    let mut r = CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        tick_interval_ms: 250.,
        ..Default::default()
    })
    .unwrap();
    r.dispatch(CompatRequest {
        title: "destiny.Ballpark.friction".into(),
        operation: "set".into(),
        target: Some("park:0".into()),
        args: vec![json!(5.)],
        kwargs: Default::default(),
    })
    .unwrap();
    for id in obstacles.into_iter().chain([500]) {
        let x = if id == 500 { 0. } else { 2. };
        park(
            &mut r,
            "AddBall",
            vec![
                json!(id),
                json!(10.),
                json!(0.5),
                json!(500.),
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
                json!(1.),
                json!(0.),
            ],
        );
        property(&mut r, id, "isMassive", json!(true));
    }
    park(&mut r, "SetBallFree", vec![json!(500), json!(true)]);
    park(
        &mut r,
        "SetBallVelocity",
        vec![json!(500), json!(500.), json!(0.), json!(0.)],
    );
    r
}
#[test]
fn gradient_retains_130_original_cpp_contact_ids_without_a_world_capacity_limit() {
    let mut r = collision_host(false, (-65..66).filter(|&id| id != -1));
    park(&mut r, "Evolve", vec![]);
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/force_oracle_samples.json"
    ))
    .unwrap();
    assert_eq!(
        serde_json::to_value(r.source_contact_ids(500).unwrap()).unwrap(),
        report["many_contacts"]["ids"]
    );
}
#[test]
fn iterative_location_ids_remain_separate_before_late_missile_dispatch() {
    let mut r = collision_host(true, 0..1);
    park(&mut r, "Evolve", vec![]);
    assert!(r.source_contact_ids(500).unwrap().is_empty());
    let locations = r.source_collision_location_ids(500).unwrap();
    assert!(
        !locations.is_empty(),
        "real iterative collision must produce a location"
    );
    assert!(
        locations.iter().all(|&id| id == 0),
        "zero is a valid collider ID, unlike source -1 sentinel: {locations:?}"
    );
}

fn force_words(value: &Value) -> [f64; 3] {
    std::array::from_fn(|i| {
        f64::from_bits(u64::from_str_radix(value[i].as_str().unwrap(), 16).unwrap())
    })
}
fn mutate_dispatch_state(
    r: &mut CompatRuntime,
    ids: Option<Vec<i64>>,
    action: Option<&Value>,
    force: Option<[f64; 3]>,
) {
    let w = r.world_mut();
    let mut q = w.query::<(
        &DestinyBallId,
        &mut destiny_bevy_compat::DestinyBallMetadata,
    )>();
    let mut metadata = q.iter_mut(w).find(|(id, _)| id.0 == 1).unwrap().1;
    if let Some(ids) = ids {
        metadata.source_contacts = ids;
    }
    if let Some(a) = action {
        if let Some(values) = a.get("append").and_then(Value::as_array) {
            metadata
                .source_contacts
                .extend(values.iter().map(|id| id.as_i64().unwrap()));
        }
        if let Some(values) = a.get("replace") {
            metadata.source_contacts[values[0].as_u64().unwrap() as usize] =
                values[1].as_i64().unwrap();
        }
    }
    if let Some(force) = force {
        metadata.source_forces.last_c = force;
    }
}
#[derive(Default)]
struct DispatchRecorder {
    trace: Vec<Value>,
    ordinal: usize,
    pending: Option<[f64; 3]>,
}
impl DispatchRecorder {
    fn observe(
        &mut self,
        r: &mut CompatRuntime,
        notice: destiny_bevy_compat::DestinyCollisionNotice,
        case: &Value,
    ) -> bool {
        use destiny_bevy_compat::DestinyCollisionNotice;
        match notice {
            DestinyCollisionNotice::Contact(event) => {
                assert_eq!(event.ball_id, 1);
                self.trace.push(json!({"kind":"contact","other_id":event.other_id,"force_bits":event.acceleration.map(|v|format!("{:016x}",v.to_bits()))}));
                let action = case["actions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|a| a["at"].as_u64().unwrap() as usize == self.ordinal)
                    .cloned();
                if let Some(a) = &action {
                    mutate_dispatch_state(r, None, Some(a), a.get("force_bits").map(force_words));
                }
                self.pending = action
                    .as_ref()
                    .and_then(|a| a.get("error_force_bits"))
                    .map(force_words);
                self.ordinal += 1;
                if action
                    .as_ref()
                    .and_then(|a| a.get("reenter"))
                    .and_then(Value::as_bool)
                    == Some(true)
                {
                    r.dispatch_source_collisions(1, &mut |r, n| self.observe(r, n, case))
                        .unwrap();
                }
                action
                    .as_ref()
                    .and_then(|a| a.get("success"))
                    .and_then(Value::as_bool)
                    .unwrap_or(true)
            }
            DestinyCollisionNotice::HostError { ball_id } => {
                assert_eq!(ball_id, 1);
                self.trace.push(json!({"kind":"host_error"}));
                mutate_dispatch_state(r, None, None, self.pending.take());
                true
            }
        }
    }
}
#[test]
fn direct_source_dispatch_matches_unchanged_cpp_reentrant_traces() {
    let report: Value = serde_json::from_str(include_str!(
        "../verification/migration/collision_dispatch_oracle_samples.json"
    ))
    .unwrap();
    for case in report["cases"].as_array().unwrap() {
        let mut r = host(false);
        let ids: Vec<i64> = serde_json::from_value(case["ids"].clone()).unwrap();
        mutate_dispatch_state(
            &mut r,
            Some(ids.clone()),
            None,
            Some(force_words(&case["force_bits"])),
        );
        let mut recorder = DispatchRecorder::default();
        let pass = r
            .dispatch_source_collisions(1, &mut |r, n| recorder.observe(r, n, case))
            .unwrap();
        assert_eq!(json!(recorder.trace), case["trace"], "{}", case["name"]);
        assert_eq!(pass.captured_count(), ids.len());
        assert_eq!(pass.delivered_count(), ids.len());
        assert_eq!(
            pass.host_error_count(),
            case.get("outer_host_errors")
                .unwrap_or(&case["host_errors"])
                .as_u64()
                .unwrap() as usize
        );
    }
}
#[test]
fn operation_owned_source_collision_cursors_support_nested_dispatch() {
    let mut r = host(false);
    set_contacts(&mut r, vec![9, 9, 0]);
    let mut outer = r.begin_source_collision_pass(1).unwrap();
    assert_eq!(
        r.next_source_collision(&mut outer)
            .unwrap()
            .unwrap()
            .other_id,
        9
    );
    let mut inner = r.begin_source_collision_pass(1).unwrap();
    assert_eq!(
        r.next_source_collision(&mut inner)
            .unwrap()
            .unwrap()
            .other_id,
        9
    );
    mutate_dispatch_state(
        &mut r,
        None,
        Some(&json!({"append":[99],"replace":[1,-7]})),
        Some([4., 5., 6.]),
    );
    let next = r.next_source_collision(&mut outer).unwrap().unwrap();
    assert_eq!(next.other_id, -7);
    assert_eq!(next.acceleration, [4., 5., 6.]);
    assert_eq!(
        r.next_source_collision(&mut inner)
            .unwrap()
            .unwrap()
            .other_id,
        -7
    );
    assert_eq!(
        r.next_source_collision(&mut outer)
            .unwrap()
            .unwrap()
            .other_id,
        0
    );
    assert!(r.next_source_collision(&mut outer).unwrap().is_none());
    assert_eq!(
        r.next_source_collision(&mut inner)
            .unwrap()
            .unwrap()
            .other_id,
        0
    );
    assert!(r.next_source_collision(&mut inner).unwrap().is_none());
    let mut next = r.begin_source_collision_pass(1).unwrap();
    assert_eq!(next.captured_count(), 4);
    assert_eq!(
        r.next_source_collision(&mut next)
            .unwrap()
            .unwrap()
            .other_id,
        9
    );
}
#[test]
fn unsafe_source_list_shrink_is_an_explicit_rust_error_after_prior_effects() {
    let mut r = host(false);
    set_contacts(&mut r, vec![9, 7]);
    let mut pass = r.begin_source_collision_pass(1).unwrap();
    assert_eq!(
        r.next_source_collision(&mut pass)
            .unwrap()
            .unwrap()
            .other_id,
        9
    );
    set_contacts(&mut r, vec![]);
    assert!(r.next_source_collision(&mut pass).is_err());
    assert_eq!(pass.delivered_count(), 1);
    assert!(r.begin_source_collision_pass(999).is_err());
}
#[test]
fn real_gradient_output_reaches_the_same_typed_source_dispatcher() {
    use destiny_bevy_compat::DestinyCollisionNotice;
    let mut r = collision_host(false, (-65..66).filter(|&id| id != -1));
    park(&mut r, "Evolve", vec![]);
    let expected = r.source_contact_ids(500).unwrap();
    let force = r.source_force_state(500).unwrap().last_c.map(f64::to_bits);
    let mut actual = Vec::new();
    let result = r
        .dispatch_source_collisions(500, &mut |_, notice| {
            let DestinyCollisionNotice::Contact(e) = notice else {
                panic!("no failing host")
            };
            actual.push(e.other_id);
            assert_eq!(e.acceleration.map(f64::to_bits), force);
            true
        })
        .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(result.delivered_count(), 130);
}

#[test]
fn source_collision_cursors_reject_other_worlds_and_cancel_replaced_owners() {
    let mut first = host(false);
    let other = host(false);
    set_contacts(&mut first, vec![9]);
    let mut pass = first.begin_source_collision_pass(1).unwrap();
    assert!(other.next_source_collision(&mut pass).is_err());
    assert_eq!(pass.delivered_count(), 0);
    assert_eq!(
        first
            .next_source_collision(&mut pass)
            .unwrap()
            .unwrap()
            .other_id,
        9
    );
    let mut stale = first.begin_source_collision_pass(1).unwrap();
    park(&mut first, "RemoveBall", vec![json!(1)]);
    park(
        &mut first,
        "AddBall",
        vec![
            json!(1),
            json!(13_000_000.),
            json!(2.),
            json!(10.),
            json!(false),
            json!(false),
            json!(false),
            json!(true),
            json!(false),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.),
            json!(0.9),
            json!(0.95),
        ],
    );
    set_contacts(&mut first, vec![777]);
    assert!(first.next_source_collision(&mut stale).unwrap().is_none());
    assert_eq!(stale.delivered_count(), 0);
}
