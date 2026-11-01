use bevy::{math::DVec3, prelude::*};
use destiny_rs::partition::{get_or_create_box, Partition, PartitionBall, PartitionBox, PartitionStatic};
use destiny_rs::partitionable::{BoxKey, Partitionable};

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Partition>();
    app
}
fn with_id(id: i64) -> Partitionable {
    let mut p = Partitionable::default();
    p.mId = id;
    p
}
#[test]
fn box_cpp_constructor_bounds_defaults_and_parent() {
    let mut app = app();
    let entity = get_or_create_box(app.world_mut(), 5, 9, 13, 7).unwrap();
    let world = app.world();
    let child = world.get::<PartitionBox>(entity).unwrap();
    assert_eq!(child.mKey, BoxKey::new(5, 9, 13, 1 << 37));
    assert_eq!(child.mLevel, 7);
    assert_eq!(child.hi - child.lo, DVec3::splat(480.0));
    assert_eq!(child.mBubble, -1);
    assert!(!child.mHandled);
    assert!(child.balls.is_empty() && child.mStaticCollidables.is_empty() && child.mChildren.is_empty());
    let parent = world.get::<PartitionBox>(child.mParent.unwrap()).unwrap();
    assert_eq!((parent.mKey.ix,parent.mKey.iy,parent.mKey.iz),(1,2,3));
    assert_eq!(parent.mChildren.get(&child.mKey),Some(&entity));
    assert_eq!(Partition::topmost(world,Some(entity))
        .and_then(|e|world.get::<PartitionBox>(e)).map(|b|b.mLevel),Some(0));
}

#[test]
fn duplicate_children_and_bad_parents_are_rejected() {
    let mut app = app();
    let a = get_or_create_box(app.world_mut(), 5, 9, 13, 7).unwrap();
    let other = get_or_create_box(app.world_mut(), 8, 8, 8, 6).unwrap();
    let parent = app.world().get::<PartitionBox>(a).unwrap().mParent.unwrap();
    assert!(!Partition::add_child(app.world_mut(),parent,a));
    assert!(!Partition::add_child(app.world_mut(),other,a));
    assert_eq!(app.world().get::<PartitionBox>(parent).unwrap().mChildren.len(),1);
}

#[test]
fn box_membership_keys_do_not_replace_existing_object() {
    let mut app = app();
    let box_entity = get_or_create_box(app.world_mut(), 4, 4, 4, 7).unwrap();
    let a = app.world_mut().spawn((with_id(44), PartitionBall::default())).id();
    let b = app.world_mut().spawn((with_id(44), PartitionBall::default())).id();
    app.world_mut().resource_scope(|world,p:Mut<'_,Partition>| {
        assert!(p.insert_ball(world,box_entity,a));
        assert!(!p.insert_ball(world,box_entity,b));
    });
    let box_ref = app.world().get::<PartitionBox>(box_entity).unwrap();
    assert_eq!(box_ref.balls.len(),1);
    assert_eq!(box_ref.balls.values().next().copied(),Some(a));
    assert_eq!(app.world().get::<Partitionable>(a).unwrap().boxes().count(),1);
    assert_eq!(app.world().get::<Partitionable>(b).unwrap().boxes().count(),0);
}

#[test]
fn static_collidable_keys_are_parent_first_and_unique() {
    let mut app = app();
    let box_entity = get_or_create_box(app.world_mut(), 1, 1, 1, 7).unwrap();
    let objects:Vec<_>=[(5,44),(9,5),(5,22)].into_iter().map(|(parent,id)|{
        app.world_mut().spawn((with_id(id),destiny_rs::StaticCollidable {mParentBallId:parent})).id()
    }).collect();
    app.world_mut().resource_scope(|world,p:Mut<'_,Partition>| {
        for &e in &objects { assert!(p.insert_static(world,box_entity,e)); }
    });
    let ordered: Vec<Entity> = app.world().get::<PartitionBox>(box_entity).unwrap().mStaticCollidables.values().copied().collect();
    assert_eq!(ordered,vec![objects[2],objects[0],objects[1]]);
}

#[test]
fn same_key_equality_follows_cpp_not_entity_identity() {
    let mut app = app();
    let e=get_or_create_box(app.world_mut(),3,2,1,7).unwrap();
    let other=get_or_create_box(app.world_mut(),4,2,1,7).unwrap();
    let box_ref=app.world().get::<PartitionBox>(e).unwrap();
    let other_ref=app.world().get::<PartitionBox>(other).unwrap();
    assert!(box_ref.same_key(box_ref));
    assert!(!box_ref.same_key(other_ref));
}

