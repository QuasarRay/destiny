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


#[test]
fn original_cpp_names_and_single_authoritative_membership_set() {
    use destiny_rs::{ID, SortedSetOfBoxes};
    let original_id: ID = -123;
    let mut p = Partitionable::new();
    p.mId = original_id;
    let mut world = World::new();
    let one = world.spawn_empty().id();
    let two = world.spawn_empty().id();
    let a = BoxKey::new(4, 3, 2, 16);
    let b = BoxKey::new(7, 3, 2, 16);
    assert!(p.track_box(b, one));
    assert!(p.track_box(a, two));
    let set: &SortedSetOfBoxes = &p.mBoxes;
    assert_eq!(set.values().copied().collect::<Vec<_>>(), vec![two, one]);
    assert_eq!(p.boxes().map(|(_, e)| e).collect::<Vec<_>>(), vec![two, one]);
    assert_eq!(p.mId, -123);
}

#[test]
fn abstract_cpp_method_dispatch_is_original_named_on_partitionable() {
    use destiny_rs::PartitionableBehavior;

    struct ExampleDerived;

    #[allow(non_snake_case)]
    impl PartitionableBehavior for ExampleDerived {
        fn ClearBoxes(world: &mut World, object: Entity) {
            world.get_mut::<Partitionable>(object).unwrap().forget_box_references();
        }
        fn DeleteFromBoxes(world: &mut World, object: Entity) {
            Self::ClearBoxes(world, object);
        }
        fn InsertInBox(world: &mut World, object: Entity, box_entity: Option<Entity>) {
            if let Some(bx) = box_entity {
                let key = world.get::<destiny_rs::Box>(bx).unwrap().mKey;
                world.get_mut::<Partitionable>(object).unwrap().track_box(key, bx);
            }
        }
        fn GetBoundingRadius(_world: &World, _object: Entity) -> f32 { 6.25 }
        fn GetInflatedRadius(_world: &World, _object: Entity, dt: f64) -> f64 {
            6.25 + dt
        }
        fn SetBoxActive(world: &mut World, object: Entity, box_id: i32, isActive: bool) {
            let _ = (world, object, box_id, isActive);
        }
        fn InsertInBoxes(
            world: &mut World, object: Entity, box1: Option<Entity>,
            top: Option<Entity>, newBubbleId: i32,
        ) {
            let _ = (top, newBubbleId);
            Self::InsertInBox(world, object, box1);
        }
    }

    let mut app = App::new();
    app.init_resource::<destiny_rs::Partition>();
    let object = app.world_mut().spawn(Partitionable::new()).id();
    let bx = app.world_mut().resource_scope(|world, mut partition: Mut<'_, destiny_rs::Partition>| {
        partition.get_box(world, 3, 2, 1, 7).unwrap()
    });
    Partitionable::InsertInBox::<ExampleDerived>(app.world_mut(), object, Some(bx));
    assert_eq!(app.world().get::<Partitionable>(object).unwrap().mBoxes.len(), 1);
    assert_eq!(Partitionable::GetBoundingRadius::<ExampleDerived>(app.world(), object), 6.25);
    assert_eq!(Partitionable::GetInflatedRadius::<ExampleDerived>(app.world(), object, 1.5), 7.75);
    Partitionable::SetBoxActive::<ExampleDerived>(app.world_mut(), object, 8, true);
    Partitionable::ClearBoxes::<ExampleDerived>(app.world_mut(), object);
    assert_eq!(app.world().get::<Partitionable>(object).unwrap().mBoxes.len(), 0);
    Partitionable::InsertInBoxes::<ExampleDerived>(app.world_mut(), object, Some(bx), None, 9);
    Partitionable::DeleteFromBoxes::<ExampleDerived>(app.world_mut(), object);
    assert!(app.world().get::<Partitionable>(object).unwrap().mBoxes.is_empty());
}
