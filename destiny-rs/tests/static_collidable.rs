use bevy::{math::DVec3, prelude::*};
use destiny_rs::{Box, Partition, Partitionable, StaticCollidable, StaticCollidableGeometry,
    StaticCollidableSortComparer, get_or_create_box};

fn setup() -> App {
    let mut app = App::new();
    app.init_resource::<Partition>();
    app
}
fn spawn_static(world: &mut World, object_id: i64, parent_id: i64) -> Entity {
    let mut base = Partitionable::default();
    base.mId = object_id;
    world.spawn((base, StaticCollidable { mParentBallId: parent_id })).id()
}

#[test]
fn original_constructor_default_and_sort_order() {
    assert_eq!(StaticCollidable::default().mParentBallId, 0);
    let mut app = setup();
    let a = spawn_static(app.world_mut(), -7, 5);
    let b = spawn_static(app.world_mut(), 3, 5);
    let c = spawn_static(app.world_mut(), 10, 2);
    let comparer = StaticCollidableSortComparer;
    assert_eq!(comparer.less(app.world(), c, a), Some(true));
    assert_eq!(comparer.less(app.world(), a, b), Some(true));
    assert_eq!(comparer.less(app.world(), b, b), Some(false));
}

#[test]
fn insert_and_delete_boxes_keep_both_sides_in_sync() {
    let mut app = setup();
    let a = get_or_create_box(app.world_mut(), 7, 7, 7, 7).unwrap();
    let b = get_or_create_box(app.world_mut(), 8, 7, 7, 7).unwrap();
    let object = spawn_static(app.world_mut(), -9, 100);
    StaticCollidable::InsertInBox(app.world_mut(), object, None);
    StaticCollidable::InsertInBox(app.world_mut(), object, Some(a));
    StaticCollidable::InsertInBox(app.world_mut(), object, Some(b));
    let state = app.world().get::<Partitionable>(object).unwrap();
    assert_eq!(state.boxes().count(), 2);
    assert_eq!(app.world().get::<Box>(a).unwrap().mStaticCollidables.len(), 1);
    assert_eq!(app.world().get::<Box>(b).unwrap().mStaticCollidables.len(), 1);
    StaticCollidable::DeleteFromBoxes(app.world_mut(), object);
    assert_eq!(app.world().get::<Partitionable>(object).unwrap().boxes().count(), 0);
    assert_eq!(app.world().resource::<Partition>().size(), 0);
}

#[test]
fn clearboxes_is_local_only_as_in_original_cpp() {
    let mut app = setup();
    let a = get_or_create_box(app.world_mut(), 2, 2, 2, 7).unwrap();
    let object = spawn_static(app.world_mut(), 1, 3);
    StaticCollidable::InsertInBox(app.world_mut(), object, Some(a));
    StaticCollidable::ClearBoxes(app.world_mut(), object);
    assert_eq!(app.world().get::<Partitionable>(object).unwrap().boxes().count(), 0);
    assert_eq!(app.world().get::<Box>(a).unwrap().mStaticCollidables.len(), 1);
}

struct SphereGeometry;
#[allow(non_snake_case)]
impl StaticCollidableGeometry for SphereGeometry {
    fn GetCenter(&self) -> DVec3 { DVec3::ZERO }
    fn CollideWithBall(&self, _world: &mut World, _ball: Entity) {}
    fn CheckCollision(
        &self, _p0: &DVec3, _p1: &DVec3, _radius: f32,
        _normal: &mut DVec3, _timeOfImpact: &mut f64,
    ) -> bool { false }
    fn GetBoundingRadius(&self) -> f32 { 15.25 }
}
#[test]
fn original_inflated_radius_ignores_timestep_and_box_active_is_noop() {
    let provider = SphereGeometry;
    assert_eq!(StaticCollidable::GetInflatedRadius(&provider, 0.0), 15.25);
    assert_eq!(StaticCollidable::GetInflatedRadius(&provider, 5.0), 15.25);
    StaticCollidable::SetBoxActive(1, true);
}