#[test]
fn box_traverse_visits_26_neighbors_only_at_same_level() {
    let mut app=app();
    let mid=get_or_create_box(app.world_mut(),10,10,10,7).unwrap();
    let diagonal=get_or_create_box(app.world_mut(),11,11,11,7).unwrap();
    let outside=get_or_create_box(app.world_mut(),14,10,10,7).unwrap();
    app.world_mut().resource_scope(|world,p:Mut<'_,Partition>|p.traverse(world,mid,101));
    assert_eq!(app.world().get::<PartitionBox>(diagonal).unwrap().mBubble,101);
    assert_eq!(app.world().get::<PartitionBox>(outside).unwrap().mBubble,-1);
    let p=app.world().resource::<Partition>();
    assert_eq!(p.nearby_bubbles(app.world(),outside),None);
    assert_eq!(p.nearby_bubbles(app.world(),mid),Some(101));
}

#[test]
fn release_refuses_nonempty_box_and_prunes_chain_on_last_ball() {
    let mut app=app();
    let bx=get_or_create_box(app.world_mut(),15,15,15,7).unwrap();
    let ball=app.world_mut().spawn((with_id(21),PartitionBall::default())).id();
    app.world_mut().resource_scope(|world,mut p:Mut<'_,Partition>|{
        p.insert_ball(world,bx,ball);
        assert!(!p.release_box(world,bx));
        p.remove_ball(world,bx,ball);
    });
    assert!(app.world().get::<PartitionBox>(bx).is_none());
    assert_eq!(app.world().resource::<Partition>().size(),0);
}

#[test]
fn release_empty_leaf_keeps_occupied_parent_and_sibling() {
    let mut app=app();
    let bx=get_or_create_box(app.world_mut(),10,10,10,7).unwrap();
    let sibling=get_or_create_box(app.world_mut(),11,10,10,7).unwrap();
    let parent=app.world().get::<PartitionBox>(bx).unwrap().mParent.unwrap();
    app.world_mut().resource_scope(|world,mut p:Mut<'_,Partition>| {
        assert!(p.release_box(world,bx));
    });
    assert!(app.world().get::<PartitionBox>(bx).is_none());
    assert!(app.world().get::<PartitionBox>(parent).is_some());
    let parent_box=app.world().get::<PartitionBox>(parent).unwrap();
    assert_eq!(parent_box.mChildren.len(),1);
    assert_eq!(parent_box.mChildren.values().next().copied(),Some(sibling));
    assert_eq!(app.world().resource::<Partition>().size(),8);
}

#[test]
fn python_object_reference_replacement_and_despawn_are_ecs_owned() {
    use destiny_rs::PartitionBox as Box;
    use pyo3::{types::{PyAnyMethods, PyDict}, Python};
    let mut app=app();
    let entity=get_or_create_box(app.world_mut(),2,2,2,7).unwrap();
    Python::initialize();
    Python::attach(|py| {
        let first=PyDict::new(py);
        let second=PyDict::new(py);
        assert!(Box::set_object(app.world_mut(),entity,Some(first.clone().into_any().unbind())));
        assert!(Box::object(app.world(),entity).unwrap().bind(py).is(&first));
        assert!(Box::set_object(app.world_mut(),entity,Some(second.clone().into_any().unbind())));
        assert!(Box::object(app.world(),entity).unwrap().bind(py).is(&second));
        assert!(Box::set_object(app.world_mut(),entity,None));
        assert!(Box::object(app.world(),entity).is_none());
        assert!(Box::set_object(app.world_mut(),entity,Some(first.into_any().unbind())));
        app.world_mut().resource_scope(|world,mut p:Mut<'_,Partition>|{
            assert!(p.release_box(world,entity));
        });
        assert!(Box::object(app.world(),entity).is_none());
        assert!(!Box::set_object(app.world_mut(),entity,None));
    });
}

#[test]
fn canonical_box_methods_keep_original_ownership_and_bubble_sentinel() {
    let mut app = app();
    let first = get_or_create_box(app.world_mut(), 20, 20, 20, 7).unwrap();
    let next = get_or_create_box(app.world_mut(), 21, 20, 20, 7).unwrap();
    assert_eq!(destiny_rs::Box::NearbyBubbles(app.world(), first), -1);
    destiny_rs::Box::Traverse(app.world_mut(), first, 123);
    assert_eq!(app.world().get::<PartitionBox>(next).unwrap().mBubble, 123);
    assert_eq!(destiny_rs::Box::NearbyBubbles(app.world(), first), 123);
}

