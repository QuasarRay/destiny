//! Source old/scratch/current pose ownership in the actual headless engine.
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
fn add(r: &mut CompatRuntime, id: i64, p: [f64; 3], v: [f64; 3]) {
    park(
        r,
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
            json!(p[0]),
            json!(p[1]),
            json!(p[2]),
            json!(v[0]),
            json!(v[1]),
            json!(v[2]),
            json!(1.),
            json!(0.),
        ],
    );
}
#[derive(bevy::prelude::Resource, Default)]
struct DuringPhysics(Option<[f64; 6]>);
fn observe(world: &mut bevy::prelude::World) {
    let mut q = world.query::<(
        &DestinyBallId,
        &avian3d::prelude::Position,
        &avian3d::prelude::LinearVelocity,
    )>();
    let found = q
        .iter(world)
        .find(|(id, ..)| id.0 == 1)
        .map(|(_, p, v)| [p.x, p.y, p.z, v.x, v.y, v.z]);
    if found.is_some() {
        world.resource_mut::<DuringPhysics>().0 = found;
    }
}
#[test]
fn source_current_pose_does_not_advance_inside_the_avian_schedule() {
    let mut r = CompatRuntime::new_with_app(
        CompatOptions {
            enable_network_components: false,
            tick_interval_ms: 250.,
            ..Default::default()
        },
        |app| {
            app.init_resource::<DuringPhysics>()
                .add_systems(bevy::prelude::Last, observe);
        },
    )
    .unwrap();
    r.dispatch(CompatRequest {
        title: "destiny.Ballpark.friction".into(),
        operation: "set".into(),
        target: Some("park:0".into()),
        args: vec![json!(5.)],
        kwargs: Default::default(),
    })
    .unwrap();
    add(&mut r, 1, [0., 1., 1.], [500., 0., 0.]);
    park(&mut r, "Evolve", vec![]);
    assert_eq!(
        r.world().resource::<DuringPhysics>().0.unwrap()[..3],
        [0., 1., 1.],
        "source mNewPos remains current until the publication swap"
    );
}

fn host(iterative: bool) -> CompatRuntime {
    CompatRuntime::new(CompatOptions {
        enable_network_components: false,
        use_iterative_collision: iterative,
        tick_interval_ms: 250.,
        ..Default::default()
    })
    .unwrap()
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
fn restore(r: &mut CompatRuntime, encoded: Value) {
    r.dispatch(CompatRequest {
        title: "dbc.compat.Ballpark.Deserialize".into(),
        operation: "call".into(),
        target: Some("park:0".into()),
        args: vec![encoded],
        kwargs: Default::default(),
    })
    .unwrap();
}
fn old_bits(r: &CompatRuntime, id: i64) -> Vec<u64> {
    let p = r.source_old_pose(id).unwrap();
    p.position
        .into_iter()
        .chain(p.velocity)
        .map(f64::to_bits)
        .collect()
}
#[test]
fn old_pose_snapshot_preserves_all_bits_and_legacy_absence_is_explicit() {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use destiny_bevy_compat::{DestinyBallMetadata, DestinyOldPose};
    let mut r = host(false);
    add(&mut r, 1, [2., 3., 4.], [5., 6., 7.]);
    let bits = [
        0x8000000000000000,
        0x7ff8000000000035,
        0x7ff0000000000000,
        0xfff0000000000000,
        1,
        0xfff8000000000047,
    ];
    {
        let w = r.world_mut();
        let mut q = w.query::<(&DestinyBallId, &mut DestinyBallMetadata)>();
        let mut m = q.iter_mut(w).find(|(id, _)| id.0 == 1).unwrap().1;
        m.source_old_pose = Some(DestinyOldPose {
            position: [bits[0], bits[1], bits[2]].map(f64::from_bits),
            velocity: [bits[3], bits[4], bits[5]].map(f64::from_bits),
        });
    }
    let wire = capture(&mut r);
    let mut other = host(true);
    restore(&mut other, wire["snapshot"].clone());
    assert_eq!(old_bits(&other, 1), bits);
    let mut document: Value =
        serde_json::from_slice(&BASE64.decode(wire["snapshot"].as_str().unwrap()).unwrap())
            .unwrap();
    for b in document["balls"].as_array_mut().unwrap() {
        b.as_object_mut().unwrap().remove("source_old_pose");
    }
    restore(
        &mut other,
        json!(BASE64.encode(serde_json::to_vec(&document).unwrap())),
    );
    assert_eq!(
        old_bits(&other, 1),
        [2., 3., 4., 5., 6., 7.].map(f64::to_bits)
    );
    assert_eq!(other.source_acceleration(1).unwrap(), [0.; 3]);
    assert!(other.source_old_pose(999).is_err());
}
#[test]
fn source_old_pose_snapshot_replays_real_future_ticks_in_both_collision_modes() {
    for iterative in [false, true] {
        let mut first = host(iterative);
        add(&mut first, 1, [2., 3., 4.], [5., 6., 7.]);
        park(&mut first, "Evolve", vec![]);
        let wire = capture(&mut first);
        let mut second = host(!iterative);
        restore(&mut second, wire["snapshot"].clone());
        for _ in 0..8 {
            assert_eq!(old_bits(&first, 1), old_bits(&second, 1));
            assert_eq!(
                first.source_acceleration(1).unwrap().map(f64::to_bits),
                second.source_acceleration(1).unwrap().map(f64::to_bits)
            );
            park(&mut first, "Evolve", vec![]);
            park(&mut second, "Evolve", vec![]);
            assert_eq!(capture(&mut first), capture(&mut second));
        }
    }
}
#[test]
fn source_nonfree_and_cached_dead_bodies_retain_old_pose() {
    for iterative in [false, true] {
        let mut r = host(iterative);
        add(&mut r, 1, [2., 3., 4.], [5., 6., 7.]);
        park(&mut r, "SetBallFree", vec![json!(1), json!(false)]);
        let before = old_bits(&r, 1);
        park(&mut r, "Evolve", vec![]);
        assert_eq!(old_bits(&r, 1), before);
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
        assert_eq!(old_bits(&r, 1), before);
    }
}
