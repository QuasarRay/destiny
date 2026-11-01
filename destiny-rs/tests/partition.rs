use bevy::{math::DVec3, prelude::*};
use destiny_rs::partition::{
    get_or_create_box, BallMode, NearbyCriteria, Partition, PartitionBall, PartitionBox,
    PartitionPlugin, PartitionStatic, py_get_box_center, register_partition_python,
};
use destiny_rs::partitionable::Partitionable;
use pyo3::types::PyAnyMethods;

fn partitionable_with_id(id: i64) -> Partitionable {
    let mut p = Partitionable::default();
    p.mId = id;
    p
}

fn setup()->App {
    let mut app=App::new();
    app.add_plugins(PartitionPlugin);
    app
}

#[test]
fn constructor_matches_cpp_grid() {
    let p=Partition::default();
    assert_eq!(p.number_of_levels(),8);
    assert_eq!(p.level_width[0],7_864_320.0);
    assert_eq!(p.level_width[7],480.0);
    assert_eq!(p.level_grid[0],1<<23);
    assert_eq!(p.level_grid[7],1<<37);
    assert_eq!(p.size(),0);
}

#[test]
fn out_of_bounds_levels_and_coordinates() {
    let p=Partition::default();
    assert_eq!(p.get_box_key(DVec3::ONE,-1),(0,0,0));
    assert_eq!(p.get_box_center(8,DVec3::ZERO),None);
    let mut a=setup();
    assert_eq!(get_or_create_box(a.world_mut(),1,2,3,-1),None);
    assert_eq!(get_or_create_box(a.world_mut(),1,2,3,8),None);
    assert_eq!(a.world().resource::<Partition>().size(),0);
}

#[test]
fn creates_coarser_ancestors_and_reuses_clamped_keys() {
    let mut app=setup();
    let one=get_or_create_box(app.world_mut(),9,13,17,7).unwrap();
    let two=get_or_create_box(app.world_mut(),9,13,17,7).unwrap();
    assert_eq!(one,two);
    let node=app.world().get::<PartitionBox>(one).unwrap();
    assert_eq!((node.mKey.ix,node.mKey.iy,node.mKey.iz),(9,13,17));
    assert_eq!(node.hi-node.lo,DVec3::splat(480.0));
    assert_eq!(Partition::topmost(app.world(),Some(one))
        .and_then(|e|app.world().get::<PartitionBox>(e))
        .map(|b|b.mLevel),Some(0));
    assert_eq!(app.world().resource::<Partition>().size(),8);
    let grid=app.world().resource::<Partition>().level_grid[7];
    let boundary=get_or_create_box(app.world_mut(),-500,grid+7,17,7).unwrap();
    let b=app.world().get::<PartitionBox>(boundary).unwrap();
    assert_eq!(b.mKey.ix,0);
    assert_eq!(b.mKey.iy,grid-1);
}

#[test]
fn lookup_does_not_create_and_checks_offset() {
    let mut app=setup();
    let a=get_or_create_box(app.world_mut(),3,3,3,6).unwrap();
    assert_eq!(app.world().resource::<Partition>().check_box(4,3,3,6),None);
    let b=app.world_mut().resource_scope(|world,mut p:Mut<'_,Partition>|{
        p.get_offset_box(world,Some(a),(1,0,0))
    }).unwrap();
    assert_eq!(app.world().resource::<Partition>()
        .check_offset_box(app.world(),Some(a),(1,0,0)),Some(b));
}

#[test]
fn box_center_matches_cpp_truncation_and_python_contract() {
    let p=Partition::default();
    let width=p.level_width[7];
    let center=p.get_box_center(7,DVec3::ZERO).unwrap();
    assert_eq!(center,DVec3::splat(width/2.0));
    assert_eq!(py_get_box_center(7,0.0,0.0,0.0).unwrap(),(240.0,240.0,240.0));
    assert!(py_get_box_center(-1,0.0,0.0,0.0).is_err());
}

