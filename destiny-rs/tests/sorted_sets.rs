use bevy::prelude::*;
use destiny_rs::{
    BallSortComparer, BoxSortComparer, Partition, PartitionBall, Partitionable, get_or_create_box,
};

fn ball(world: &mut World, id: i64, owner: i64) -> Entity {
    let mut p = Partitionable::default();
    p.mId = id;
    world.spawn((p, PartitionBall {owner_id: owner, ..Default::default()})).id()
}

#[test]
fn ball_sort_comparer_follows_original_cpp_negative_id_and_owner_rules() {
    let mut app = App::new();
    let ids = [
        ball(app.world_mut(), -10, 100),
        ball(app.world_mut(), -1, 50),
        ball(app.world_mut(), -5, 50),
        ball(app.world_mut(), 2, 999),
        ball(app.world_mut(), 3, -50),
    ];
    let comparer = BallSortComparer;
    // Both mini IDs: owner primary, then mini ID.
    assert_eq!(comparer.less(app.world(), ids[2], ids[1]), Some(true));
    assert_eq!(comparer.less(app.world(), ids[1], ids[0]), Some(true));
    // If either ID is nonnegative: only ID is compared.
    assert_eq!(comparer.less(app.world(), ids[0], ids[3]), Some(true));
    assert_eq!(comparer.less(app.world(), ids[3], ids[0]), Some(false));
    assert_eq!(comparer.less(app.world(), ids[3], ids[4]), Some(true));
    for &left in &ids {
        for &right in &ids {
            let less = comparer.less(app.world(), left, right).unwrap();
            let key = BallSortComparer::key(app.world(), left).unwrap();
            let other_key = BallSortComparer::key(app.world(), right).unwrap();
            assert_eq!(less, key < other_key);
        }
    }
}

#[test]
fn box_sort_comparer_follows_original_spatial_keys_not_entities() {
    let mut app = App::new();
    app.init_resource::<Partition>();
    // Spawn higher spatial key first to ensure Entity order is irrelevant.
    let high = app.world_mut().resource_scope(|world, mut partition: Mut<'_, Partition>| {
        partition.get_box(world, 35, 4, 2, 7).unwrap()
    });
    let low = app.world_mut().resource_scope(|world, mut partition: Mut<'_, Partition>| {
        partition.get_box(world, 5, 4, 2, 7).unwrap()
    });
    let comparer = BoxSortComparer;
    assert_eq!(comparer.less(app.world(), low, high), Some(true));
    assert_eq!(comparer.less(app.world(), high, low), Some(false));
    assert_eq!(comparer.less(app.world(), high, high), Some(false));
}