#[test]
fn original_box_addchildren_uses_key_uniqueness() {
    let mut app = app();
    let first = get_or_create_box(app.world_mut(), 24, 24, 24, 7).unwrap();
    let parent = app.world().get::<PartitionBox>(first).unwrap().mParent.unwrap();
    destiny_rs::Box::AddChildren(app.world_mut(), parent, Some(first));
    destiny_rs::Box::AddChildren(app.world_mut(), parent, None);
    assert_eq!(app.world().get::<PartitionBox>(parent).unwrap().mChildren.len(), 1);
}


#[test]
fn cpp_box_add_operations_preserve_unique_sort_keys_and_null_guards() {
    let mut app = app();
    let bx = get_or_create_box(app.world_mut(), 25, 25, 25, 7).unwrap();
    let first = app.world_mut().spawn((with_id(44), PartitionBall::default())).id();
    let equivalent = app.world_mut().spawn((with_id(44), PartitionBall::default())).id();
    assert!(!destiny_rs::Box::AddBall(app.world_mut(), bx, None));
    assert!(destiny_rs::Box::AddBall(app.world_mut(), bx, Some(first)));
    assert!(!destiny_rs::Box::AddBall(app.world_mut(), bx, Some(equivalent)));
    let b = app.world().get::<PartitionBox>(bx).unwrap();
    assert_eq!(b.balls.values().copied().collect::<Vec<_>>(), vec![first]);
    let one = app.world_mut().spawn((with_id(-3),
        destiny_rs::StaticCollidable { mParentBallId: 12 })).id();
    let duplicate = app.world_mut().spawn((with_id(-3),
        destiny_rs::StaticCollidable { mParentBallId: 12 })).id();
    assert!(!destiny_rs::Box::AddStaticCollidable(app.world_mut(), bx, None));
    assert!(destiny_rs::Box::AddStaticCollidable(app.world_mut(), bx, Some(one)));
    assert!(!destiny_rs::Box::AddStaticCollidable(app.world_mut(), bx, Some(duplicate)));
    assert_eq!(app.world().get::<PartitionBox>(bx).unwrap().mStaticCollidables.len(), 1);
}

#[test]
fn cpp_box_remove_operations_are_box_owned_and_prune_via_partition() {
    let mut app = app();
    let bx = get_or_create_box(app.world_mut(), 35, 35, 35, 7).unwrap();
    let ball = app.world_mut().spawn((with_id(100), PartitionBall::default())).id();
    app.world_mut().resource_scope(|world, p: Mut<'_, Partition>| {
        assert!(p.insert_ball(world, bx, ball));
    });
    app.world_mut().resource_scope(|world, mut p: Mut<'_, Partition>| {
        destiny_rs::Box::RemoveBall(&mut p, world, bx, ball);
    });
    assert!(app.world().get::<PartitionBox>(bx).is_none());
    assert_eq!(app.world().get::<Partitionable>(ball).unwrap().boxes().count(), 0);
    assert_eq!(app.world().resource::<Partition>().size(), 0);
}


#[test]
fn canonical_box_setparent_and_removechildren_match_owner_boundaries() {
    let mut app = app();
    let child = get_or_create_box(app.world_mut(), 5, 5, 5, 7).unwrap();
    let parent = app.world().get::<PartitionBox>(child).unwrap().mParent.unwrap();
    app.world_mut().resource_scope(|world, mut partition: Mut<'_, Partition>| {
        destiny_rs::Box::SetParent(&mut partition, world, child);
    });
    assert_eq!(app.world().get::<PartitionBox>(child).unwrap().mParent, Some(parent));
    assert_eq!(app.world().get::<PartitionBox>(parent).unwrap().mChildren.len(), 1);

    app.world_mut().resource_scope(|world, mut partition: Mut<'_, Partition>| {
        destiny_rs::Box::RemoveChildren(&mut partition, world, parent, child);
    });
    // The child is still alive, and the parent index was removed only after
    // its last child was unlinked (this direct call simulates the destructor).
    assert!(app.world().get::<PartitionBox>(parent).is_none());
}
