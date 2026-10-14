// Copyright © 2015 CCP ehf. MIT license, retained from original Destiny tests.
// Generated from pinned Git blobs by generate_park_harnesses.py.
// Concrete state projection: 8 retained objects, 4096 ordered boundary calls.
// Spatial indexing, Blue dispatch/lifetime, interpreter and native libm
// correspondence remain required; no production refinement is claimed here.
#![allow(non_snake_case, unused_variables)]
use destiny_original_spec::{
    ActionValue, BoundedList, OriginalBallAttribute, OriginalBallInit, OriginalBoxObject,
    OriginalBoxObjectKey, OriginalBubbleDelta, OriginalBubbleDeltaMap, OriginalMiniBall,
    OriginalMiniBox, OriginalMiniCapsule, OriginalMode, OriginalParkBall, OriginalParkError,
    OriginalParkProjection, Vec3, python_almost_equal, python_rounded_equal,
};

fn box_object(values: &[(OriginalBoxObjectKey, ActionValue)]) -> OriginalBoxObject {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}

fn user_list(values: &[Option<i64>]) -> BoundedList<Option<i64>, 8> {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}
fn proximity_events(values: &[(i64, i64)]) -> BoundedList<(i64, i64), 8> {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}
fn transition_list(values: &[(i64, i64, i64)]) -> BoundedList<(i64, i64, i64), 8> {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}
fn delta_map(values: &[(i64, OriginalBubbleDelta)]) -> OriginalBubbleDeltaMap {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}
fn delta_maps_equal(left: &OriginalBubbleDeltaMap, right: &OriginalBubbleDeltaMap) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut i = 0;
    while i < left.len() {
        let value = left.get(i);
        let mut matches = 0;
        let mut j = 0;
        while j < right.len() {
            if value == right.get(j) {
                matches += 1;
            }
            j += 1;
        }
        if matches != 1 {
            return false;
        }
        i += 1;
    }
    true
}
fn store_delta_map(destination: &mut OriginalBubbleDeltaMap, value: OriginalBubbleDeltaMap) {
    *destination = value;
}

fn point_list<const N: usize>(points: &[Vec3]) -> BoundedList<Vec3, N> {
    let mut result = BoundedList::new();
    let mut i = 0_usize;
    while i < points.len() {
        if let Some(point) = points.get(i) {
            assert!(result.push(*point));
        }
        i = i.saturating_add(1);
    }
    result
}
fn points_exact_equal<const A: usize, const B: usize>(
    left: &BoundedList<Vec3, A>,
    right: &BoundedList<Vec3, B>,
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut i = 0;
    while i < left.len() {
        if left.get(i) != right.get(i) {
            return false;
        }
        i += 1;
    }
    true
}
fn point_sets_equal<const A: usize, const B: usize>(
    left: &BoundedList<Vec3, A>,
    right: &BoundedList<Vec3, B>,
) -> bool {
    let mut i = 0;
    while i < left.len() {
        let mut found = false;
        let mut j = 0;
        while j < right.len() {
            found |= left.get(i) == right.get(j);
            j += 1;
        }
        if !found {
            return false;
        }
        i += 1;
    }
    i = 0;
    while i < right.len() {
        let mut found = false;
        let mut j = 0;
        while j < left.len() {
            found |= right.get(i) == left.get(j);
            j += 1;
        }
        if !found {
            return false;
        }
        i += 1;
    }
    true
}
fn sort_points<const N: usize>(values: &BoundedList<Vec3, N>) -> BoundedList<Vec3, N> {
    let mut result = *values;
    let mut i = 1;
    while i < result.len() {
        let point = result.get(i);
        assert!(point.is_some());
        if let Some(point) = point {
            let mut j = i;
            while j > 0 {
                let previous = result.get(j - 1);
                assert!(previous.is_some());
                if let Some(previous) = previous {
                    if (previous.x, previous.y, previous.z) <= (point.x, point.y, point.z) {
                        break;
                    }
                    assert!(result.set(j, previous));
                }
                j -= 1;
            }
            assert!(result.set(j, point));
        }
        i += 1;
    }
    result
}
// helpers.py:129..150 compares each rounded coordinate only for unequal
// points, then checks list lengths. Preserve that order and all three axes.
fn assert_points_rounded<const A: usize, const B: usize>(
    first: &BoundedList<Vec3, A>,
    second: &BoundedList<Vec3, B>,
    places: u32,
) {
    let limit = first.len().min(second.len());
    let mut i = 0_usize;
    while i < limit {
        let left = first.get(i);
        let right = second.get(i);
        assert!(left.is_some() && right.is_some());
        if let (Some(left), Some(right)) = (left, right) {
            if left != right {
                assert!(python_rounded_equal(left.x, right.x, places) == Some(true));
                assert!(python_rounded_equal(left.y, right.y, places) == Some(true));
                assert!(python_rounded_equal(left.z, right.z, places) == Some(true));
            }
        }
        i = i.saturating_add(1);
    }
    assert!(first.len() == second.len());
}

fn id_list(values: &[i64]) -> BoundedList<i64, 8> {
    let mut result = BoundedList::new();
    let mut i = 0_usize;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i = i.saturating_add(1);
    }
    result
}
fn ids_contain(values: &BoundedList<i64, 8>, id: i64) -> bool {
    let mut i = 0_usize;
    while i < values.len() {
        if values.get(i) == Some(id) {
            return true;
        }
        i = i.saturating_add(1);
    }
    false
}
fn distance_list(values: &[(f64, i64)]) -> BoundedList<(f64, i64), 8> {
    let mut result = BoundedList::new();
    let mut i = 0_usize;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i = i.saturating_add(1);
    }
    result
}
fn query_ids(values: &BoundedList<(f64, i64), 8>) -> BoundedList<i64, 8> {
    let mut result = BoundedList::new();
    let mut i = 0_usize;
    while i < values.len() {
        if let Some((_, id)) = values.get(i) {
            assert!(result.push(id));
        }
        i = i.saturating_add(1);
    }
    result
}
fn id_sets_equal(left: &BoundedList<i64, 8>, right: &BoundedList<i64, 8>) -> bool {
    let mut i = 0_usize;
    while i < left.len() {
        if let Some(id) = left.get(i) {
            if !ids_contain(right, id) {
                return false;
            }
        }
        i = i.saturating_add(1);
    }
    i = 0;
    while i < right.len() {
        if let Some(id) = right.get(i) {
            if !ids_contain(left, id) {
                return false;
            }
        }
        i = i.saturating_add(1);
    }
    true
}

