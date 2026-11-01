use bevy::{math::DVec3, prelude::*};
use destiny_partition::{get_or_create_box, Partition, PartitionBall, PartitionBox, PartitionStatic};
use destiny_partitionable::{BoxKey, Partitionable};

fn app() -> App {
    let mut app = App::new();
    app.init_resource::<Partition>();
    app
}
fn with_id(id: i64) -> Partitionable {
    let mut p = Partitionable::default();
    p.id = id;
    p
}
#[test]
fn box_cpp_constructor_bounds_defaults_and_parent() {
    let mut app = app();
    let entity = get_or_create_box(app.world_mut(), 5, 9, 13, 7).unwrap();
    let world = app.world();
    let child = world.get::<PartitionBox>(entity).unwrap();
    assert_eq!(child.key, BoxKey::new(5, 9, 13, 1 << 37));
    assert_eq!(child.level, 7);
    assert_eq!(child.hi - child.lo, DVec3::splat(480.0));
    assert_eq!(child.bubble, -1);
    assert!(!child.handled);
    assert!(child.balls.is_empty() && child.statics.is_empty() && child.children.is_empty());
    let parent = world.get::<PartitionBox>(child.parent.unwrap()).unwrap();
    assert_eq!((parent.key.ix,parent.key.iy,parent.key.iz),(1,2,3));
    assert_eq!(parent.children.get(&child.key),Some(&entity));
    assert_eq!(Partition::topmost(world,Some(entity))
        .and_then(|e|world.get::<PartitionBox>(e)).map(|b|b.level),Some(0));
}

#[test]
fn duplicate_children_and_bad_parents_are_rejected() {
    let mut app = app();
    let a = get_or_create_box(app.world_mut(), 5, 9, 13, 7).unwrap();
    let other = get_or_create_box(app.world_mut(), 8, 8, 8, 6).unwrap();
    let parent = app.world().get::<PartitionBox>(a).unwrap().parent.unwrap();
    assert!(!Partition::add_child(app.world_mut(),parent,a));
    assert!(!Partition::add_child(app.world_mut(),other,a));
    assert_eq!(app.world().get::<PartitionBox>(parent).unwrap().children.len(),1);
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
        app.world_mut().spawn((with_id(id),PartitionStatic {parent_ball_id:parent})).id()
    }).collect();
    app.world_mut().resource_scope(|world,p:Mut<'_,Partition>| {
        for &e in &objects { assert!(p.insert_static(world,box_entity,e)); }
    });
    let ordered: Vec<Entity> = app.world().get::<PartitionBox>(box_entity).unwrap().statics.values().copied().collect();
    assert_eq!(ordered,vec![objects[2],objects[0],objects[1]]);
}

#[test]
fn same_key_equality_follows_cpp_not_entity_identity() {
    let mut app = app();
    let e=get_or_create_box(app.world_mut(),3,2,1,7).unwrap();
    let box_ref=app.world().get::<PartitionBox>(e).unwrap();
    let copy=box_ref.clone();
    assert!(box_ref.same_key(&copy));
    assert!(copy.same_key(box_ref));
}

#[test]
fn box_traverse_visits_26_neighbors_only_at_same_level() {
    let mut app=app();
    let mid=get_or_create_box(app.world_mut(),10,10,10,7).unwrap();
    let diagonal=get_or_create_box(app.world_mut(),11,11,11,7).unwrap();
    let outside=get_or_create_box(app.world_mut(),14,10,10,7).unwrap();
    app.world_mut().resource_scope(|world,p:Mut<'_,Partition>|p.traverse(world,mid,101));
    assert_eq!(app.world().get::<PartitionBox>(diagonal).unwrap().bubble,101);
    assert_eq!(app.world().get::<PartitionBox>(outside).unwrap().bubble,-1);
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
    let parent=app.world().get::<PartitionBox>(bx).unwrap().parent.unwrap();
    app.world_mut().resource_scope(|world,mut p:Mut<'_,Partition>| {
        assert!(p.release_box(world,bx));
    });
    assert!(app.world().get::<PartitionBox>(bx).is_none());
    assert!(app.world().get::<PartitionBox>(parent).is_some());
    let parent_box=app.world().get::<PartitionBox>(parent).unwrap();
    assert_eq!(parent_box.children.len(),1);
    assert_eq!(parent_box.children.values().next().copied(),Some(sibling));
    assert_eq!(app.world().resource::<Partition>().size(),8);
}

#[test]
fn python_object_reference_replacement_and_despawn_are_ecs_owned() {
    use destiny_partition::PartitionBox as Box;
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
