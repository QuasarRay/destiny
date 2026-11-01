use bevy::{math::DVec3, prelude::*};
use destiny_rs::partitionable::{BoxKey, Partitionable};

#[test]
fn cpp_constructor_defaults_match() {
    let p = Partitionable::default();
    assert_eq!(p.mId, 0);
    assert!(!p.mHandled);
    assert_eq!(p.mNewPos, DVec3::ZERO);
    assert_eq!(p.mOldPos, DVec3::ZERO);
    assert_eq!(p.mNewBubble, -1);
    assert_eq!(p.mOldBubble, -1);
    assert!(!p.isInteractive);
    assert_eq!(p.boxes().count(), 0);
    assert_eq!(p.mMyBox, None);
    assert_eq!(p.mPark, None);
    assert_eq!(p.mEffectStamp, 0);
    assert!(!p.isMoribund);
}

#[test]
fn entity_component_uses_bevy_world_storage() {
    let mut world = World::new();
    let entity = world.spawn(Partitionable::default()).id();
    let pos = DVec3::new(1e10, -2.5, 0.125);
    world.get_mut::<Partitionable>(entity).unwrap().mNewPos = pos;
    assert_eq!(world.get::<Partitionable>(entity).unwrap().mNewPos, pos);
}

#[test]
fn key_clamps_like_cpp_partition_key() {
    assert_eq!(
        BoxKey::new(-7, 200, 2, 8),
        BoxKey { ix: 0, iy: 7, iz: 2, n: 8 }
    );
}

#[test]
fn box_membership_is_ordered_by_cpp_box_key_not_entity_id() {
    let mut world = World::new();
    let ids: Vec<Entity> = (0..3).map(|_| world.spawn_empty().id()).collect();
    let mut p = Partitionable::default();
    let later_key = BoxKey::new(9, 0, 0, 16);
    let earlier_key = BoxKey::new(1, 0, 0, 16);
    let mid_key = BoxKey::new(4, 0, 0, 16);
    assert!(p.track_box(later_key, ids[0]));
    assert!(p.track_box(earlier_key, ids[1]));
    assert!(p.track_box(mid_key, ids[2]));
    let actual: Vec<_> = p.boxes().collect();
    assert_eq!(actual, vec![
        (earlier_key, ids[1]), (mid_key, ids[2]), (later_key, ids[0]),
    ]);
}

#[test]
fn box_membership_has_cpp_set_uniqueness_by_key() {
    let mut world = World::new();
    let a = world.spawn_empty().id();
    let b = world.spawn_empty().id();
    let key = BoxKey::new(1, 2, 3, 8);
    let mut p = Partitionable::default();
    assert!(p.track_box(key, a));
    assert!(!p.track_box(key, b));
    assert_eq!(p.boxes().collect::<Vec<_>>(), vec![(key, a)]);
}

#[test]
fn clear_only_forgets_own_references() {
    let mut world = World::new();
    let box_entity = world.spawn_empty().id();
    let mut p = Partitionable::default();
    p.track_box(BoxKey::new(1, 2, 3, 8), box_entity);
    p.mMyBox = Some(box_entity);
    p.mPark = Some(world.spawn_empty().id());
    p.forget_box_references();
    assert_eq!(p.boxes().count(), 0);
    assert_eq!(p.mMyBox, None);
    assert!(p.mPark.is_some(), "C++ ClearBoxes does not clear the park");
}