#[test]
fn active_boxes_and_traversal_bubbles() {
    let mut app=setup();
    let a=get_or_create_box(app.world_mut(),10,10,10,7).unwrap();
    let b=get_or_create_box(app.world_mut(),11,10,10,7).unwrap();
    assert_eq!(app.world().resource::<Partition>().nearby_bubbles(app.world(),a),None);
    app.world_mut().resource_scope(|world,part:Mut<'_,Partition>|part.traverse(world,a,55));
    assert_eq!(app.world().get::<PartitionBox>(b).unwrap().mBubble,55);
    let act=app.world().resource::<Partition>().active_boxes(app.world(),7).unwrap();
    assert_eq!(act.1.len(),2);
    app.world_mut().resource_scope(|world,part:Mut<'_,Partition>|part.clear_traversal_flag(world));
    let top=Partition::topmost(app.world(),Some(a)).unwrap();
    assert_eq!(app.world().get::<PartitionBox>(top).unwrap().mBubble,-1);
}

#[test]
fn membership_is_bidirectional_and_pruned() {
    let mut app=setup();
    let b=get_or_create_box(app.world_mut(),15,15,15,7).unwrap();
    let object=app.world_mut().spawn((Partitionable::default(),PartitionBall::default())).id();
    app.world_mut().resource_scope(|world,part:Mut<'_,Partition>|{
        assert!(part.insert_ball(world,b,object));
    });
    assert_eq!(app.world().get::<Partitionable>(object).unwrap().boxes().count(),1);
    app.world_mut().resource_scope(|world,mut part:Mut<'_,Partition>|{
        part.remove_ball(world,b,object)
    });
    assert!(app.world().get::<PartitionBox>(b).is_none());
    assert_eq!(app.world().resource::<Partition>().size(),0);
    assert_eq!(app.world().get::<Partitionable>(object).unwrap().boxes().count(),0);
}

#[test]
fn nearby_filters_avoid_missiles_and_preserve_statically_ordered_objects() {
    let mut app=setup();
    let bx=get_or_create_box(app.world_mut(),3,3,3,7).unwrap();
    let origin=app.world_mut().spawn((Partitionable::default(),PartitionBall::default())).id();
    let target=app.world_mut().spawn((
        partitionable_with_id(42),
        PartitionBall {massive:true,free:true,..Default::default()},
    )).id();
    let missile=app.world_mut().spawn((
        partitionable_with_id(43),
        PartitionBall {massive:true,mode:BallMode::Missile,..Default::default()},
    )).id();
    app.world_mut().resource_scope(|world,part:Mut<'_,Partition>|{
        assert!(part.insert_ball(world,bx,origin));
        assert!(part.insert_ball(world,bx,target));
        assert!(part.insert_ball(world,bx,missile));
    });
    let p=app.world().resource::<Partition>();
    let result=p.nearby_candidates(app.world(),origin,false,NearbyCriteria::CASE_1,1);
    assert_eq!(result.balls,vec![target]);
}

#[test]
fn statics_share_box_membership_without_duplicate_results() {
    let mut app=setup();
    let box_a=get_or_create_box(app.world_mut(),1,1,1,7).unwrap();
    let box_b=get_or_create_box(app.world_mut(),2,1,1,7).unwrap();
    let source=app.world_mut().spawn((Partitionable::default(),PartitionBall::default())).id();
    let stat=app.world_mut().spawn((partitionable_with_id(7),destiny_rs::StaticCollidable {mParentBallId:22})).id();
    app.world_mut().resource_scope(|world,part:Mut<'_,Partition>|{
        part.insert_ball(world,box_a,source);
        part.insert_static(world,box_a,stat);
        part.insert_static(world,box_b,stat);
    });
    let result=app.world().resource::<Partition>()
        .nearby_candidates(app.world(),source,false,NearbyCriteria::default(),1);
    assert_eq!(result.static_collidables,vec![stat]);
}

#[test]
fn blue_python_get_box_center_is_registered() {
    let app=setup();
    let registry=app.world().resource::<blueexposure::BlueRegistry>();
    assert!(registry.functions().any(|f|f.qualified_name()=="destiny.GetBoxCenter"));
    pyo3::Python::initialize();
    pyo3::Python::attach(|py|->pyo3::PyResult<()> {
        let module=pyo3::types::PyModule::new(py,"destiny")?;
        register_partition_python(&module)?;
        let result=module.getattr("GetBoxCenter")?.call1((7,0.0,0.0,0.0))?;
        assert_eq!(result.extract::<(f64,f64,f64)>()?,(240.0,240.0,240.0));
        Ok(())
    }).unwrap();
}