// Iterative-specific source helpers. Native float pow/sqrt and Python list
// indexing/alias correspondence remain separate executable requirements.
fn point_at<const N: usize>(values: &BoundedList<Vec3, N>, index: i64) -> Vec3 {
    let length = i64::try_from(values.len());
    assert!(length.is_ok());
    let actual = if index < 0 {
        length.unwrap_or(0).checked_add(index)
    } else {
        Some(index)
    };
    assert!(actual.is_some());
    let actual = actual.unwrap_or(-1);
    assert!(actual >= 0);
    let index = usize::try_from(actual);
    assert!(index.is_ok());
    let value = values.get(index.unwrap_or(usize::MAX));
    assert!(value.is_some());
    value.unwrap_or(Vec3::ZERO)
}
fn float_at<const N: usize>(values: &BoundedList<f64, N>, index: i64) -> f64 {
    let length = i64::try_from(values.len());
    assert!(length.is_ok());
    let actual = if index < 0 {
        length.unwrap_or(0).checked_add(index)
    } else {
        Some(index)
    };
    assert!(actual.is_some());
    let actual = actual.unwrap_or(-1);
    assert!(actual >= 0);
    let index = usize::try_from(actual);
    assert!(index.is_ok());
    let value = values.get(index.unwrap_or(usize::MAX));
    assert!(value.is_some());
    value.unwrap_or(0.0)
}
fn distance_between_points(a: Vec3, b: Vec3) -> f64 {
    // Complete pinned Python helper: sqrt((x1-x2)**2 + ...), in its order.
    ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0) + (a.z - b.z).powf(2.0)).sqrt()
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_stopped_balls_with_same_location
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_stopped_balls_with_same_location() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut a_coordinates = BoundedList::<Vec3, 200>::new();
    let mut b_coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_40 = 0_i64;
    while iteration_40 < (10.0_f64) as i64 {
        let i = iteration_40;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(a_coordinates.push(Vec3 {
            x: park.retained(ball_a).position.x,
            y: park.retained(ball_a).position.y,
            z: park.retained(ball_a).position.z
        })); // original complete point/velocity trace
        assert!(b_coordinates.push(Vec3 {
            x: park.retained(ball_b).position.x,
            y: park.retained(ball_b).position.y,
            z: park.retained(ball_b).position.z
        })); // original complete point/velocity trace
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:37
        assert!(
            python_almost_equal(
                park.retained(ball_a).velocity.x,
                -(park.retained(ball_b).velocity.x),
                7
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:38
        assert!(
            python_almost_equal(
                park.retained(ball_a).velocity.y,
                -(park.retained(ball_b).velocity.y),
                7
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:39
        assert!(
            python_almost_equal(
                park.retained(ball_a).velocity.z,
                -(park.retained(ball_b).velocity.z),
                7
            ) == Some(true)
        );
        iteration_40 += 1;
    }
    let distance = distance_between_points(
        point_at(&a_coordinates, (2.0_f64) as i64),
        point_at(&b_coordinates, (2.0_f64) as i64),
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:43
    assert!(
        (distance
            > (f64::from(park.retained(ball_a).radius) + f64::from(park.retained(ball_b).radius)))
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_at_angle
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_at_angle() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let mut xp = BoundedList::<f64, 3>::new();
    assert!(xp.push(0.0_f64));
    assert!(xp.push(50.0_f64));
    assert!(xp.push(-(50.0_f64)));
    let mut yp = BoundedList::<f64, 3>::new();
    assert!(yp.push(50.0_f64));
    assert!(yp.push(-(50.0_f64)));
    assert!(yp.push(0.0_f64));
    let mut zp = BoundedList::<f64, 3>::new();
    assert!(zp.push(-(50.0_f64)));
    assert!(zp.push(0.0_f64));
    assert!(zp.push(50.0_f64));
    let mut xv = BoundedList::<f64, 3>::new();
    assert!(xv.push(0.1_f64));
    assert!(xv.push(0.0_f64));
    assert!(xv.push(90.0_f64));
    let mut yv = BoundedList::<f64, 3>::new();
    assert!(yv.push(0.0_f64));
    assert!(yv.push(90.0_f64));
    assert!(yv.push(0.1_f64));
    let mut zv = BoundedList::<f64, 3>::new();
    assert!(zv.push(90.0_f64));
    assert!(zv.push(0.1_f64));
    assert!(zv.push(0.0_f64));
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_38 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_38,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_40 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_40,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_42 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_42,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_44 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_44,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_46 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_46,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_48 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_48,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut iteration_50 = 0_i64;
    while iteration_50 < (3.0_f64) as i64 {
        let i = iteration_50;
        let attribute_value_53 = float_at(&xp, i);
        park.write_attribute_notified(
            ball,
            |b| b.position.x = attribute_value_53,
            OriginalBallAttribute::Position,
        );
        let attribute_value_55 = float_at(&yp, i);
        park.write_attribute_notified(
            ball,
            |b| b.position.y = attribute_value_55,
            OriginalBallAttribute::Position,
        );
        let attribute_value_57 = float_at(&zp, i);
        park.write_attribute_notified(
            ball,
            |b| b.position.z = attribute_value_57,
            OriginalBallAttribute::Position,
        );
        let attribute_value_59 = float_at(&xv, i);
        park.write_attribute_notified(
            ball,
            |b| b.velocity.x = attribute_value_59,
            OriginalBallAttribute::Velocity,
        );
        let attribute_value_61 = float_at(&yv, i);
        park.write_attribute_notified(
            ball,
            |b| b.velocity.y = attribute_value_61,
            OriginalBallAttribute::Velocity,
        );
        let attribute_value_63 = float_at(&zv, i);
        park.write_attribute_notified(
            ball,
            |b| b.velocity.z = attribute_value_63,
            OriginalBallAttribute::Velocity,
        );
        let attribute_value_65 = (100.0_f64) as f32;
        park.write_attribute_notified(
            ball,
            |b| b.max_velocity = attribute_value_65,
            OriginalBallAttribute::MaxSpeed,
        );
        let attribute_value_67 = (0.5_f64) as f32;
        park.write_attribute_notified(
            ball,
            |b| b.agility = attribute_value_67,
            OriginalBallAttribute::Agility,
        );
        park.goto_direction(
            ball,
            Vec3 {
                x: float_at(&xv, i),
                y: float_at(&yv, i),
                z: float_at(&zv, i),
            },
        );
        let mut iteration_70 = 0_i64;
        while iteration_70 < (40.0_f64) as i64 {
            let j = iteration_70;
            assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
            if ((((i) as f64 == 0.0_f64) && (park.retained(ball).position.x > 100.0_f64))
                || (((i) as f64 == 1.0_f64) && (park.retained(ball).position.z > 100.0_f64))
                || (((i) as f64 == 2.0_f64) && (park.retained(ball).position.y > 100.0_f64)))
            {
                break; // original early loop exit
            }
            if ((i) as f64 == 0.0_f64) {
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:782
                assert!(
                    (park.retained(ball).position.z
                        + (0.9_f64 * f64::from(park.retained(ball).radius)))
                        < 0.0_f64
                );
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:783
                assert!(
                    python_almost_equal(park.retained(ball).velocity.y, 0.0_f64, 2) == Some(true)
                );
            }
            if ((i) as f64 == 1.0_f64) {
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:785
                assert!(
                    (park.retained(ball).position.y
                        + (0.9_f64 * f64::from(park.retained(ball).radius)))
                        < 0.0_f64
                );
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:786
                assert!(
                    python_almost_equal(park.retained(ball).velocity.x, 0.0_f64, 2) == Some(true)
                );
            }
            if ((i) as f64 == 2.0_f64) {
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:788
                assert!(
                    (park.retained(ball).position.x
                        + (0.9_f64 * f64::from(park.retained(ball).radius)))
                        < 0.0_f64
                );
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:789
                assert!(
                    python_almost_equal(park.retained(ball).velocity.z, 0.0_f64, 2) == Some(true)
                );
            }
            iteration_70 += 1;
        }
        iteration_50 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_from_back
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_from_back() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: -(10.0_f64),
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_28 = 0_i64;
    while iteration_28 < (20.0_f64) as i64 {
        let i = iteration_28;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_28 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_36 = 0_i64;
    while iteration_36 < (coordinates).len() as i64 {
        let i = iteration_36;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 50.0_f64,
                y: 50.0_f64,
                z: 0.0_f64,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:742
        let _diagnostic_arguments = (distance, f64::from(park.retained(ball).radius)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!(((1.11_f64 * distance) > f64::from(park.retained(ball).radius)));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:744
        assert!(python_almost_equal(point_at(&coordinates, i).x, 50.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:745
        assert!(python_almost_equal(point_at(&coordinates, i).y, 50.0_f64, 4) == Some(true));
        iteration_36 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_from_bottom
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_from_bottom() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 50.0_f64,
            y: -(50.0_f64),
            z: 50.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            (-0.7853981633974483_f64).sin(),
            0.0_f64,
            0.0_f64,
            (-0.7853981633974483_f64).cos(),
        ],
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 50.0_f64,
                y: 0.0_f64,
                z: 50.0_f64,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:706
        let _diagnostic_arguments = (distance, f64::from(park.retained(ball).radius)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!(((1.11_f64 * distance) > f64::from(park.retained(ball).radius)));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:708
        assert!(python_almost_equal(point_at(&coordinates, i).x, 50.0_f64, 3) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:709
        assert!(python_almost_equal(point_at(&coordinates, i).z, 50.0_f64, 3) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_from_front
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_from_front() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 110.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(ball, [1.0_f64, 0.0_f64, 0.0_f64, 0.0_f64]);
    park.goto_point(
        ball,
        Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (100.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 50.0_f64,
                y: 50.0_f64,
                z: 100.0_f64,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:729
        let _diagnostic_arguments = (distance, f64::from(park.retained(ball).radius)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!(((1.11_f64 * distance) > f64::from(park.retained(ball).radius)));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:731
        assert!(python_almost_equal(point_at(&coordinates, i).x, 50.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:732
        assert!(python_almost_equal(point_at(&coordinates, i).y, 50.0_f64, 4) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_from_left
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_from_left() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: -(10.0_f64),
            y: 50.0_f64,
            z: 50.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            0.0_f64,
            (0.5_f64 * (2.0_f64).sqrt()),
            0.0_f64,
            (0.5_f64 * (2.0_f64).sqrt()),
        ],
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 0.0_f64,
                y: 50.0_f64,
                z: 50.0_f64,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:636
        let _diagnostic_arguments = (distance, f64::from(park.retained(ball).radius)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!(((1.11_f64 * distance) > f64::from(park.retained(ball).radius)));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:638
        assert!(python_almost_equal(point_at(&coordinates, i).y, 50.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:639
        assert!(python_almost_equal(point_at(&coordinates, i).z, 50.0_f64, 4) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_from_right
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_from_right() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 110.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            0.0_f64,
            (-(0.5_f64) * (2.0_f64).sqrt()),
            0.0_f64,
            (0.5_f64 * (2.0_f64).sqrt()),
        ],
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 100.0_f64,
                y: 50.0_f64,
                z: 50.0_f64,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:660
        let _diagnostic_arguments = (distance, f64::from(park.retained(ball).radius)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!(((1.11_f64 * distance) > f64::from(park.retained(ball).radius)));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:662
        assert!(python_almost_equal(point_at(&coordinates, i).y, 50.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:663
        assert!(python_almost_equal(point_at(&coordinates, i).z, 50.0_f64, 4) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_collision_from_top
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_collision_from_top() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 50.0_f64,
            y: 110.0_f64,
            z: 50.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            (0.7853981633974483_f64).sin(),
            0.0_f64,
            0.0_f64,
            (0.7853981633974483_f64).cos(),
        ],
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 50.0_f64,
            y: 50.0_f64,
            z: 50.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 50.0_f64,
                y: 100.0_f64,
                z: 50.0_f64,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:683
        let _diagnostic_arguments = (distance, f64::from(park.retained(ball).radius)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!(((1.11_f64 * distance) > f64::from(park.retained(ball).radius)));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:685
        assert!(python_almost_equal(point_at(&coordinates, i).x, 50.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:686
        assert!(python_almost_equal(point_at(&coordinates, i).z, 50.0_f64, 2) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_starting_within_box
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_starting_within_box() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 20.0_f64,
            y: 60.0_f64,
            z: 30.0_f64,
        },
        velocity: Vec3 {
            x: -(10.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_26 = (0.01_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_26,
        OriginalBallAttribute::Agility,
    );
    let mut iteration_28 = 0_i64;
    while iteration_28 < (100.0_f64) as i64 {
        let i = iteration_28;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:798
        assert!(park.retained(ball).velocity.x < 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:799
        assert!(python_almost_equal(park.retained(ball).velocity.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:800
        assert!(python_almost_equal(park.retained(ball).velocity.z, 0.0_f64, 7) == Some(true));
        iteration_28 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:803
    assert!(
        (park.retained(ball).position.x + (0.9_f64 * f64::from(park.retained(ball).radius)))
            < 0.0_f64
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:805
    assert!(python_almost_equal(park.retained(ball).velocity.x, 0.0_f64, 3) == Some(true));
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeBoxCollision::test_two_aligning_boxes
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeBoxCollision__test_two_aligning_boxes() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_boxes).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_box = child_access_5.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 50.0_f64,
            z: -(10.0_f64),
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 1000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_26 = (1000.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.5_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: -(100.0_f64),
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 100.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    park.goto_direction(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 1.0_f64,
        },
    );
    let mut iteration_34 = 0_i64;
    while iteration_34 < (100.0_f64) as i64 {
        let i = iteration_34;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:824
        assert!(python_almost_equal(park.retained(ball).velocity.x, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:825
        assert!(python_almost_equal(park.retained(ball).velocity.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:827
        assert!(
            park.retained(ball).position.z < (-(0.8_f64) * f64::from(park.retained(ball).radius))
        );
        iteration_34 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_ball_starting_within_cylinder
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_ball_starting_within_cylinder() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 160.0_f64,
            y: 95.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut iteration_26 = 0_i64;
    while iteration_26 < (20.0_f64) as i64 {
        let i = iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:551
        assert!(park.retained(ball).velocity.y < 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:552
        assert!(python_almost_equal(park.retained(ball).velocity.x, 0.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:553
        assert!(python_almost_equal(park.retained(ball).velocity.z, 0.0_f64, 2) == Some(true));
        iteration_26 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:556
    assert!(
        (park.retained(ball).position.y
            + (0.9_f64
                * (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius))))
            < fixture_capsule.b.y
    );
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_ball_starting_within_sphere
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_ball_starting_within_sphere() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 95.0_f64,
            y: 100.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut iteration_26 = 0_i64;
    while iteration_26 < (20.0_f64) as i64 {
        let i = iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:565
        assert!(park.retained(ball).velocity.x < 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:566
        assert!(python_almost_equal(park.retained(ball).velocity.y, 0.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:567
        assert!(python_almost_equal(park.retained(ball).velocity.z, 0.0_f64, 2) == Some(true));
        iteration_26 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:570
    assert!(
        (park.retained(ball).position.x
            + (0.9_f64
                * (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius))))
            < fixture_capsule.a.x
    );
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_ball_with_miniballs_multi_collisions
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_ball_with_miniballs_multi_collisions() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    // Original unconditional return ends this test body. Its later source assertions are dead Python code; teardown still runs.
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_at_angle
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_collision_at_angle() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 170.0_f64,
            y: 80.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: -(6.7_f64),
            y: 6.7_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_26 = (0.3_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_26,
        OriginalBallAttribute::Agility,
    );
    park.goto_direction(
        ball,
        Vec3 {
            x: -(1.0_f64),
            y: 1.0_f64,
            z: 0.0_f64,
        },
    );
    park.set_velocity(
        ball,
        Vec3 {
            x: -(6.7_f64),
            y: 6.7_f64,
            z: 0.0_f64,
        },
    );
    let mut iteration_30 = 0_i64;
    while iteration_30 < (10.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:539
        assert!(
            (park.retained(ball).position.y
                + (0.9_f64
                    * (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius))))
                < fixture_capsule.b.y
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:542
        assert!(python_almost_equal(park.retained(ball).position.z, 0.0_f64, 1) == Some(true));
        iteration_30 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_from_bottom
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_collision_from_bottom() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 150.0_f64,
            y: 80.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(ball, [-(1.0_f64), 0.0_f64, 0.0_f64, 1.0_f64]);
    park.goto_direction(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 1.0_f64,
            z: 0.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 150.0_f64,
                y: fixture_capsule.b.y,
                z: fixture_capsule.b.z,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:503
        assert!(
            ((1.11_f64 * distance)
                > (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius)))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:505
        assert!(python_almost_equal(point_at(&coordinates, i).x, 150.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:506
        assert!(python_almost_equal(point_at(&coordinates, i).z, 0.0_f64, 2) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_from_left
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_collision_from_left() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 80.0_f64,
            y: 100.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 300.0_f64,
            y: 100.0_f64,
            z: 0.0_f64,
        },
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            0.0_f64,
            (0.5_f64 * (2.0_f64).sqrt()),
            0.0_f64,
            (0.5_f64 * (2.0_f64).sqrt()),
        ],
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: fixture_capsule.a.x,
                y: fixture_capsule.a.y,
                z: fixture_capsule.a.z,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:434
        assert!(
            ((1.11_f64 * distance)
                > (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius)))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:436
        assert!(python_almost_equal(point_at(&coordinates, i).y, 100.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:437
        assert!(python_almost_equal(point_at(&coordinates, i).z, 0.0_f64, 2) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_from_right
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_collision_from_right() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 220.0_f64,
            y: 100.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            0.0_f64,
            (-(0.7853981633974483_f64)).sin(),
            0.0_f64,
            (-(0.7853981633974483_f64)).cos(),
        ],
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 100.0_f64,
            z: 0.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: fixture_capsule.b.x,
                y: fixture_capsule.b.y,
                z: fixture_capsule.b.z,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:457
        assert!(
            ((1.11_f64 * distance)
                > (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius)))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:459
        assert!(python_almost_equal(point_at(&coordinates, i).y, 100.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:460
        assert!(python_almost_equal(point_at(&coordinates, i).z, 0.0_f64, 2) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_from_top
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_collision_from_top() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 150.0_f64,
            y: 120.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_14 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_14,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_16 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_16,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_18 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_18,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_20 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_20,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_22 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_22,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_24 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_24,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(
        ball,
        [
            (0.7853981633974483_f64).sin(),
            0.0_f64,
            0.0_f64,
            (0.7853981633974483_f64).cos(),
        ],
    );
    park.goto_point(
        ball,
        Vec3 {
            x: 150.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_30 = 0_i64;
    while iteration_30 < (20.0_f64) as i64 {
        let i = iteration_30;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_30 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_38 = 0_i64;
    while iteration_38 < (coordinates).len() as i64 {
        let i = iteration_38;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: 150.0_f64,
                y: fixture_capsule.b.y,
                z: fixture_capsule.b.z,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:480
        assert!(
            ((1.11_f64 * distance)
                > (f64::from(park.retained(ball).radius) + f64::from(fixture_capsule.radius)))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:482
        assert!(python_almost_equal(point_at(&coordinates, i).x, 150.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:483
        assert!(python_almost_equal(point_at(&coordinates, i).z, 0.0_f64, 2) == Some(true));
        iteration_38 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCapsuleCollision::test_collision_with_zero_length_capsule
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCapsuleCollision__test_collision_with_zero_length_capsule() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 200.0_f64,
                y: 100.0_f64,
                z: 0.0_f64
            },
            (10.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_5 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_5.is_some()); // original list index must exist
    let fixture_capsule = child_access_5.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        radius: (500.0_f64) as f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 100.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            (2.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_11 = (park.retained(parentBall).mini_capsules).get(0);
    assert!(child_access_11.is_some()); // original list index must exist
    let capsule = child_access_11.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 100.0_f64,
            y: 0.0_f64,
            z: -(10.0_f64),
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_20 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_20,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_22 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_22,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_24 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_24,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_26 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_26,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_28 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_28,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_30 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_30,
        OriginalBallAttribute::SpeedFraction,
    );
    park.goto_direction(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 1.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_34 = 0_i64;
    while iteration_34 < (20.0_f64) as i64 {
        let i = iteration_34;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_34 += 1;
    }
    let coordinates = coordinates;
    let mut iteration_42 = 0_i64;
    while iteration_42 < (coordinates).len() as i64 {
        let i = iteration_42;
        let distance = distance_between_points(
            point_at(&coordinates, i),
            Vec3 {
                x: capsule.b.x,
                y: capsule.b.y,
                z: capsule.b.z,
            },
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:520
        assert!(
            (1.11_f64 * distance)
                > (f64::from(park.retained(ball).radius) + f64::from(capsule.radius))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:522
        assert!(python_almost_equal(point_at(&coordinates, i).x, 100.0_f64, 4) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:523
        assert!(python_almost_equal(point_at(&coordinates, i).y, 0.0_f64, 2) == Some(true));
        iteration_42 += 1;
    }
    park.iterative_collision = false; // original Reset occurs before base teardown
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_ball_within_static_ball
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_ball_within_static_ball() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 8.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 2.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let asteroid = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(asteroid) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(asteroid, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        asteroid,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        asteroid,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        asteroid,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        asteroid,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        asteroid,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        asteroid,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_38 = (10.0_f64) as f32;
    park.write_attribute_notified(
        asteroid,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    park.set_free(asteroid, false);
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_42 = 0_i64;
    while iteration_42 < (10.0_f64) as i64 {
        let i = iteration_42;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z
        })); // original complete point/velocity trace
        iteration_42 += 1;
    }
    let coordinates = coordinates;
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:121
    assert!(point_at(&coordinates, (-(1.0_f64)) as i64).z > 12.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:122
    assert!(
        python_almost_equal(point_at(&coordinates, (-(1.0_f64)) as i64).x, 0.0_f64, 4)
            == Some(true)
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:123
    assert!(
        python_almost_equal(point_at(&coordinates, (-(1.0_f64)) as i64).y, 0.0_f64, 4)
            == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_balls_in_warp_should_not_collide
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_balls_in_warp_should_not_collide() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: -(40.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 40.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: -(10.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_38 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_40 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_40,
        OriginalBallAttribute::Radius,
    );
    park.warp_to(
        ball_a,
        Vec3 {
            x: 10000000.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        0.0_f64,
        (1000.0_f64) as i32,
    );
    park.warp_to(
        ball_b,
        Vec3 {
            x: -(10000000.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        0.0_f64,
        (1000.0_f64) as i32,
    );
    let mut iteration_44 = 0_i64;
    while iteration_44 < (80.0_f64) as i64 {
        let i = iteration_44;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        iteration_44 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:283
    assert!(park.retained(ball_a).position.x > park.retained(ball_b).position.x);
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:284
    assert!(park.retained(ball_a).position.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:285
    assert!(park.retained(ball_a).position.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:286
    assert!(park.retained(ball_b).position.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:287
    assert!(park.retained(ball_b).position.z == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_balls_intersecting_with_goto
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_balls_intersecting_with_goto() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 3.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: -(3.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_38 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_40 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_40,
        OriginalBallAttribute::Radius,
    );
    park.goto_direction(
        ball_a,
        Vec3 {
            x: 1.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    park.goto_direction(
        ball_b,
        Vec3 {
            x: -(1.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    let mut coordinates_a = BoundedList::<Vec3, 200>::new();
    let mut coordinates_b = BoundedList::<Vec3, 200>::new();
    let mut vel_a = BoundedList::<Vec3, 200>::new();
    let mut vel_b = BoundedList::<Vec3, 200>::new();
    let mut iteration_48 = 0_i64;
    while iteration_48 < (20.0_f64) as i64 {
        let i = iteration_48;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates_a.push(Vec3 {
            x: park.retained(ball_a).position.x,
            y: park.retained(ball_a).position.y,
            z: park.retained(ball_a).position.z
        })); // original complete point/velocity trace
        assert!(coordinates_b.push(Vec3 {
            x: park.retained(ball_b).position.x,
            y: park.retained(ball_b).position.y,
            z: park.retained(ball_b).position.z
        })); // original complete point/velocity trace
        assert!(vel_a.push(Vec3 {
            x: park.retained(ball_a).velocity.x,
            y: park.retained(ball_a).velocity.y,
            z: park.retained(ball_a).velocity.z
        })); // original complete point/velocity trace
        assert!(vel_b.push(Vec3 {
            x: park.retained(ball_b).velocity.x,
            y: park.retained(ball_b).velocity.y,
            z: park.retained(ball_b).velocity.z
        })); // original complete point/velocity trace
        iteration_48 += 1;
    }
    let mut distances = BoundedList::<f64, 200>::new();
    let mut comprehension_59 = 0_i64;
    while comprehension_59 < (coordinates_a).len() as i64 {
        let i = comprehension_59;
        assert!(distances.push(distance_between_points(
            point_at(&coordinates_b, i),
            point_at(&coordinates_a, i)
        )));
        comprehension_59 += 1;
    }
    let mut inside = true;
    let mut iteration_66 = 0_i64;
    while iteration_66 < (coordinates_a).len() as i64 {
        let i = iteration_66;
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:152
        assert!(python_almost_equal(point_at(&coordinates_a, i).y, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:153
        assert!(python_almost_equal(point_at(&coordinates_a, i).z, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:154
        assert!(python_almost_equal(point_at(&coordinates_b, i).y, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:155
        assert!(python_almost_equal(point_at(&coordinates_b, i).z, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:156
        assert!(python_almost_equal(point_at(&vel_a, i).y, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:157
        assert!(python_almost_equal(point_at(&vel_a, i).z, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:158
        assert!(python_almost_equal(point_at(&vel_b, i).y, 0.0_f64, 1) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:159
        assert!(python_almost_equal(point_at(&vel_b, i).z, 0.0_f64, 1) == Some(true));
        if (float_at(&distances, i)
            > (f64::from(park.retained(ball_a).radius) + f64::from(park.retained(ball_b).radius)))
        {
            inside = false;
        }
        if inside {
            // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:167
            assert!(point_at(&vel_a, i).x < 0.0_f64);
            // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:168
            assert!(point_at(&vel_b, i).x > 0.0_f64);
            if ((i) as f64 > 0.0_f64) {
                // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:170
                assert!(
                    float_at(&distances, i) > float_at(&distances, ((i) as f64 - 1.0_f64) as i64)
                );
            }
        } else {
            if (((i) as f64 + 1.0_f64) < ((distances).len() as i64) as f64) {
                if ((float_at(&distances, i) + (point_at(&vel_b, i).x - point_at(&vel_a, i).x))
                    > (f64::from(park.retained(ball_a).radius)
                        + f64::from(park.retained(ball_b).radius)))
                {
                    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:174
                    assert!(
                        point_at(&vel_b, ((i) as f64 + 1.0_f64) as i64).x < point_at(&vel_b, i).x
                    );
                    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:175
                    assert!(
                        point_at(&vel_a, ((i) as f64 + 1.0_f64) as i64).x > point_at(&vel_a, i).x
                    );
                } else {
                    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:177
                    assert!(
                        point_at(&vel_b, ((i) as f64 + 1.0_f64) as i64).x > point_at(&vel_b, i).x
                    );
                    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:178
                    assert!(
                        point_at(&vel_a, ((i) as f64 + 1.0_f64) as i64).x < point_at(&vel_a, i).x
                    );
                }
            }
        }
        iteration_66 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:181
    assert!(!(inside));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_balls_with_mini_balls_are_collision_targets
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_balls_with_mini_balls_are_collision_targets() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(4.0_f64),
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 4.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_38 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_40 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_40,
        OriginalBallAttribute::Radius,
    );
    park.set_free(ball_b, (0.0_f64) != 0.0);
    assert!(
        park.add_mini_ball(
            ball_b,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 10.0_f64
            },
            (2.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    park.goto_direction(
        ball_a,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 1.0_f64,
        },
    );
    let mut coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_46 = 0_i64;
    while iteration_46 < (50.0_f64) as i64 {
        let i = iteration_46;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 {
            x: park.retained(ball_a).position.x,
            y: park.retained(ball_a).position.y,
            z: park.retained(ball_a).position.z
        })); // original complete point/velocity trace
        iteration_46 += 1;
    }
    let a_coords = coordinates;
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:321
    assert!(python_almost_equal(park.retained(ball_b).position.x, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:322
    assert!(python_almost_equal(park.retained(ball_b).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:323
    assert!(python_almost_equal(park.retained(ball_b).position.z, 4.0_f64, 7) == Some(true));
    let mut point_iteration_60 = 0_usize;
    while point_iteration_60 < a_coords.len() {
        let point_access_60 = a_coords.get(point_iteration_60);
        assert!(point_access_60.is_some()); // original complete ordered list traversal
        let coord = point_access_60.unwrap_or(Vec3::ZERO);
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:327
        assert!(python_almost_equal(coord.x, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:328
        assert!(python_almost_equal(coord.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:329
        assert!(coord.z < 0.5_f64);
        point_iteration_60 = point_iteration_60.saturating_add(1);
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_collision_massive_with_small
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_collision_massive_with_small() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let small = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(small) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(small, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 1000000.0_f64;
    park.write_attribute_notified(
        small,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        small,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let large = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 30.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: -(20.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(large) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(large, true);
    let attribute_value_26 = (20.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 1000000000.0_f64;
    park.write_attribute_notified(
        large,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        large,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_38 = (20.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_40 = (1.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.speed_fraction = attribute_value_40,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_42 = (3.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.agility = attribute_value_42,
        OriginalBallAttribute::Agility,
    );
    park.goto_direction(
        large,
        Vec3 {
            x: -(1.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    let mut iteration_45 = 0_i64;
    while iteration_45 < (20.0_f64) as i64 {
        let i = iteration_45;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:225
        assert!(
            distance_between_points(
                Vec3 {
                    x: park.retained(small).position.x,
                    y: park.retained(small).position.y,
                    z: park.retained(small).position.z
                },
                Vec3 {
                    x: park.retained(large).position.x,
                    y: park.retained(large).position.y,
                    z: park.retained(large).position.z
                }
            ) >= (0.9_f64
                * (f64::from(park.retained(large).radius)
                    + f64::from(park.retained(small).radius)))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:229
        assert!(park.retained(large).velocity.x < -(19.0_f64));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:232
        assert!(python_almost_equal(park.retained(small).velocity.y, 0.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:233
        assert!(python_almost_equal(park.retained(small).velocity.z, 0.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:234
        assert!(python_almost_equal(park.retained(large).velocity.y, 0.0_f64, 2) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:235
        assert!(python_almost_equal(park.retained(large).velocity.z, 0.0_f64, 2) == Some(true));
        iteration_45 += 1;
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_collision_small_with_massive
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_collision_small_with_massive() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let small = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: -(2.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 20.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(small) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(small, true);
    let attribute_value_8 = (20.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 1000000.0_f64;
    park.write_attribute_notified(
        small,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        small,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_20 = (1.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.speed_fraction = attribute_value_20,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_22 = (1000.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.agility = attribute_value_22,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_24 = (2.0_f64) as f32;
    park.write_attribute_notified(
        small,
        |b| b.radius = attribute_value_24,
        OriginalBallAttribute::Radius,
    );
    let large = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 22.01_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(large) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(large, true);
    let attribute_value_32 = (10.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.max_velocity = attribute_value_32,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_34 = (0.9_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.agility = attribute_value_34,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_36 = 1000000000.0_f64;
    park.write_attribute_notified(
        large,
        |b| b.mass = attribute_value_36,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_38 = (2.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_40 = true;
    park.write_attribute_notified(
        large,
        |b| b.is_massive = attribute_value_40,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_42 = (0.95_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.speed_fraction = attribute_value_42,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_44 = (1.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.speed_fraction = attribute_value_44,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_46 = (20.0_f64) as f32;
    park.write_attribute_notified(
        large,
        |b| b.radius = attribute_value_46,
        OriginalBallAttribute::Radius,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(small, [0.0_f64, -(1.0_f64), 0.0_f64, 1.0_f64]);
    park.goto_direction(
        small,
        Vec3 {
            x: -(1.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:201
    assert!(python_almost_equal(park.retained(large).velocity.x, 0.04_f64, 2) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:202
    assert!(python_almost_equal(park.retained(large).velocity.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:203
    assert!(python_almost_equal(park.retained(large).velocity.z, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:205
    assert!(python_almost_equal(park.retained(small).velocity.x, -(19.96_f64), 1) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:206
    assert!(python_almost_equal(park.retained(small).velocity.y, 0.0_f64, 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:207
    assert!(python_almost_equal(park.retained(small).velocity.z, 0.0_f64, 4) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_correct_collision_order
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_correct_collision_order() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 4.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_c = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_c) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_c, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 1000000.0_f64;
    park.write_attribute_notified(
        ball_c,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_c,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(4.0_f64),
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 20.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_44 = (100.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_44,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_46 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_46,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_48 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_48,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_50 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_50,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_52 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_52,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_54 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_54,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_56 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_56,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_58 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_58,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_60 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.radius = attribute_value_60,
        OriginalBallAttribute::Radius,
    );
    park.set_free(ball_b, (0.0_f64) != 0.0);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:343
    assert!(python_almost_equal(park.retained(ball_c).position.x, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:344
    assert!(python_almost_equal(park.retained(ball_c).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:345
    assert!(python_almost_equal(park.retained(ball_c).position.z, 10.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:348
    assert!(park.retained(ball_b).position.z > park.retained(ball_a).position.z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_goto_collision
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_goto_collision() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let mut a_coordinates = BoundedList::<Vec3, 200>::new();
    let mut b_coordinates = BoundedList::<Vec3, 200>::new();
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_10 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_10,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_12 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_12,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_14 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_14,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_16 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_16,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_18 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_18,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_20 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_20,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_28 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_28,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_30 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_30,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_32 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_32,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_34 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_34,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_36 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_36,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_38 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_38,
        OriginalBallAttribute::SpeedFraction,
    );
    // Pinned Ballpark_Blue.cpp exposes SetBallRotation even with old orientation; execute the actual mutation.
    park.set_ball_rotation(ball_b, [0.0_f64, 1.0_f64, 0.0_f64, 0.0_f64]);
    park.goto_point(
        ball_a,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
    );
    park.goto_point(
        ball_b,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    let mut iteration_44 = 0_i64;
    while iteration_44 < (20.0_f64) as i64 {
        let i = iteration_44;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(a_coordinates.push(Vec3 {
            x: park.retained(ball_a).position.x,
            y: park.retained(ball_a).position.y,
            z: park.retained(ball_a).position.z
        })); // original complete point/velocity trace
        assert!(b_coordinates.push(Vec3 {
            x: park.retained(ball_b).position.x,
            y: park.retained(ball_b).position.y,
            z: park.retained(ball_b).position.z
        })); // original complete point/velocity trace
        iteration_44 += 1;
    }
    let mut iteration_52 = 0_i64;
    while iteration_52 < (b_coordinates).len() as i64 {
        let i = iteration_52;
        let distance =
            distance_between_points(point_at(&a_coordinates, i), point_at(&b_coordinates, i));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:264
        assert!(
            (1.11_f64 * distance)
                > (f64::from(park.retained(ball_a).radius)
                    + f64::from(park.retained(ball_b).radius))
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:266
        assert!(
            python_almost_equal(
                point_at(&a_coordinates, i).x,
                point_at(&a_coordinates, (0.0_f64) as i64).x,
                7
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:267
        assert!(
            python_almost_equal(
                point_at(&a_coordinates, i).y,
                point_at(&a_coordinates, (0.0_f64) as i64).y,
                7
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:268
        assert!(
            python_almost_equal(
                point_at(&b_coordinates, i).x,
                point_at(&b_coordinates, (0.0_f64) as i64).x,
                7
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:269
        assert!(
            python_almost_equal(
                point_at(&b_coordinates, i).y,
                point_at(&b_coordinates, (0.0_f64) as i64).y,
                7
            ) == Some(true)
        );
        iteration_52 += 1;
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_multiple_overlapping_balls
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_multiple_overlapping_balls() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 1.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: -(1.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_c = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 1.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_c) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_c, true);
    let attribute_value_44 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.max_velocity = attribute_value_44,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_46 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.agility = attribute_value_46,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_48 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_c,
        |b| b.mass = attribute_value_48,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_50 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.radius = attribute_value_50,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_52 = true;
    park.write_attribute_notified(
        ball_c,
        |b| b.is_massive = attribute_value_52,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_54 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.speed_fraction = attribute_value_54,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_d = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(1.0_f64),
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_d) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_d, true);
    let attribute_value_62 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.max_velocity = attribute_value_62,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_64 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.agility = attribute_value_64,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_66 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_d,
        |b| b.mass = attribute_value_66,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_68 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.radius = attribute_value_68,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_70 = true;
    park.write_attribute_notified(
        ball_d,
        |b| b.is_massive = attribute_value_70,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_72 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.speed_fraction = attribute_value_72,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut ac = BoundedList::<Vec3, 200>::new();
    let mut bc = BoundedList::<Vec3, 200>::new();
    let mut cc = BoundedList::<Vec3, 200>::new();
    let mut dc = BoundedList::<Vec3, 200>::new();
    let mut iteration_78 = 0_i64;
    while iteration_78 < (10.0_f64) as i64 {
        let i = iteration_78;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(ac.push(Vec3 {
            x: park.retained(ball_a).position.x,
            y: park.retained(ball_a).position.y,
            z: park.retained(ball_a).position.z
        })); // original complete point/velocity trace
        assert!(bc.push(Vec3 {
            x: park.retained(ball_b).position.x,
            y: park.retained(ball_b).position.y,
            z: park.retained(ball_b).position.z
        })); // original complete point/velocity trace
        assert!(cc.push(Vec3 {
            x: park.retained(ball_c).position.x,
            y: park.retained(ball_c).position.y,
            z: park.retained(ball_c).position.z
        })); // original complete point/velocity trace
        assert!(dc.push(Vec3 {
            x: park.retained(ball_d).position.x,
            y: park.retained(ball_d).position.y,
            z: park.retained(ball_d).position.z
        })); // original complete point/velocity trace
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:67
        assert!(python_almost_equal(park.retained(ball_a).velocity.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:68
        assert!(python_almost_equal(park.retained(ball_b).velocity.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:69
        assert!(python_almost_equal(park.retained(ball_c).velocity.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:70
        assert!(python_almost_equal(park.retained(ball_d).velocity.y, 0.0_f64, 7) == Some(true));
        iteration_78 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:73
    assert!(
        distance_between_points(
            point_at(&ac, (4.0_f64) as i64),
            point_at(&bc, (4.0_f64) as i64)
        ) > (f64::from(park.retained(ball_a).radius) * 2.0_f64)
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:74
    assert!(
        distance_between_points(
            point_at(&ac, (4.0_f64) as i64),
            point_at(&cc, (4.0_f64) as i64)
        ) > (f64::from(park.retained(ball_a).radius) * 2.0_f64)
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:75
    assert!(
        distance_between_points(
            point_at(&ac, (4.0_f64) as i64),
            point_at(&dc, (4.0_f64) as i64)
        ) > (f64::from(park.retained(ball_a).radius) * 2.0_f64)
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:76
    assert!(
        distance_between_points(
            point_at(&bc, (4.0_f64) as i64),
            point_at(&cc, (4.0_f64) as i64)
        ) > (f64::from(park.retained(ball_a).radius) * 2.0_f64)
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:77
    assert!(
        distance_between_points(
            point_at(&bc, (4.0_f64) as i64),
            point_at(&dc, (4.0_f64) as i64)
        ) > (f64::from(park.retained(ball_a).radius) * 2.0_f64)
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:78
    assert!(
        distance_between_points(
            point_at(&cc, (4.0_f64) as i64),
            point_at(&dc, (4.0_f64) as i64)
        ) > (f64::from(park.retained(ball_a).radius) * 2.0_f64)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_non_massive_balls_should_not_collide
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_non_massive_balls_should_not_collide() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(4.0_f64),
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 4.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let attribute_value_38 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_38,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_40 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_40,
        OriginalBallAttribute::Radius,
    );
    park.set_massive(ball_b, (0.0_f64) != 0.0);
    park.goto_direction(
        ball_a,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 1.0_f64,
        },
    );
    let mut iteration_44 = 0_i64;
    while iteration_44 < (80.0_f64) as i64 {
        let i = iteration_44;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        iteration_44 += 1;
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:300
    assert!(park.retained(ball_a).position.z > park.retained(ball_b).position.z);
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:301
    assert!(python_almost_equal(park.retained(ball_a).position.x, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:302
    assert!(python_almost_equal(park.retained(ball_a).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:303
    assert!(python_almost_equal(park.retained(ball_b).position.x, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:304
    assert!(python_almost_equal(park.retained(ball_b).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:305
    assert!(python_almost_equal(park.retained(ball_b).position.z, 4.0_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_stopped_balls_intersecting
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_stopped_balls_intersecting() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 1.9_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_26,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_28,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_30,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_32,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_34 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_34,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_36,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut a_coordinates = BoundedList::<Vec3, 200>::new();
    let mut b_coordinates = BoundedList::<Vec3, 200>::new();
    let mut iteration_40 = 0_i64;
    while iteration_40 < (10.0_f64) as i64 {
        let i = iteration_40;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(a_coordinates.push(Vec3 {
            x: park.retained(ball_a).position.x,
            y: park.retained(ball_a).position.y,
            z: park.retained(ball_a).position.z
        })); // original complete point/velocity trace
        assert!(b_coordinates.push(Vec3 {
            x: park.retained(ball_b).position.x,
            y: park.retained(ball_b).position.y,
            z: park.retained(ball_b).position.z
        })); // original complete point/velocity trace
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:93
        assert!(park.retained(ball_a).velocity.y > 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:94
        assert!(park.retained(ball_b).velocity.y < 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:95
        assert!(
            python_almost_equal(
                park.retained(ball_a).velocity.y,
                -(park.retained(ball_b).velocity.y),
                7
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:96
        assert!(python_almost_equal(park.retained(ball_a).velocity.x, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:97
        assert!(python_almost_equal(park.retained(ball_a).velocity.z, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:98
        assert!(python_almost_equal(park.retained(ball_b).velocity.x, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:99
        assert!(python_almost_equal(park.retained(ball_b).velocity.z, 0.0_f64, 7) == Some(true));
        iteration_40 += 1;
    }
    let distance = distance_between_points(
        point_at(&a_coordinates, (1.0_f64) as i64),
        point_at(&b_coordinates, (1.0_f64) as i64),
    );
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:104
    assert!(
        (distance
            > (f64::from(park.retained(ball_a).radius) + f64::from(park.retained(ball_b).radius)))
    );
    let mut iteration_65 = 0_i64;
    while iteration_65 < (a_coordinates).len() as i64 {
        let i = iteration_65;
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:107
        assert!(
            python_almost_equal(
                point_at(&a_coordinates, i).x,
                point_at(&a_coordinates, (0.0_f64) as i64).x,
                4
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:108
        assert!(
            python_almost_equal(
                point_at(&a_coordinates, i).z,
                point_at(&a_coordinates, (0.0_f64) as i64).z,
                4
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:109
        assert!(
            python_almost_equal(
                point_at(&b_coordinates, i).x,
                point_at(&b_coordinates, (0.0_f64) as i64).x,
                4
            ) == Some(true)
        );
        // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:110
        assert!(
            python_almost_equal(
                point_at(&b_coordinates, i).z,
                point_at(&b_coordinates, (0.0_f64) as i64).z,
                4
            ) == Some(true)
        );
        iteration_65 += 1;
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_substep_collision
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_substep_collision() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    park.friction = 1e-10_f64; // original raw double park field write
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 20.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_9 = (100.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_9,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_11,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_13,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_15,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_17 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_17,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_19,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_c = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_c) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_c, true);
    let attribute_value_27 = (100.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.max_velocity = attribute_value_27,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_29 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.agility = attribute_value_29,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_31 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_c,
        |b| b.mass = attribute_value_31,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_33 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.radius = attribute_value_33,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_35 = true;
    park.write_attribute_notified(
        ball_c,
        |b| b.is_massive = attribute_value_35,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_37 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.speed_fraction = attribute_value_37,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 5.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_45 = (100.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_45,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_47 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_47,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_49 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_49,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_51 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_51,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_53 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_53,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_55 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_55,
        OriginalBallAttribute::SpeedFraction,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:362
    assert!(python_almost_equal(park.retained(ball_a).velocity.z, 0.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:363
    assert!(python_almost_equal(park.retained(ball_a).position.z, 1.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:366
    assert!(python_almost_equal(park.retained(ball_b).velocity.z, 0.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:367
    assert!(python_almost_equal(park.retained(ball_b).position.z, 6.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:370
    assert!(python_almost_equal(park.retained(ball_c).velocity.z, 20.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:371
    assert!(python_almost_equal(park.retained(ball_c).position.z, 28.0_f64, 0) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_iterative_collision.py::TestIterativeCollision::test_substep_collision_static
// original-source-sha256: 024b1dd360c1d92562e175640559cd4d8284c9a10b8c87c8f372f53f2a69c831
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(177))]
#[cfg_attr(test, test)]
fn TestIterativeCollision__test_substep_collision_static() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id = 1_i64; // fresh helper-module counter; native suite order remains an environment obligation
    park.iterative_collision = true; // GetDefault/Apply source default: twenty passes, old orientation
    park.friction = 1e-10_f64; // original raw double park field write
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(14.0_f64),
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 50.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_9 = (100.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.max_velocity = attribute_value_9,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.agility = attribute_value_11,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_a,
        |b| b.mass = attribute_value_13,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.radius = attribute_value_15,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_17 = true;
    park.write_attribute_notified(
        ball_a,
        |b| b.is_massive = attribute_value_17,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_a,
        |b| b.speed_fraction = attribute_value_19,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: -(10.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 50.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_27 = (100.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.max_velocity = attribute_value_27,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_29 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.agility = attribute_value_29,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_31 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_b,
        |b| b.mass = attribute_value_31,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_33 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.radius = attribute_value_33,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_35 = true;
    park.write_attribute_notified(
        ball_b,
        |b| b.is_massive = attribute_value_35,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_37 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_b,
        |b| b.speed_fraction = attribute_value_37,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_c = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 20.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_c) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_c, true);
    let attribute_value_45 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.max_velocity = attribute_value_45,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_47 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.agility = attribute_value_47,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_49 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_c,
        |b| b.mass = attribute_value_49,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_51 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.radius = attribute_value_51,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_53 = true;
    park.write_attribute_notified(
        ball_c,
        |b| b.is_massive = attribute_value_53,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_55 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_c,
        |b| b.speed_fraction = attribute_value_55,
        OriginalBallAttribute::SpeedFraction,
    );
    let ball_d = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 20.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_d) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_d, true);
    let attribute_value_63 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.max_velocity = attribute_value_63,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_65 = (0.9_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.agility = attribute_value_65,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_67 = 13000000.0_f64;
    park.write_attribute_notified(
        ball_d,
        |b| b.mass = attribute_value_67,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_69 = (2.0_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.radius = attribute_value_69,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_71 = true;
    park.write_attribute_notified(
        ball_d,
        |b| b.is_massive = attribute_value_71,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_73 = (0.95_f64) as f32;
    park.write_attribute_notified(
        ball_d,
        |b| b.speed_fraction = attribute_value_73,
        OriginalBallAttribute::SpeedFraction,
    );
    park.set_free(ball_c, (0.0_f64) != 0.0);
    park.set_free(ball_d, (0.0_f64) != 0.0);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:390
    assert!(python_almost_equal(park.retained(ball_a).velocity.z, 0.0_f64, 2) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:391
    assert!(python_almost_equal(park.retained(ball_a).velocity.x, 0.0_f64, 2) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:392
    assert!(python_almost_equal(park.retained(ball_a).position.z, -(4.0_f64), 2) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:393
    assert!(python_almost_equal(park.retained(ball_b).velocity.z, 50.0_f64, 2) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_iterative_collision.py:394
    assert!(python_almost_equal(park.retained(ball_b).velocity.x, 50.0_f64, 2) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    park.iterative_collision = false; // original Reset occurs after base teardown
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original iterative collision scenario and teardown reached"
    );
}
