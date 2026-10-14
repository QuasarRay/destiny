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

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_mass
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_mass() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_mass(ball, 3.14_f64);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:10
    assert!(park.retained(ball).mass == 3.14_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_accuracy_is_zero_when_ball_targets_itself
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_accur__d9e5d6af73c8183700022d9d() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let (accuracy, returned_optimal_range, returned_tracking_speed) =
        park.accuracy(src, src, 1.0_f64, 1.0_f64, 1.0_f64, 1.0_f64, 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:189
    assert!(accuracy == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_accuracy_is_zero_when_distance_is_zero
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_accur__912ea17afc64b9395d4eaeee() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let fallOff = 1.0_f64;
    let optimalRange = 1.0_f64;
    let trackingSpeed = 1.0_f64;
    let signatureRadius = 1.0_f64;
    let optimalSignatureRadius = 1.0_f64;
    let (accuracy, returned_optimal_range, returned_tracking_speed) = park.accuracy(
        src,
        dst,
        optimalRange,
        fallOff,
        trackingSpeed,
        signatureRadius,
        optimalSignatureRadius,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:184
    assert!(accuracy == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_combination_parameters
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_combination_parameters() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let attribute_value_4 = (park.retained(dst).position.x + 2.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let fallOff = 0.95_f64;
    let optimalRange = 0.5_f64;
    let trackingSpeed = 0.6_f64;
    let signatureRadius = 0.2_f64;
    let optimalSignatureRadius = 0.7_f64;
    let (accuracy, returned_optimal_range, returned_tracking_speed) = park.accuracy(
        src,
        dst,
        optimalRange,
        fallOff,
        trackingSpeed,
        signatureRadius,
        optimalSignatureRadius,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:261
    assert!(python_almost_equal(accuracy, 0.17762729659881724_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_full_accuracy
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_full_accuracy() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let attribute_value_4 = (park.retained(dst).position.x + 1.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let fallOff = 1.0_f64;
    let optimalRange = 1.0_f64;
    let trackingSpeed = 1.0_f64;
    let signatureRadius = 1.0_f64;
    let optimalSignatureRadius = 1.0_f64;
    let (accuracy, returned_optimal_range, returned_tracking_speed) = park.accuracy(
        src,
        dst,
        optimalRange,
        fallOff,
        trackingSpeed,
        signatureRadius,
        optimalSignatureRadius,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:207
    assert!(accuracy == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_no_falloff
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_no_falloff() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let attribute_value_4 = (park.retained(dst).position.x + 2.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let fallOff = 1.0_f64;
    let optimalRange = 1.0_f64;
    let trackingSpeed = 1.0_f64;
    let signatureRadius = 1.0_f64;
    let optimalSignatureRadius = 1.0_f64;
    let (accuracy, returned_optimal_range, returned_tracking_speed) = park.accuracy(
        src,
        dst,
        optimalRange,
        fallOff,
        trackingSpeed,
        signatureRadius,
        optimalSignatureRadius,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:225
    assert!(accuracy == 0.5_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_returns_accuracy_and_tracking_speed_that_we_specify
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_retur__c78ed1e98d187e6ba1f77d87() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let fallOff = 1.0_f64;
    let optimalRange = 42.0_f64;
    let trackingSpeed = 3.14_f64;
    let signatureRadius = 1.0_f64;
    let optimalSignatureRadius = 1.0_f64;
    let (accuracy, returned_optimal_range, returned_tracking_speed) = park.accuracy(
        src,
        dst,
        optimalRange,
        fallOff,
        trackingSpeed,
        signatureRadius,
        optimalSignatureRadius,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:278
    assert!(returned_optimal_range == optimalRange);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:279
    assert!(returned_tracking_speed == trackingSpeed);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetAccuracy::test_some_falloff
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetAccuracy__test_some_falloff() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let attribute_value_4 = (park.retained(dst).position.x + 2.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let fallOff = 0.95_f64;
    let optimalRange = 1.0_f64;
    let trackingSpeed = 1.0_f64;
    let signatureRadius = 1.0_f64;
    let optimalSignatureRadius = 1.0_f64;
    let (accuracy, returned_optimal_range, returned_tracking_speed) = park.accuracy(
        src,
        dst,
        optimalRange,
        fallOff,
        trackingSpeed,
        signatureRadius,
        optimalSignatureRadius,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:243
    assert!(python_almost_equal(accuracy, 0.46392604883716854_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetCenterDist::test_center_distance_between_two_balls
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetCenterDist__test_center_distance_between_two_balls() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 100.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let result = park.center_distance(ball, other);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:321
    assert!(result == 100.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetCenterDist::test_center_distance_between_two_balls_in_the_same_place
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetCenterDist__test_cen__56fe3d4dceb262c1e9bf6aa8() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let result = park.center_distance(ball, other);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:315
    assert!(result == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetCurrentEgoPos::test_returns_default_initial_ego_value
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetCurrentEgoPos__test___ad222e044f33a7c5e47411fa() {
    let mut park = OriginalParkProjection::default();
    let ego_pos = park.current_ego_position_without_ego();
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:380
    assert!(ego_pos == (0.0_f64, 0.0_f64, 0.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetFollowers::test_returns_empty_list_when_there_are_no_followers
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetFollowers__test_retu__66f249b39aea14e81ac3b3df() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let followers = park.followers(ball);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:368
    assert!(followers == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetFollowers::test_returns_list_of_followers
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetFollowers__test_returns_list_of_followers() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.follow(src, dst, (1.0_f64) as f32, false);
    let followers = park.followers(dst);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:374
    assert!(followers == id_list(&[src]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetSurfaceDist::test_euclidean_distance
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetSurfaceDist__test_euclidean_distance() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 1.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let attribute_value_6 = (park.retained(other).position.y + 2.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.y = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.z + 3.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let result = park.surface_distance(ball, other);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:308
    assert!(python_almost_equal(result, 3.7416573867739413_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetSurfaceDist::test_surface_distance_between_two_balls_equals_the_distance_between_the_balls_minus_their_combined_radii
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetSurfaceDist__test_su__001e1cc2265b641e8abae70d() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (10.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (5.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (park.retained(other).position.x + 100.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let result = park.surface_distance(ball, other);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:300
    assert!(result == 85.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetSurfaceDist::test_surface_distance_between_two_balls_in_the_same_place_with_no_radius_is_zero
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetSurfaceDist__test_su__56b9245a7e85ba1146616721() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let result = park.surface_distance(ball, other);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:286
    assert!(result == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetSurfaceDist::test_surface_distance_between_two_balls_with_no_radius_equals_the_distance_between_the_balls
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetSurfaceDist__test_su__4609e7b60e4e969a265f9fc2() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 100.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let result = park.surface_distance(ball, other);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:292
    assert!(result == 100.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetBallFormation::test_followers_whom_we_no_longer_have_room_for_get_dropped
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetBallFormation__test___54c2a023619ec83a0a2a8d10() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.load_original_formations();
    park.set_formation(dst, (1.0_f64) as i8);
    park.formation_follow(src, dst, (1.0_f64) as i8);
    park.set_formation(dst, (2.0_f64) as i8);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:402
    assert!(park.retained(src).mode == OriginalMode::Stop);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetBallFormation::test_formation_out_of_range_does_not_get_set
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetBallFormation__test___cb32eb66747446c02e0a1d63() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.load_original_formations();
    park.set_formation(ball, (3_i64) as i8);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:394
    assert!((i64::from(park.retained(ball).formation_id as u8)) as f64 == 255.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetBallFormation::test_valid_formation_gets_set
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetBallFormation__test_valid_formation_gets_set() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.load_original_formations();
    park.set_formation(ball, (0.0_f64) as i8);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:388
    assert!((i64::from(park.retained(ball).formation_id as u8)) as f64 == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_agility
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_agility() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_agility_nonpositive_time_factor(ball, (3.14_f64) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:86
    assert!(python_almost_equal(f64::from(park.retained(ball).agility), 3.14_f64, 6) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_free
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_free() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_free(ball, true);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:71
    assert!(park.retained(ball).is_free == true);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_global
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_global() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_global(ball, true);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:81
    assert!(park.retained(ball).is_global == true);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_harmonic_make_forcefield
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_harmonic_make_forcefield() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let harmonic = 1.0_f64;
    let corportationID = 2.0_f64;
    let allianceID = 3.0_f64;
    let force_field = true;
    park.set_harmonic(
        ball,
        (harmonic) as i64,
        (corportationID) as i32,
        (allianceID) as i32,
        force_field,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:153
    assert!((park.retained(ball).harmonic) as f64 == harmonic);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:154
    assert!((i64::from(park.retained(ball).corporation_id)) as f64 == corportationID);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:155
    assert!((i64::from(park.retained(ball).alliance_id)) as f64 == allianceID);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:156
    assert!(park.retained(ball).mode == OriginalMode::Field);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_harmonic_not_forcefield
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_harmonic_not_forcefield() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let harmonic = 1.0_f64;
    let corportationID = 2.0_f64;
    let allianceID = 3.0_f64;
    let force_field = false;
    park.set_harmonic(
        ball,
        (harmonic) as i64,
        (corportationID) as i32,
        (allianceID) as i32,
        force_field,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:141
    assert!((park.retained(ball).harmonic) as f64 == harmonic);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:142
    assert!((i64::from(park.retained(ball).corporation_id)) as f64 == corportationID);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:143
    assert!((i64::from(park.retained(ball).alliance_id)) as f64 == allianceID);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:144
    assert!(park.retained(ball).mode == OriginalMode::Stop);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_harmonic_remove_forcefield
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_harmonic_remove_forcefield() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let harmonic = 1.0_f64;
    let corportationID = 2.0_f64;
    let allianceID = 3.0_f64;
    park.set_harmonic(
        ball,
        (harmonic) as i64,
        (corportationID) as i32,
        (allianceID) as i32,
        true,
    );
    park.set_harmonic(
        ball,
        (harmonic) as i64,
        (corportationID) as i32,
        (allianceID) as i32,
        false,
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:165
    assert!(park.retained(ball).mode == OriginalMode::Stop);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_interactive
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_interactive() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_interactive(ball, true);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:96
    assert!((park.retained(ball).is_interactive));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_massive
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_massive() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_massive(ball, true);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:76
    assert!(park.retained(ball).is_massive == true);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_position
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_position() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    park.set_position(ball, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:41
    assert!(python_almost_equal(park.retained(ball).position.x, x, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:42
    assert!(python_almost_equal(park.retained(ball).position.y, y, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:43
    assert!(python_almost_equal(park.retained(ball).position.z, z, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_radius
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_radius() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_radius(ball, (3.14_f64) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:20
    assert!(python_almost_equal(f64::from(park.retained(ball).radius), 3.14_f64, 6) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_ball_velocity
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_ball_velocity() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    park.set_velocity(ball, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:49
    assert!(python_almost_equal(park.retained(ball).velocity.x, x, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:50
    assert!(python_almost_equal(park.retained(ball).velocity.y, y, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:51
    assert!(python_almost_equal(park.retained(ball).velocity.z, z, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_max_speed
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_max_speed() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_max_speed(ball, (3.14_f64) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:30
    assert!(
        python_almost_equal(f64::from(park.retained(ball).max_velocity), 3.14_f64, 6) == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_can_set_speed_fraction
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_can_set_speed_fraction() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_speed_fraction(ball, (0.2_f64) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:56
    assert!(
        python_almost_equal(f64::from(park.retained(ball).speed_fraction), 0.2_f64, 7)
            == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_set_ball_rigid
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_set_ball_rigid() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_rigid(ball);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:132
    assert!(park.retained(ball).mode == OriginalMode::Rigid);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_set_ball_troll_no_delay
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_set_ball_troll_no_delay() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_troll(ball, (0.0_f64) as i32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:120
    assert!(park.retained(ball).mode == OriginalMode::Troll);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_set_ball_troll_with_delay
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_set_ball_troll_with_delay() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let delay = 3.0_f64;
    park.set_troll(ball, (delay) as i32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:126
    assert!(park.retained(ball).mode == OriginalMode::Troll);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:127
    assert!((park.retained(ball).effect_stamp) as f64 == delay);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_set_boid_false
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_set_boid_false() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_boid(ball, true);
    park.set_boid(ball, false);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:115
    assert!(park.retained(ball).mode == OriginalMode::Stop);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_set_boid_true
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_set_boid_true() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_boid(ball, true);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:109
    assert!(park.retained(ball).mode == OriginalMode::Boid);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_agility_to_negative_value_is_ineffective
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_a__3172dec0c88aa893b8ae71bb() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_agility_nonpositive_time_factor(ball, (-(1.0_f64)) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:91
    assert!(python_almost_equal(f64::from(park.retained(ball).agility), 1.0_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_ball_mass_to_a_negative_value_is_ineffective
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_b__7ec06b31aaba4422b15009c3() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.reject_nonpositive_mass(ball, -(1.0_f64));
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:15
    assert!(park.retained(ball).mass == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_ball_radius_to_negative_value_is_ineffective
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_b__7ebf0ead7f49feeed080eaff() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_radius(ball, (-(1.0_f64)) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:25
    assert!(f64::from(park.retained(ball).radius) == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_ball_velocity_updates_yaw_pitch_and_roll
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_b__4ab618e1dc316f7803a6bc9d() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    park.set_velocity(ball, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:102
    assert!(python_almost_equal(park.retained(ball).yaw, 0.3217505543966422_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:103
    assert!(
        python_almost_equal(park.retained(ball).pitch, -(0.5639426413606289_f64), 7) == Some(true)
    );
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:104
    assert!(python_almost_equal(park.retained(ball).roll, 0.0_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_max_speed_to_negative_value_is_ineffective
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_m__07398ab01cba6a2340ed33ec() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_max_speed(ball, (-(1.0_f64)) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:35
    assert!(f64::from(park.retained(ball).max_velocity) == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_speed_fraction_clamps_lowe_bound_at_zero
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_s__94c5880a4ff7b108db700899() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_speed_fraction(ball, (-(2.0_f64)) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:66
    assert!(
        python_almost_equal(f64::from(park.retained(ball).speed_fraction), 0.0_f64, 7)
            == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestSetters::test_setting_speed_fraction_clamps_upper_bound_at_one
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestSetters__test_setting_s__9b6486626dcd6e84c07dadc2() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.set_speed_fraction(ball, (2.0_f64) as f32);
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:61
    assert!(
        python_almost_equal(f64::from(park.retained(ball).speed_fraction), 1.0_f64, 7)
            == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBallpark::test_add_ball_adds_ball_to_park
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestBallpark__test_add_ball_adds_ball_to_park() {
    let mut park = OriginalParkProjection::default();
    let ignored_ball_0 = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ignored_ball_0) && !park.scope_error);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:17
    assert!((park.count() as i64) as f64 == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBallpark::test_remove_ball_from_park
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestBallpark__test_remove_ball_from_park() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.remove_ball(ball, (0.0_f64) as i32);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:22
    assert!((park.count() as i64) as f64 == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestRemoveBall::test_remove_ball_no_delay
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestRemoveBall__test_remove_ball_no_delay() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.remove_ball(ball, (0.0_f64) as i32);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:29
    assert!((park.retained(ball).is_moribund));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestRemoveBall::test_remove_ball_with_delay
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestRemoveBall__test_remove_ball_with_delay() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.remove_ball(ball, (100.0_f64) as i32);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:34
    assert!((park.retained(ball).is_moribund));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_direction_vector_gets_normalized
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_direction_vector_gets_normalized() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    park.goto_direction(
        first,
        Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
    );
    park.goto_direction(
        second,
        Vec3 {
            x: 5.0_f64,
            y: 5.0_f64,
            z: 5.0_f64,
        },
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:184
    assert!(
        python_almost_equal(
            (park.retained(first).goto.x / 1e+16_f64),
            (park.retained(second).goto.x / 1e+16_f64),
            7
        ) == Some(true)
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:185
    assert!(
        python_almost_equal(
            (park.retained(first).goto.y / 1e+16_f64),
            (park.retained(second).goto.y / 1e+16_f64),
            7
        ) == Some(true)
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:186
    assert!(
        python_almost_equal(
            (park.retained(first).goto.z / 1e+16_f64),
            (park.retained(second).goto.z / 1e+16_f64),
            7
        ) == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_go_backward
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_go_backward() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = -(1.0_f64);
    park.goto_direction(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:166
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:167
    assert!(park.retained(src).goto.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:168
    assert!(park.retained(src).goto.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:169
    assert!(park.retained(src).goto.z == -(1e+17_f64));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:170
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_go_down
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_go_down() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 0.0_f64;
    let y = -(1.0_f64);
    let z = 0.0_f64;
    park.goto_direction(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:146
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:147
    assert!(park.retained(src).goto.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:148
    assert!(park.retained(src).goto.y == -(1e+17_f64));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:149
    assert!(park.retained(src).goto.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:150
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_go_forward
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_go_forward() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 1.0_f64;
    park.goto_direction(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:156
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:157
    assert!(park.retained(src).goto.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:158
    assert!(park.retained(src).goto.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:159
    assert!(park.retained(src).goto.z == 1e+17_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:160
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_go_left
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_go_left() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = -(1.0_f64);
    let y = 0.0_f64;
    let z = 0.0_f64;
    park.goto_direction(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:126
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:127
    assert!(park.retained(src).goto.x == -(1e+17_f64));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:128
    assert!(park.retained(src).goto.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:129
    assert!(park.retained(src).goto.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:130
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_go_right
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_go_right() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 1.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    park.goto_direction(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:116
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:117
    assert!(park.retained(src).goto.x == 1e+17_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:118
    assert!(park.retained(src).goto.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:119
    assert!(park.retained(src).goto.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:120
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_go_up
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_go_up() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 0.0_f64;
    let y = 1.0_f64;
    let z = 0.0_f64;
    park.goto_direction(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:136
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:137
    assert!(park.retained(src).goto.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:138
    assert!(park.retained(src).goto.y == 1e+17_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:139
    assert!(park.retained(src).goto.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:140
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoDirection::test_three_dimensional_movement
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoDirection__test_three_dimensional_movement() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.goto_direction(
        ball,
        Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:175
    assert!(
        python_almost_equal(park.retained(ball).goto.x, 5.7735026918962584e+16_f64, 7)
            == Some(true)
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:176
    assert!(
        python_almost_equal(park.retained(ball).goto.y, 5.7735026918962584e+16_f64, 7)
            == Some(true)
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:177
    assert!(
        python_almost_equal(park.retained(ball).goto.z, 5.7735026918962584e+16_f64, 7)
            == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::GotoPoint::test_can_goto
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__GotoPoint__test_can_goto() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    park.goto_point(src, Vec3 { x: x, y: y, z: z });
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:104
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:105
    assert!(park.retained(src).goto.x == x);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:106
    assert!(park.retained(src).goto.y == y);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:107
    assert!(park.retained(src).goto.z == z);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:108
    assert!(f64::from(park.retained(src).speed_fraction) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFollow::test_ball_can_not_follow_moribund_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestFollow__test_ball_can_not_follow_moribund_ball() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.remove_ball(dst, (0.0_f64) as i32);
    park.follow(src, dst, (1.0_f64) as f32, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:28
    assert!(park.retained(src).follow_id != dst);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFollow::test_ball_can_not_follow_self
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestFollow__test_ball_can_not_follow_self() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    park.follow(src, src, (1.0_f64) as f32, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:22
    assert!(park.retained(src).follow_id != src);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFollow::test_follow_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestFollow__test_follow_ball() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.follow(src, dst, (1.0_f64) as f32, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:10
    assert!(park.retained(src).follow_id == dst);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:11
    assert!(f64::from(park.retained(src).follow_range) == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFollow::test_follow_ball_with_range
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestFollow__test_follow_ball_with_range() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.follow(src, dst, (42.0_f64) as f32, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:16
    assert!(park.retained(src).follow_id == dst);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:17
    assert!(f64::from(park.retained(src).follow_range) == 42.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFollow::test_removing_ball_stops_followers_from_following_that_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestFollow__test_removing_bal__5a44843ec1591f394b72d2eb() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.follow(src, dst, (1.0_f64) as f32, false);
    park.remove_ball(dst, (0.0_f64) as i32);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:34
    assert!(park.retained(src).follow_id != dst);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFormationFollow::test_can_follow_in_formation
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestFormationFollow__test_can_follow_in_formation() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.load_original_formations();
    park.set_formation(dst, (1.0_f64) as i8);
    park.formation_follow(src, dst, (1.0_f64) as i8);
    let dst_followers = park.followers(dst);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:353
    assert!(ids_contain(&dst_followers, src));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:354
    assert!(park.retained(src).mode == OriginalMode::Formation);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_can_not_follow_moribund_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestMissileFollow__test_can_not_follow_moribund_ball() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    park.remove_ball(dst, (0.0_f64) as i32);
    park.missile_follow(src, dst, owner);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:55
    assert!(park.retained(src).follow_id != dst);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_removing_ball_stops_followers_from_following_that_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestMissileFollow__test_remov__497bf848c2ea7d2788225516() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    park.follow(src, dst, (1.0_f64) as f32, false);
    park.remove_ball(dst, (0.0_f64) as i32);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:61
    assert!(park.retained(src).follow_id != dst);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_sets_follow_range_as_negative_sum_of_src_and_dst_radii
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestMissileFollow__test_sets___1c924032185d9e37b3c1b708() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let attribute_value_6 = (1.0_f64) as f32;
    park.write_attribute_notified(
        src,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (2.0_f64) as f32;
    park.write_attribute_notified(
        dst,
        |b| b.radius = attribute_value_8,
        OriginalBallAttribute::Radius,
    );
    park.missile_follow(src, dst, owner);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:49
    assert!(f64::from(park.retained(src).follow_range) == -(3.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_sets_ids_correctly
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestMissileFollow__test_sets_ids_correctly() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    park.missile_follow(src, dst, owner);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:41
    assert!(park.retained(src).follow_id == dst);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:42
    assert!(park.retained(src).owner_id == owner);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_sets_missile_mode_on_src_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestMissileFollow__test_sets_missile_mode_on_src_ball() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    park.missile_follow(src, dst, owner);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:66
    assert!(park.retained(src).mode == OriginalMode::Missile);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestOrbit::test_can_not_orbit_at_nan_range
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestOrbit__test_can_not_orbit_at_nan_range() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    park.follow(first, second, (f64::NAN) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:214
    assert!(park.retained(first).follow_id != second);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:215
    assert!(f64::from(park.retained(first).follow_range) != 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:216
    assert!(park.retained(first).mode != OriginalMode::Orbit);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:217
    assert!(!ids_contain(&park.followers(second), first));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestOrbit::test_can_not_orbit_self
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestOrbit__test_can_not_orbit_self() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.follow(ball, ball, (1.0_f64) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:201
    assert!(park.retained(ball).follow_id != ball);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:202
    assert!(f64::from(park.retained(ball).follow_range) != 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:203
    assert!(park.retained(ball).mode != OriginalMode::Orbit);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:204
    assert!(!ids_contain(&park.followers(ball), ball));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestOrbit::test_can_orbit
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestOrbit__test_can_orbit() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    park.follow(first, second, (1.0_f64) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:193
    assert!(park.retained(first).follow_id == second);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:194
    assert!(f64::from(park.retained(first).follow_range) == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:195
    assert!(park.retained(first).mode == OriginalMode::Orbit);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:196
    assert!(ids_contain(&park.followers(second), first));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestOrbit::test_orbit_at_range
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestOrbit__test_orbit_at_range() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    park.follow(first, second, (42.0_f64) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:209
    assert!(f64::from(park.retained(first).follow_range) == 42.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestStop::test_stop_ball_in_formation
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestStop__test_stop_ball_in_formation() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.load_original_formations();
    park.set_formation(dst, (1.0_f64) as i8);
    park.formation_follow(src, dst, (1.0_f64) as i8);
    park.stop(src);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:278
    assert!(park.retained(src).mode == OriginalMode::Stop);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:279
    assert!(!ids_contain(&park.followers(dst), src));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:280
    assert!((park.retained(src).follow_id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:281
    assert!(f64::from(park.retained(src).follow_range) == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestStop::test_stop_following_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestStop__test_stop_following_ball() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    park.follow(first, second, (1.0_f64) as f32, false);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:246
    assert!((park.retained(first).effect_stamp) as f64 == 0.0_f64);
    park.stop(first);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:248
    assert!(park.retained(first).mode == OriginalMode::Stop);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:249
    assert!(!ids_contain(&park.followers(second), first));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:250
    assert!((park.retained(first).follow_id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:251
    assert!(f64::from(park.retained(first).follow_range) == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestStop::test_stop_missile
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestStop__test_stop_missile() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    park.missile_follow(src, dst, owner);
    park.stop(src);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:267
    assert!(park.retained(src).mode == OriginalMode::Stop);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:268
    assert!(!ids_contain(&park.followers(dst), src));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:269
    assert!((park.retained(src).follow_id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:270
    assert!(f64::from(park.retained(src).follow_range) == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestStop::test_stop_orbiting_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestStop__test_stop_orbiting_ball() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    park.follow(first, second, (1.0_f64) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:256
    assert!((park.retained(first).effect_stamp) as f64 == 0.0_f64);
    park.stop(first);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:258
    assert!(park.retained(first).mode == OriginalMode::Stop);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:259
    assert!(!ids_contain(&park.followers(second), first));
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:260
    assert!((park.retained(first).follow_id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:261
    assert!(f64::from(park.retained(first).follow_range) == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestStop::test_stop_stopped_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestStop__test_stop_stopped_ball() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:241
    assert!(park.retained(ball).mode == OriginalMode::Stop);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestWarpTo::test_can_warp
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestWarpTo__test_can_warp() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    park.warp_to(
        src,
        Vec3 { x: x, y: y, z: z },
        20000.0_f64,
        (20.0_f64) as i32,
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:74
    assert!(park.retained(src).mode == OriginalMode::Warp);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:75
    assert!(park.retained(src).goto.x == x);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:76
    assert!(park.retained(src).goto.y == y);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:77
    assert!(park.retained(src).goto.z == z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestWarpTo::test_if_warp_location_is_too_close_we_goto_it_instead
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestWarpTo__test_if_warp_loca__d334086901da9bddc3965978() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    park.warp_to(
        src,
        Vec3 { x: x, y: y, z: z },
        20000.0_f64,
        (20.0_f64) as i32,
    );
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:93
    assert!(park.retained(src).mode == OriginalMode::Goto);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:94
    assert!(park.retained(src).goto.x == x);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:95
    assert!(park.retained(src).goto.y == y);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:96
    assert!(park.retained(src).goto.z == z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestWarpTo::test_minrange_and_warpfactor_are_stored_in_strange_places
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestWarpTo__test_minrange_and__51e8b487911f0a81f7f74373() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    let min_range = 42.0_f64;
    let warp_factor = 10000.0_f64;
    park.warp_to(
        src,
        Vec3 { x: x, y: y, z: z },
        min_range,
        (warp_factor) as i32,
    );
    let min_range_interpreted_as_int = (min_range).to_bits() as i64;
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:86
    assert!(park.retained(src).follow_id == min_range_interpreted_as_int);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:87
    assert!((park.retained(src).owner_id) as f64 == warp_factor);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_time.py::TestPauseAndStart::test_can_pause
// original-source-sha256: 358ad2c8e9b28cf71b0ece55591cea8499b308a8bc0cf28a0717a898c3d58f4f
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn time__TestPauseAndStart__test_can_pause() {
    let mut park = OriginalParkProjection::default();
    park.start();
    park.pause();
    // original-assertion: python/destiny/test/ballpark/test_time.py:29
    assert!(!(park.is_running));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_time.py::TestPauseAndStart::test_can_start
// original-source-sha256: 358ad2c8e9b28cf71b0ece55591cea8499b308a8bc0cf28a0717a898c3d58f4f
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn time__TestPauseAndStart__test_can_start() {
    let mut park = OriginalParkProjection::default();
    park.start();
    // original-assertion: python/destiny/test/ballpark/test_time.py:24
    assert!((park.is_running));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_time.py::TestPauseAndStart::test_initial_state_is_paused
// original-source-sha256: 358ad2c8e9b28cf71b0ece55591cea8499b308a8bc0cf28a0717a898c3d58f4f
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn time__TestPauseAndStart__test_initial_state_is_paused() {
    let mut park = OriginalParkProjection::default();
    // original-assertion: python/destiny/test/ballpark/test_time.py:20
    assert!(!(park.is_running));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_time.py::TestTime::test_adjust_times_adds_to_ballpark_time
// original-source-sha256: 358ad2c8e9b28cf71b0ece55591cea8499b308a8bc0cf28a0717a898c3d58f4f
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn time__TestTime__test_adjust_times_adds_to_ballpark_time() {
    let mut park = OriginalParkProjection::default();
    park.adjust_times((2.0_f64) as i64);
    // original-assertion: python/destiny/test/ballpark/test_time.py:13
    assert!((park.time) as f64 == 2.0_f64);
    park.adjust_times((3.0_f64) as i64);
    // original-assertion: python/destiny/test/ballpark/test_time.py:15
    assert!((park.time) as f64 == 5.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_time.py::TestTime::test_ballpark_time_initialises_to_zero
// original-source-sha256: 358ad2c8e9b28cf71b0ece55591cea8499b308a8bc0cf28a0717a898c3d58f4f
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn time__TestTime__test_ballpark_time_initialises_to_zero() {
    let mut park = OriginalParkProjection::default();
    let old_time = park.time;
    // original-assertion: python/destiny/test/ballpark/test_time.py:9
    assert!((old_time) as f64 == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_default_values
// original-source-sha256: eb79d55cf1e8b94b5895b9824f68c6f5d4fee7c61b384212316f9f284fe45d26
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn ball__BallTest__test_default_values() {
    let mut park = OriginalParkProjection::default();
    let b = OriginalParkBall::default();
    // original-assertion: python/destiny/test/test_ball.py:86
    assert!((b.id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:87
    assert!(b.mass == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:88
    assert!(f64::from(b.radius) == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:89
    assert!(f64::from(b.max_velocity) == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:90
    assert!(!(b.is_free));
    // original-assertion: python/destiny/test/test_ball.py:91
    assert!(!(b.is_global));
    // original-assertion: python/destiny/test/test_ball.py:92
    assert!(!(b.is_massive));
    // original-assertion: python/destiny/test/test_ball.py:93
    assert!(!(b.is_interactive));
    // original-assertion: python/destiny/test/test_ball.py:94
    assert!(!(b.is_cloaked != 0));
    // original-assertion: python/destiny/test/test_ball.py:95
    assert!(!(b.is_moribund));
    // original-assertion: python/destiny/test/test_ball.py:96
    assert!(!(b.is_massive));
    // original-assertion: python/destiny/test/test_ball.py:97
    assert!((b.harmonic) as f64 == -(1.0_f64));
    // original-assertion: python/destiny/test/test_ball.py:98
    assert!((i64::from(b.corporation_id)) as f64 == -(1.0_f64));
    // original-assertion: python/destiny/test/test_ball.py:99
    assert!((i64::from(b.alliance_id)) as f64 == -(1.0_f64));
    // original-assertion: python/destiny/test/test_ball.py:100
    assert!(b.position.x == 1e10_f64);
    // original-assertion: python/destiny/test/test_ball.py:101
    assert!(b.position.y == 1e10_f64);
    // original-assertion: python/destiny/test/test_ball.py:102
    assert!(b.position.z == 1e10_f64);
    // original-assertion: python/destiny/test/test_ball.py:103
    assert!(b.velocity.x == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:104
    assert!(b.velocity.y == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:105
    assert!(b.velocity.z == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:106
    assert!(b.yaw == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:107
    assert!(b.pitch == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:108
    assert!(b.roll == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:109
    assert!(f64::from(b.agility) == 1.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:110
    assert!(f64::from(b.speed_fraction) == 1.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:111
    assert!(b.mode == OriginalMode::Stop);
    // original-assertion: python/destiny/test/test_ball.py:112
    assert!(b.goto.x == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:113
    assert!(b.goto.y == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:114
    assert!(b.goto.z == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:115
    assert!((b.follow_id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:116
    assert!(f64::from(b.follow_range) == 10.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:117
    assert!((b.owner_id) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:118
    assert!((b.effect_stamp) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:119
    assert!((b.new_bubble) as f64 == -(1.0_f64));
    // original-assertion: python/destiny/test/test_ball.py:120
    assert!((b.old_bubble) as f64 == -(1.0_f64));
    // original-assertion: python/destiny/test/test_ball.py:121
    assert!((i64::from(b.formation_id as u8)) as f64 == 255.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:122
    assert!(((b.mini_balls).len() as i64) as f64 == 0.0_f64);
    // original-assertion: python/destiny/test/test_ball.py:123
    assert!((b.in_park.then_some(())).is_none());
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_can_add_miniball
// original-source-sha256: eb79d55cf1e8b94b5895b9824f68c6f5d4fee7c61b384212316f9f284fe45d26
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn ball__BallTest__test_can_add_miniball() {
    let mut park = OriginalParkProjection::default();
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    let x = 1.0_f64;
    let y = 2.0_f64;
    let z = 3.0_f64;
    let radius = 1.0_f64;
    assert!(park.add_mini_ball(b, Vec3 { x: x, y: y, z: z }, (radius) as f32) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/test_ball.py:19
    assert!(((park.retained(b).mini_balls).len() as i64) as f64 == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_can_add_minicapsule
// original-source-sha256: eb79d55cf1e8b94b5895b9824f68c6f5d4fee7c61b384212316f9f284fe45d26
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn ball__BallTest__test_can_add_minicapsule() {
    let mut park = OriginalParkProjection::default();
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            b,
            Vec3 {
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 2.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/test_ball.py:24
    assert!(((park.retained(b).mini_capsules).len() as i64) as f64 == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_can_not_add_minicapsule_with_negative_radius
// original-source-sha256: eb79d55cf1e8b94b5895b9824f68c6f5d4fee7c61b384212316f9f284fe45d26
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn ball__BallTest__test_can_not_add_minicapsule_with_negative_radius() {
    let mut park = OriginalParkProjection::default();
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    // original-assertion: python/destiny/test/test_ball.py:28
    assert!(
        park.add_mini_capsule(
            b,
            Vec3 {
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 2.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            (-(1.0_f64)) as f32
        ) == Err(OriginalParkError::ValueError)
    );
    // original-assertion: python/destiny/test/test_ball.py:30
    assert!(((park.retained(b).mini_capsules).len() as i64) as f64 == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestCapsule::test_parameters_get_set_correctly
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestCapsule__test_parameters_get_set_correctly() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 2.0_f64,
                z: 3.0_f64
            },
            Vec3 {
                x: 4.0_f64,
                y: 5.0_f64,
                z: 6.0_f64
            },
            (42.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_3 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_3.is_some()); // original negative list index must exist
    let capsule = child_access_3.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:257
    assert!(capsule.a.x == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:258
    assert!(capsule.a.y == 2.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:259
    assert!(capsule.a.z == 3.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:261
    assert!(capsule.b.x == 4.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:262
    assert!(capsule.b.y == 5.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:263
    assert!(capsule.b.z == 6.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:265
    assert!(f64::from(capsule.radius) == 42.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_capsule_ids_get_incremented
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_capsule_ids_get_incremented() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 2.0_f64,
                z: 3.0_f64
            },
            Vec3 {
                x: 4.0_f64,
                y: 5.0_f64,
                z: 6.0_f64
            },
            (42.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_4 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_4.is_some()); // original negative list index must exist
    let capsule = child_access_4.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let last_id = capsule.id;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 2.0_f64,
                z: 3.0_f64
            },
            Vec3 {
                x: 4.0_f64,
                y: 5.0_f64,
                z: 6.0_f64
            },
            (42.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_11 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_11.is_some()); // original negative list index must exist
    let capsule = child_access_11.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let next_id = capsule.id;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:318
    assert!((next_id) as f64 == ((last_id) as f64 - 1.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_ball_and_capsule_ids_share_the_same_id_generator
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_bal__de396e28aab508d932f7c84d() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let ball = park.add_ball(OriginalBallInit {
        id: (-(1.0_f64)) as i64,
        mass: 1.0_f64,
        radius: (1.0_f64) as f32,
        max_velocity: (1.0_f64) as f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: (1.0_f64) as f32,
        speed_fraction: (1.0_f64) as f32,
        position: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
        velocity: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error);
    let first_id = ball;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 2.0_f64,
                z: 3.0_f64
            },
            Vec3 {
                x: 4.0_f64,
                y: 5.0_f64,
                z: 6.0_f64
            },
            (42.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_7 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_7.is_some()); // original negative list index must exist
    let capsule = child_access_7.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let second_id = capsule.id;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:328
    assert!((second_id) as f64 == ((first_id) as f64 - 1.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_ball_id_gets_generated
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_ball_id_gets_generated() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let ball = park.add_ball(OriginalBallInit {
        id: (-(1.0_f64)) as i64,
        mass: 1.0_f64,
        radius: (1.0_f64) as f32,
        max_velocity: (1.0_f64) as f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: (1.0_f64) as f32,
        speed_fraction: (1.0_f64) as f32,
        position: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
        velocity: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error);
    let ball_id = ball;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:300
    assert!(ball_id == first_id_generated);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_ball_ids_get_incremented
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_ball_ids_get_incremented() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let ball = park.add_ball(OriginalBallInit {
        id: (-(1.0_f64)) as i64,
        mass: 1.0_f64,
        radius: (1.0_f64) as f32,
        max_velocity: (1.0_f64) as f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: (1.0_f64) as f32,
        speed_fraction: (1.0_f64) as f32,
        position: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
        velocity: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error);
    let last_id = ball;
    let ball = park.add_ball(OriginalBallInit {
        id: (-(1.0_f64)) as i64,
        mass: 1.0_f64,
        radius: (1.0_f64) as f32,
        max_velocity: (1.0_f64) as f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: (1.0_f64) as f32,
        speed_fraction: (1.0_f64) as f32,
        position: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
        velocity: Vec3 {
            x: 1.0_f64,
            y: 1.0_f64,
            z: 1.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = ball;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:313
    assert!((next_id) as f64 == ((last_id) as f64 - 1.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_capsule_id_gets_generated
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_capsule_id_gets_generated() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 2.0_f64,
                z: 3.0_f64
            },
            Vec3 {
                x: 4.0_f64,
                y: 5.0_f64,
                z: 6.0_f64
            },
            (42.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_4 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_4.is_some()); // original negative list index must exist
    let capsule = child_access_4.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let capsule_id = capsule.id;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:304
    assert!(capsule_id == first_id_generated);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_oriented_box_id_gets_generated
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_oriented_box_id_gets_generated() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
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
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 1.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 1.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_4 = (park.retained(parentBall).mini_boxes).get(
        (park.retained(parentBall).mini_boxes)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_4.is_some()); // original negative list index must exist
    let obb = child_access_4.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let obb_id = obb.id;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:308
    assert!(obb_id == first_id_generated);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestIdGeneration::test_oriented_box_ids_get_incremented
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestIdGeneration__test_oriented_box_ids_get_incremented() {
    let mut park = OriginalParkProjection::default();
    let first_id_generated = -(1_i64 << 30) - 1;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
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
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 1.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 1.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_4 = (park.retained(parentBall).mini_boxes).get(
        (park.retained(parentBall).mini_boxes)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_4.is_some()); // original negative list index must exist
    let obb = child_access_4.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let last_id = obb.id;
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit {
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
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 1.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 1.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_11 = (park.retained(parentBall).mini_boxes).get(
        (park.retained(parentBall).mini_boxes)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_11.is_some()); // original negative list index must exist
    let obb = child_access_11.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let next_id = obb.id;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:323
    assert!((next_id) as f64 == ((last_id) as f64 - 1.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_add_proximity_sensor
// original-source-sha256: eb79d55cf1e8b94b5895b9824f68c6f5d4fee7c61b384212316f9f284fe45d26
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn ball__BallTest__test_add_proximity_sensor() {
    let mut park = OriginalParkProjection::default();
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    let sensor_range = 5.0_f64;
    let period = 10.0_f64;
    let shuffle = 1.0_f64;
    let onlyInteractives = 0.0_f64;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            b,
            (sensor_range) as f32,
            period,
            (shuffle) as i32,
            (onlyInteractives) != 0.0,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAddMushroom::test_mushroom_gets_created_correctly
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAddMushroom__test_mushroom_gets_created_correctly() {
    let mut park = OriginalParkProjection::default();
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let _range = 10.0_f64;
    let time = 1.0_f64;
    park.add_mushroom(owner, (_range) as f32, time);
    let mushroom = (0.0_f64) as i64;
    assert!(park.contains(mushroom) && !park.scope_error);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:197
    assert!(park.retained(mushroom).mode == OriginalMode::Mushroom);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:198
    assert!(park.retained(mushroom).owner_id == owner);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:199
    assert!(f64::from(park.retained(mushroom).follow_range) == _range);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:200
    assert!(park.retained(mushroom).goto.x == time);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:201
    assert!(!(park.retained(mushroom).is_moribund));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAddMushroom::test_mushroom_can_not_be_created_with_invalid_owner
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAddMushroom__test_mush__15b113c160fc13a7ae832c62() {
    let mut park = OriginalParkProjection::default();
    let _range = 10.0_f64;
    let time = 1.0_f64;
    park.add_mushroom((1.0_f64) as i64, (_range) as f32, time);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:207
    assert!(!park.contains((0.0_f64) as i64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAddMushroom::test_mushroom_that_is_not_set_free_does_not_expand
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAddMushroom__test_mush__5e33fdb786cb7c731ea25b1a() {
    let mut park = OriginalParkProjection::default();
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let _range = 10.0_f64;
    let time = 5.0_f64;
    park.add_mushroom(owner, (_range) as f32, time);
    let mushroom = (0.0_f64) as i64;
    assert!(park.contains(mushroom) && !park.scope_error);
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:217
    assert!(
        python_almost_equal(f64::from(park.retained(mushroom).radius), 0.0_f64, 7) == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAddMushroom::test_mushroom_expands_correctly
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAddMushroom__test_mushroom_expands_correctly() {
    let mut park = OriginalParkProjection::default();
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let _range = 10.0_f64;
    let time = 5.0_f64;
    park.add_mushroom(owner, (_range) as f32, time);
    let mushroom = (0.0_f64) as i64;
    assert!(park.contains(mushroom) && !park.scope_error);
    park.set_free(mushroom, true);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:226
    assert!(f64::from(park.retained(mushroom).radius) == 0.0_f64);
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:228
    assert!(f64::from(park.retained(mushroom).radius) == 0.0_f64);
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:230
    assert!(
        python_almost_equal(
            f64::from(park.retained(mushroom).radius),
            6.687403202056885_f64,
            7
        ) == Some(true)
    );
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:232
    assert!(
        python_almost_equal(
            f64::from(park.retained(mushroom).radius),
            7.952707290649414_f64,
            7
        ) == Some(true)
    );
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:234
    assert!(
        python_almost_equal(
            f64::from(park.retained(mushroom).radius),
            8.801117897033691_f64,
            7
        ) == Some(true)
    );
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:236
    assert!(
        python_almost_equal(
            f64::from(park.retained(mushroom).radius),
            9.457415580749512_f64,
            7
        ) == Some(true)
    );
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:238
    assert!(
        python_almost_equal(
            f64::from(park.retained(mushroom).radius),
            9.457415580749512_f64,
            7
        ) == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAddMushroom::test_mushrooms_die_when_fully_expanded
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAddMushroom__test_mushrooms_die_when_fully_expanded() {
    let mut park = OriginalParkProjection::default();
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let _range = 10.0_f64;
    let time = 1.0_f64;
    park.add_mushroom(owner, (_range) as f32, time);
    let mushroom = (0.0_f64) as i64;
    assert!(park.contains(mushroom) && !park.scope_error);
    park.set_free(mushroom, true);
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    assert!(park.evolve_owner_and_mushroom() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:249
    assert!((park.retained(mushroom).is_moribund));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestInitialize_bubbles::test_can_initialize_bubbles
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestInitialize_bubbles__test_can_initialize_bubbles() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:11
    assert!((park.retained(ball).new_bubble) as f64 == -(1.0_f64));
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:13
    assert!((park.retained(ball).new_bubble) as f64 != -(1.0_f64));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:14
    assert!((park.retained(ball).old_bubble) as f64 == -(1.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestInitialize_bubbles::test_can_initialize_bubble_for_specific_ball
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestInitialize_bubbles__test___7a4cbd06ea1e5596e46cc05a() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other_ball) && !park.scope_error);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:18
    assert!((park.retained(ball).new_bubble) as f64 == -(1.0_f64));
    assert!(park.initialize_query_bubbles(ball) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:20
    assert!((park.retained(ball).new_bubble) as f64 != -(1.0_f64));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:21
    assert!((park.retained(ball).old_bubble) as f64 == -(1.0_f64));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:22
    assert!((park.retained(other_ball).new_bubble) as f64 == -(1.0_f64));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInRange::test_get_ball_ids_in_range_for_ball_id
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInRange__test_get_ball_ids_in_range_for_ball_id() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_5 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_5.is_ok()); // original query must not raise
    let result = query_ids(&query_result_5.unwrap_or_default());
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:30
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInRange::test_ball_on_edge_of_range_gets_included
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInRange__test_b__c485877a542b60fb1a867394() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 100.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_7 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_7.is_ok()); // original query must not raise
    let result = query_ids(&query_result_7.unwrap_or_default());
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:37
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInRange::test_ignores_out_of_range_balls
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInRange__test_ignores_out_of_range_balls() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 100.1_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_7 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_7.is_ok()); // original query must not raise
    let result = query_ids(&query_result_7.unwrap_or_default());
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:44
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInRange::test_get_ball_ids_in_range_for_location
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInRange__test_g__c0db64a80b1456d3ff3275bf() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_5 = park.query_range(
        None,
        Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z,
        },
        100.0_f64,
        false,
    );
    assert!(query_result_5.is_ok()); // original query must not raise
    let result = query_ids(&query_result_5.unwrap_or_default());
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:50
    assert!(result == id_list(&[ball, other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsAndDistInRange::test_get_ball_ids_in_range_for_ball_id
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsAndDistInRange___6fbf7e70b8d6533cfa362c66() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_5 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_5.is_ok()); // original query must not raise
    let result = query_result_5.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:58
    assert!(result == distance_list(&[(0.0_f64, (2.0_f64) as i64)]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsAndDistInRange::test_returns_distance_squared
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsAndDistInRange__test_returns_distance_squared() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 2.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_7 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_7.is_ok()); // original query must not raise
    let result = query_result_7.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:65
    assert!(result == distance_list(&[(4.0_f64, other)]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsAndDistInRange::test_ball_on_edge_of_range
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsAndDistInRange__test_ball_on_edge_of_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 100.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_7 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_7.is_ok()); // original query must not raise
    let result = query_result_7.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:72
    assert!(result == distance_list(&[(10000.0_f64, other)]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsAndDistInRange::test_ignores_out_of_range_balls
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsAndDistInRange__test_ignores_out_of_range_balls() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 100.1_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_7 = park.query_range(Some(ball), Vec3::ZERO, 100.0_f64, false);
    assert!(query_result_7.is_ok()); // original query must not raise
    let result = query_result_7.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:79
    assert!(result == distance_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsAndDistInRange::test_get_ball_ids_in_range_for_location
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsAndDistInRange___4848e8403e7f5de500ec8916() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_5 = park.query_range(
        None,
        Vec3 {
            x: park.retained(ball).position.x,
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z,
        },
        100.0_f64,
        false,
    );
    assert!(query_result_5.is_ok()); // original query must not raise
    let result = query_result_5.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:85
    assert!(result == distance_list(&[(0.0_f64, ball), (0.0_f64, other)]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdInRangeOfTriangle::test_source_does_not_exist_raises_runtime_error
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdInRangeOfTriangl__1b99713e61b1a083fbfcb6b1() {
    let mut park = OriginalParkProjection::default();
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let ballNotInPark = 12345.0_f64;
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:92
    assert!(
        park.query_triangle(
            (ballNotInPark) as i64,
            Vec3 {
                x: 10.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 10.0_f64,
                z: 0.0_f64
            },
            5.0_f64
        ) == Err(OriginalParkError::RuntimeError)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdInRangeOfTriangle::test_no_other_ball
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdInRangeOfTriangle__test_no_other_ball() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_3 = park.query_triangle(
        ball,
        Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 10.0_f64,
            z: 0.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_3.is_ok()); // original query must not raise
    let result = query_result_3.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:106
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdInRangeOfTriangle::test_other_ball_in_triangle
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdInRangeOfTriangle__test_other_ball_in_triangle() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.x + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.y + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.y = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_11 = park.query_triangle(
        ball,
        Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 10.0_f64,
            z: 0.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_11.is_ok()); // original query must not raise
    let result = query_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:119
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdInRangeOfTriangle::test_other_ball_out_of_range
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdInRangeOfTriangle__test_other_ball_out_of_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.x + 10.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.y + 10.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.y = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_11 = park.query_triangle(
        ball,
        Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 10.0_f64,
            z: 0.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_11.is_ok()); // original query must not raise
    let result = query_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:132
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdInRangeOfTriangle::test_other_ball_on_edge_of_range
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdInRangeOfTriangl__42aaf44ac5e6bc901cd3c65b() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.x + 16.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_9 = park.query_triangle(
        ball,
        Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 10.0_f64,
            z: 0.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_9.is_ok()); // original query must not raise
    let result = query_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:144
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdInRangeOfTriangle::test_multiple_balls_in_triangle
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdInRangeOfTriangl__c3e40bc5d758b78e81d93eaa() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(a) && !park.scope_error);
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    let attribute_value_6 = (1.0_f64) as f32;
    park.write_attribute_notified(
        a,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (1.0_f64) as f32;
    park.write_attribute_notified(
        b,
        |b| b.radius = attribute_value_8,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_10 = (park.retained(a).position.x + 5.0_f64);
    park.write_attribute_notified(
        a,
        |b| b.position.x = attribute_value_10,
        OriginalBallAttribute::Position,
    );
    let attribute_value_12 = (park.retained(b).position.y + 5.0_f64);
    park.write_attribute_notified(
        b,
        |b| b.position.y = attribute_value_12,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_15 = park.query_triangle(
        ball,
        Vec3 {
            x: 10.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 10.0_f64,
            z: 0.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_15.is_ok()); // original query must not raise
    let result = query_result_15.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:158
    assert!(id_sets_equal(&result, &id_list(&[a, b])));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_source_does_not_exist_raises_runtime_error
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test__f4ededdc3b04869f5c446aa3() {
    let mut park = OriginalParkProjection::default();
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let ballNotInPark = 12345.0_f64;
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:165
    assert!(
        park.query_capsule(
            (ballNotInPark) as i64,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 10.0_f64
            },
            5.0_f64
        ) == Err(OriginalParkError::RuntimeError)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_no_other_ball
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test_no_other_ball() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_3 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_3.is_ok()); // original query must not raise
    let result = query_result_3.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:177
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_other_ball_in_capsule
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test_other_ball_in_capsule() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.z + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_9 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_9.is_ok()); // original query must not raise
    let result = query_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:188
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_other_ball_in_capsule_and_on_the_edge_of_one_side
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test__1a5ea967908869e53f3ffbbf() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.z + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.x + 6.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_11 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_11.is_ok()); // original query must not raise
    let result = query_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:200
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_other_ball_not_in_capsule_too_far_from_the_edge_of_one_side
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test__f9d71474df7ae7eb3a729867() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.z + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.x + 7.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_11 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_11.is_ok()); // original query must not raise
    let result = query_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:212
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_other_ball_not_in_capsule
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test_other_ball_not_in_capsule() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.x + 10.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.y + 10.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.y = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_11 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_11.is_ok()); // original query must not raise
    let result = query_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:224
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_other_ball_on_edge_of_capsule
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test_other_ball_on_edge_of_capsule() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.z + 16.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_9 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_9.is_ok()); // original query must not raise
    let result = query_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:235
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCapsule::test_multiple_balls_in_capsule
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCapsule__test_multiple_balls_in_capsule() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let a = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(a) && !park.scope_error);
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    let attribute_value_6 = (1.0_f64) as f32;
    park.write_attribute_notified(
        a,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (1.0_f64) as f32;
    park.write_attribute_notified(
        b,
        |b| b.radius = attribute_value_8,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_10 = (park.retained(a).position.z + 0.0_f64);
    park.write_attribute_notified(
        a,
        |b| b.position.z = attribute_value_10,
        OriginalBallAttribute::Position,
    );
    let attribute_value_12 = (park.retained(b).position.z + 10.0_f64);
    park.write_attribute_notified(
        b,
        |b| b.position.z = attribute_value_12,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let query_result_15 = park.query_capsule(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        5.0_f64,
    );
    assert!(query_result_15.is_ok()); // original query must not raise
    let result = query_result_15.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:248
    assert!(id_sets_equal(&result, &id_list(&[a, b])));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_no_other_ball
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_no_other_ball() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let query_result_2 = park.query_cone(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        0.7853981633974483_f64,
    );
    assert!(query_result_2.is_ok()); // original query must not raise
    let result = query_result_2.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:261
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_source_does_not_exist_raises_runtime_error
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_so__172ffe8446d9d4b77172df0c() {
    let mut park = OriginalParkProjection::default();
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let ballNotInPark = 12345.0_f64;
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:266
    assert!(
        park.query_cone(
            (ballNotInPark) as i64,
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 10.0_f64
            },
            0.7853981633974483_f64
        ) == Err(OriginalParkError::RuntimeError)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_ball_in_cone
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_ball_in_cone() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let attribute_value_5 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_5,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_7 = (park.retained(other).position.z + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_7,
        OriginalBallAttribute::Position,
    );
    let query_result_9 = park.query_cone(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        0.7853981633974483_f64,
    );
    assert!(query_result_9.is_ok()); // original query must not raise
    let result = query_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:278
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_ball_too_far_away
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_ball_too_far_away() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.z + 12.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let query_result_8 = park.query_cone(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        0.7853981633974483_f64,
    );
    assert!(query_result_8.is_ok()); // original query must not raise
    let result = query_result_8.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:285
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_ball_intersects_front_of_cone
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_ball_intersects_front_of_cone() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let attribute_value_5 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_5,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_7 = (park.retained(other).position.z + 11.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_7,
        OriginalBallAttribute::Position,
    );
    let query_result_9 = park.query_cone(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        0.7853981633974483_f64,
    );
    assert!(query_result_9.is_ok()); // original query must not raise
    let result = query_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:293
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_ball_intersects_cone
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_ball_intersects_cone() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let attribute_value_5 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_5,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_7 = (park.retained(other).position.z + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_7,
        OriginalBallAttribute::Position,
    );
    let attribute_value_9 = (park.retained(other).position.x + 6.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_9,
        OriginalBallAttribute::Position,
    );
    let query_result_11 = park.query_cone(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        0.7853981633974483_f64,
    );
    assert!(query_result_11.is_ok()); // original query must not raise
    let result = query_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:302
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBallIdsInCone::test_ball_does_not_intersect_cone
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBallIdsInCone__test_ball_does_not_intersect_cone() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (1.0_f64) as f32;
    park.write_attribute_notified(
        other,
        |b| b.radius = attribute_value_4,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_6 = (park.retained(other).position.z + 5.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = (park.retained(other).position.x + 7.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let query_result_10 = park.query_cone(
        ball,
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 10.0_f64,
        },
        0.7853981633974483_f64,
    );
    assert!(query_result_10.is_ok()); // original query must not raise
    let result = query_result_10.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:310
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_addition
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAdditionsAndDeletions__test_addition() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_interactive = attribute_value_2,
        OriginalBallAttribute::NoHandler,
    );
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[Some(ball)]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let bubble_update_result_10 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_10.is_ok()); // original native mutation must not raise
    let bubble_update_result_10 = bubble_update_result_10.unwrap_or_default();
    additions_per_player = bubble_update_result_10.additions_per_player;
    deletions_per_player = bubble_update_result_10.deletions_per_player;
    additions_per_bubble = bubble_update_result_10.additions_per_bubble;
    deletions_per_bubble = bubble_update_result_10.deletions_per_bubble;
    ();
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:74
    assert!(delta_maps_equal(
        &additions_per_player,
        &delta_map(&[(ball, OriginalBubbleDelta::Balls(id_list(&[ball])))])
    ));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:75
    assert!(delta_maps_equal(
        &deletions_per_player,
        &delta_map(&[(ball, OriginalBubbleDelta::Balls(id_list(&[])))])
    ));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:76
    assert!(delta_maps_equal(
        &additions_per_bubble,
        &delta_map(&[(
            park.retained(ball).new_bubble,
            OriginalBubbleDelta::Balls(id_list(&[ball]))
        )])
    ));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:77
    assert!(delta_maps_equal(
        &deletions_per_bubble,
        &delta_map(&[(
            park.retained(ball).new_bubble,
            OriginalBubbleDelta::Balls(id_list(&[]))
        )])
    ));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_no_change
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAdditionsAndDeletions__test_no_change() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_interactive = attribute_value_2,
        OriginalBallAttribute::NoHandler,
    );
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[Some(ball)]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let bubble_update_result_10 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_10.is_ok()); // original native mutation must not raise
    let bubble_update_result_10 = bubble_update_result_10.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_10.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_10.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_10.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_10.deletions_per_bubble,
    );
    ();
    assert!(park.copy_query_bubbles() == Ok(())); // original call must not raise
    let bubble_update_result_19 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_19.is_ok()); // original native mutation must not raise
    let bubble_update_result_19 = bubble_update_result_19.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_19.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_19.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_19.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_19.deletions_per_bubble,
    );
    ();
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:91
    assert!(delta_maps_equal(
        &additions_per_player,
        &delta_map(&[(
            ball,
            OriginalBubbleDelta::Bubble(park.retained(ball).new_bubble)
        )])
    ));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:92
    assert!(delta_maps_equal(
        &deletions_per_player,
        &delta_map(&[(
            ball,
            OriginalBubbleDelta::Bubble(park.retained(ball).new_bubble)
        )])
    ));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:93
    assert!(delta_maps_equal(
        &additions_per_bubble,
        &delta_map(&[(
            park.retained(ball).new_bubble,
            OriginalBubbleDelta::Balls(id_list(&[]))
        )])
    ));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:94
    assert!(delta_maps_equal(
        &deletions_per_bubble,
        &delta_map(&[(
            park.retained(ball).new_bubble,
            OriginalBubbleDelta::Balls(id_list(&[]))
        )])
    ));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_removal
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAdditionsAndDeletions__test_removal() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_interactive = attribute_value_2,
        OriginalBallAttribute::NoHandler,
    );
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[Some(ball)]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let bubble_update_result_10 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_10.is_ok()); // original native mutation must not raise
    let bubble_update_result_10 = bubble_update_result_10.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_10.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_10.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_10.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_10.deletions_per_bubble,
    );
    ();
    park.remove_ball(ball, (0.0_f64) as i32);
    assert!(park.copy_query_bubbles() == Ok(())); // original call must not raise
    let bubble_update_result_20 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_20.is_ok()); // original native mutation must not raise
    let bubble_update_result_20 = bubble_update_result_20.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_20.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_20.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_20.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_20.deletions_per_bubble,
    );
    ();
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:111
    assert!(delta_maps_equal(&additions_per_player, &delta_map(&[])));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:112
    assert!(delta_maps_equal(&deletions_per_player, &delta_map(&[])));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:113
    assert!(delta_maps_equal(&additions_per_bubble, &delta_map(&[])));
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:114
    assert!(delta_maps_equal(&deletions_per_bubble, &delta_map(&[])));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_addition_of_invalid_ball_id_raises_typeerror
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestAdditionsAndDeletions___3b33c1cce8349d40612fe2f6() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_interactive = attribute_value_2,
        OriginalBallAttribute::NoHandler,
    );
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[None::<i64>]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:54
    assert!(park.bubble_additions_and_deletions(&user_ships) == Err(OriginalParkError::TypeError));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBubbleTransitions::test_empty_bubble_transitions_list_is_sent_as_empty
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestBubbleTransitions__tes__c7298c73a094e732a56214f9() {
    let mut park = OriginalParkProjection::default();
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let bubble_update_result_6 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_6.is_ok()); // original native mutation must not raise
    let bubble_update_result_6 = bubble_update_result_6.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_6.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_6.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_6.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_6.deletions_per_bubble,
    );
    ();
    let expected_transitions = transition_list(&[]);
    // original-assertion: python/destiny/test/helpers.py:288
    assert!(expected_transitions == park.bubble_transitions); // complete original helper assertion
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBubbleTransitions::test_adding_ball_transitions_it_to_bubble
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestBubbleTransitions__tes__c736089673b0c1de5f0cc3dc() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_interactive = attribute_value_2,
        OriginalBallAttribute::NoHandler,
    );
    let ball_id = ball;
    let old_bubble_id = park.retained(ball).new_bubble;
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[Some(ball_id)]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let bubble_update_result_12 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_12.is_ok()); // original native mutation must not raise
    let bubble_update_result_12 = bubble_update_result_12.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_12.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_12.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_12.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_12.deletions_per_bubble,
    );
    ();
    let new_bubble_id = park.retained(ball).new_bubble;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:151
    assert!(old_bubble_id != new_bubble_id);
    let expected_transitions = transition_list(&[(ball_id, old_bubble_id, new_bubble_id)]);
    // original-assertion: python/destiny/test/helpers.py:288
    assert!(expected_transitions == park.bubble_transitions); // complete original helper assertion
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBubbleTransitions::test_adding_two_balls_transition_them_to_bubble
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestBubbleTransitions__tes__f4c2197a913206aa13d3b49b() {
    let mut park = OriginalParkProjection::default();
    let ball_1 = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_1) && !park.scope_error);
    let ball_2 = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball_2) && !park.scope_error);
    let attribute_value_4 = true;
    park.write_attribute_notified(
        ball_1,
        |b| b.is_interactive = attribute_value_4,
        OriginalBallAttribute::NoHandler,
    );
    let attribute_value_6 = true;
    park.write_attribute_notified(
        ball_2,
        |b| b.is_interactive = attribute_value_6,
        OriginalBallAttribute::NoHandler,
    );
    let old_bubble_id_1 = park.retained(ball_1).new_bubble;
    let old_bubble_id_2 = park.retained(ball_2).new_bubble;
    let mut additions_per_player = delta_map(&[]);
    let mut deletions_per_player = delta_map(&[]);
    let mut additions_per_bubble = delta_map(&[]);
    let mut deletions_per_bubble = delta_map(&[]);
    let user_ships = user_list(&[Some(ball_1), Some(ball_2)]);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let bubble_update_result_16 = park.bubble_additions_and_deletions(&user_ships);
    assert!(bubble_update_result_16.is_ok()); // original native mutation must not raise
    let bubble_update_result_16 = bubble_update_result_16.unwrap_or_default();
    store_delta_map(
        &mut additions_per_player,
        bubble_update_result_16.additions_per_player,
    );
    store_delta_map(
        &mut deletions_per_player,
        bubble_update_result_16.deletions_per_player,
    );
    store_delta_map(
        &mut additions_per_bubble,
        bubble_update_result_16.additions_per_bubble,
    );
    store_delta_map(
        &mut deletions_per_bubble,
        bubble_update_result_16.deletions_per_bubble,
    );
    ();
    let new_bubble_id_1 = park.retained(ball_1).new_bubble;
    let new_bubble_id_2 = park.retained(ball_2).new_bubble;
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:177
    assert!(old_bubble_id_1 != new_bubble_id_1);
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:178
    assert!(old_bubble_id_2 != new_bubble_id_2);
    let expected_transitions = transition_list(&[
        (ball_1, old_bubble_id_1, new_bubble_id_1),
        (ball_2, old_bubble_id_2, new_bubble_id_2),
    ]);
    // original-assertion: python/destiny/test/helpers.py:288
    assert!(expected_transitions == park.bubble_transitions); // complete original helper assertion
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestClearAll::test_clear_all_removes_ball_from_park
// original-source-sha256: 2fc8a1007583e83d11a9bea8ac2179e624a202684c22813817a32c60c143f086
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn lifecycle_management__TestClearAll__test_clear_all_removes_ball_from_park() {
    let mut park = OriginalParkProjection::default();
    let ignored_constructor_result_0 = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 {
            x: 1.0_f64,
            y: 2.0_f64,
            z: 3.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ignored_constructor_result_0) && !park.scope_error);
    park.clear_all();
    // original-assertion: python/destiny/test/ballpark/test_lifecycle_management.py:41
    let query_result_4 = park.query_bubble_at(Vec3 {
        x: 1.0_f64,
        y: 2.0_f64,
        z: 3.0_f64,
    });
    assert!(query_result_4.is_ok()); // original coordinate lookup must not raise
    assert!((query_result_4.unwrap_or(None)).is_none());
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetBallNotInParkCallback::test_gets_called_when_addminiball_is_called_on_ball_that_is_not_in_park
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetBallNotInParkCallback__test_ge__9ee1afa8ae5fd8cfd6d6d902() {
    let mut park = OriginalParkProjection::default();
    park.ball_not_in_park_callback_called = false; // original self.called fixture initialization
    park.set_ball_not_in_park_callback(true);
    let mut b = OriginalParkBall::default();
    assert!(
        park.add_detached_mini_ball(
            &mut b,
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: 1.0_f64
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:197
    assert!((park.ball_not_in_park_callback_called));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetBallNotInParkCallback::test_does_not_get_called_when_addminiball_is_called_on_ball_that_is_in_park
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetBallNotInParkCallback__test_do__40cfebf92111549ea1efe5a7() {
    let mut park = OriginalParkProjection::default();
    park.ball_not_in_park_callback_called = false; // original self.called fixture initialization
    park.set_ball_not_in_park_callback(true);
    let b = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(b) && !park.scope_error);
    assert!(
        park.add_mini_ball(
            b,
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: 1.0_f64
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:203
    assert!(!(park.ball_not_in_park_callback_called));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetBallNotInParkCallback::test_remove_callback
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetBallNotInParkCallback__test_remove_callback() {
    let mut park = OriginalParkProjection::default();
    park.ball_not_in_park_callback_called = false; // original self.called fixture initialization
    park.set_ball_not_in_park_callback(true);
    park.set_ball_not_in_park_callback(false);
    let mut b = OriginalParkBall::default();
    assert!(
        park.add_detached_mini_ball(
            &mut b,
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: 1.0_f64
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:210
    assert!(!(park.ball_not_in_park_callback_called));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestNotificationRange::test_detects_ball_in_range_when_in_goto_mode
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestNotificationRange__test_detects_ball_in_range_when_in_goto_mode() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: (park.retained(ball).position.x + 1.0_f64),
            y: 10000000000.0_f64,
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.goto_point(
        ball,
        Vec3 {
            x: park.retained(other).position.x,
            y: park.retained(other).position.y,
            z: park.retained(other).position.z,
        },
    );
    park.set_notification_range(ball, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:117
    assert!(park.retained(ball).range_callback_args == id_list(&[(1.0_f64) as i64]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_callback_gets_called_when_period_has_passed
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_callback_g__581fde7c5485e7c9f1d3d542() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 2.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:18
    assert!(park.retained(proximity_ball).proximity_callback_args == proximity_events(&[]));
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:20
    assert!(
        park.retained(proximity_ball).proximity_callback_args
            == proximity_events(&[(regular_ball, (1.0_f64) as i64)])
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_callback_gets_called
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_callback_gets_called() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:32
    assert!(
        park.retained(proximity_ball).proximity_callback_args
            == proximity_events(&[(regular_ball, (1.0_f64) as i64)])
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_ignores_balls_that_are_not_free
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_ignores_balls_that_are_not_free() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = false;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:44
    assert!(park.retained(proximity_ball).proximity_callback_args == proximity_events(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_only_interactives_ignores_non_interactive_balls
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_only_inter__45d62c0514e13f9dc6e190ca() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = true;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:56
    assert!(park.retained(proximity_ball).proximity_callback_args == proximity_events(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_remove_proximity_sensor
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_remove_proximity_sensor() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    park.remove_proximity_sensor(proximity_ball);
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_15 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_15,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:69
    assert!(park.retained(proximity_ball).proximity_callback_args == proximity_events(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_ball_out_of_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_ball_out_of_range() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: (park.retained(proximity_ball).position.x + 11.0_f64),
            y: 10000000000.0_f64,
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:81
    assert!(park.retained(proximity_ball).proximity_callback_args == proximity_events(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_ball_on_edge_of_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_ball_on_edge_of_range() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: (park.retained(proximity_ball).position.x + 10.0_f64),
            y: 10000000000.0_f64,
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:93
    assert!(
        park.retained(proximity_ball).proximity_callback_args
            == proximity_events(&[(regular_ball, (1.0_f64) as i64)])
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestProximitySensors::test_leave_proximity
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestProximitySensors__test_leave_proximity() {
    let mut park = OriginalParkProjection::default();
    let proximity_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(proximity_ball) && !park.scope_error);
    park.write_attribute(proximity_ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let range = 10.0_f64;
    let period = 0.0_f64;
    let shuffle = false;
    let onlyInteractives = false;
    // External rand() input; Linux RAND_MAX from the inspected C header is 2147483647.
    #[cfg(kani)]
    let random_draw: u32 = kani::any();
    #[cfg(kani)]
    kani::assume(random_draw <= 2_147_483_647);
    #[cfg(test)]
    let random_draw: u32 = 0;
    assert!(
        park.add_proximity_sensor(
            proximity_ball,
            (range) as f32,
            period,
            i32::from(shuffle),
            onlyInteractives,
            random_draw,
            2_147_483_647
        ) == Ok(())
    ); // original call must not raise
    let regular_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(regular_ball) && !park.scope_error);
    let attribute_value_14 = true;
    park.write_attribute_notified(
        regular_ball,
        |b| b.is_free = attribute_value_14,
        OriginalBallAttribute::Free,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    let attribute_value_17 = (park.retained(regular_ball).position.x + 11.0_f64);
    park.write_attribute_notified(
        regular_ball,
        |b| b.position.x = attribute_value_17,
        OriginalBallAttribute::Position,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:107
    assert!(
        park.retained(proximity_ball).proximity_callback_args
            == proximity_events(&[
                (regular_ball, (1.0_f64) as i64),
                (regular_ball, (0.0_f64) as i64)
            ])
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestNotificationRange::test_ignores_ball_on_edge_of_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestNotificationRange__test_ignores_ball_on_edge_of_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        radius: (1.0_f64) as f32,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: (park.retained(ball).position.x + 11.0_f64),
            y: 10000000000.0_f64,
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.goto_point(
        ball,
        Vec3 {
            x: park.retained(other).position.x,
            y: park.retained(other).position.y,
            z: park.retained(other).position.z,
        },
    );
    park.set_notification_range(ball, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:125
    assert!(park.retained(ball).range_callback_args == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestNotificationRange::test_ignores_ball_that_should_be_out_of_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestNotificationRange__test_ignores_ball_that_should_be_out_of_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: 10000000000.0_f64,
            y: (park.retained(ball).position.y + 100.0_f64),
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.goto_point(
        ball,
        Vec3 {
            x: park.retained(other).position.x,
            y: park.retained(other).position.y,
            z: park.retained(other).position.z,
        },
    );
    park.set_notification_range(ball, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:133
    assert!(park.retained(ball).range_callback_args == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestNotificationRange::test_ignores_ball_in_range_if_stopped
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestNotificationRange__test_ignores_ball_in_range_if_stopped() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let ignored_constructor_result_1 = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ignored_constructor_result_1) && !park.scope_error);
    park.set_notification_range(ball, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:140
    assert!(park.retained(ball).range_callback_args == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestNotificationRange::test_detects_ball_leaving_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestNotificationRange__test_detects_ball_leaving_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    park.set_notification_range(ball, 10.0_f64);
    let ignored_constructor_result_1 = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ignored_constructor_result_1) && !park.scope_error);
    park.goto_point(
        ball,
        Vec3 {
            x: (park.retained(ball).position.x + 1.0_f64),
            y: park.retained(ball).position.y,
            z: park.retained(ball).position.z,
        },
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    let attribute_value_8 = (park.retained(ball).position.x + 100.0_f64);
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:150
    assert!(
        park.retained(ball).range_callback_args == id_list(&[(1.0_f64) as i64, (0.0_f64) as i64])
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetTargetTracking::test_foo
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetTargetTracking__test_foo() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.set_target_tracking(ball, other, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:159
    assert!(park.retained(ball).target_tracking_callback_args == id_list(&[(1.0_f64) as i64]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetTargetTracking::test_detects_target_on_edge_of_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetTargetTracking__test_detects_target_on_edge_of_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: (park.retained(ball).position.x + 10.0_f64),
            y: 10000000000.0_f64,
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.set_target_tracking(ball, other, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:166
    assert!(park.retained(ball).target_tracking_callback_args == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetTargetTracking::test_ignores_target_that_should_be_out_of_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetTargetTracking__test_ignores_t__6839b2352c589659e8620b97() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        position: Vec3 {
            x: (park.retained(ball).position.x + 11.0_f64),
            y: 10000000000.0_f64,
            z: 10000000000.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.set_target_tracking(ball, other, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:173
    assert!(park.retained(ball).target_tracking_callback_args == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_callbacks.py::TestSetTargetTracking::test_detects_target_leaving_range
// original-source-sha256: 884737d2a9eaf8ae610fe175d1793e5623ccbfa00ca9b1bf0f45ebce2dcf6ee0
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn callbacks__TestSetTargetTracking__test_detects_target_leaving_range() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    park.write_attribute(ball, |b| b.event_spy = true); // original BallEventSpy wraps this same native ball and initializes its argument lists
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    park.set_target_tracking(ball, other, 10.0_f64);
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    let attribute_value_7 = (park.retained(other).position.x + 100.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_7,
        OriginalBallAttribute::Position,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_callbacks.py:182
    assert!(
        park.retained(ball).target_tracking_callback_args
            == id_list(&[(1.0_f64) as i64, (0.0_f64) as i64])
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetRemoteFollowers::test_finds_following_ball
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetRemoteFollowers__test_finds_following_ball() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.set_free(src, true);
    park.follow(src, dst, (1.0_f64) as f32, false);
    let attribute_value_6 = 0.0_f64;
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let remote_followers_result_9 = park.remote_followers();
    assert!(remote_followers_result_9.is_ok()); // original remote follower call must not raise
    let followers = remote_followers_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:345
    assert!(followers == id_list(&[src]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetRemoteFollowers::test_returns_empty_list_when_there_are_none
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetRemoteFollowers__tes__22c97b39189470afab29bfe3() {
    let mut park = OriginalParkProjection::default();
    let remote_followers_result_0 = park.remote_followers();
    assert!(remote_followers_result_0.is_ok()); // original remote follower call must not raise
    let followers = remote_followers_result_0.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:327
    assert!(followers == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetRemoteFollowers::test_ignores_ball_that_is_not_free
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetRemoteFollowers__test_ignores_ball_that_is_not_free() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.set_free(src, false);
    park.follow(src, dst, (1.0_f64) as f32, false);
    let attribute_value_6 = 0.0_f64;
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let remote_followers_result_9 = park.remote_followers();
    assert!(remote_followers_result_9.is_ok()); // original remote follower call must not raise
    let followers = remote_followers_result_9.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:336
    assert!(followers == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetRemoteFollowers::test_ignores_balls_that_are_not_following
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetRemoteFollowers__tes__daee8d26625110e74073445f() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.set_free(src, true);
    let attribute_value_5 = 0.0_f64;
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_5,
        OriginalBallAttribute::Position,
    );
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let remote_followers_result_8 = park.remote_followers();
    assert!(remote_followers_result_8.is_ok()); // original remote follower call must not raise
    let followers = remote_followers_result_8.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:353
    assert!(followers == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_getters_and_setters.py::TestGetRemoteFollowers::test_ignores_balls_that_are_in_the_same_bubble
// original-source-sha256: 14437b607ecf4a97471762b5f448147f75f70d7ab264a962f8110f30506d2596
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn getters_and_setters__TestGetRemoteFollowers__tes__0fc1bd85c43b2ab0fe6f837a() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    park.set_free(src, true);
    park.follow(src, dst, (1.0_f64) as f32, false);
    assert!(park.initialize_query_bubbles(0_i64) == Ok(())); // original call must not raise
    let remote_followers_result_7 = park.remote_followers();
    assert!(remote_followers_result_7.is_ok()); // original remote follower call must not raise
    let followers = remote_followers_result_7.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_getters_and_setters.py:361
    assert!(followers == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCheckVisibility::test_returns_the_id_of_the_occluding_ball_when_there_is_an_occlusion
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCheckVisibility__test_returns_th__426f23f3d1026353ca3ac3ae() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let occluder = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(occluder) && !park.scope_error);
    let attribute_value_6 = (25.0_f64) as f32;
    park.write_attribute_notified(
        occluder,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (park.retained(occluder).position.x + 50.0_f64);
    park.write_attribute_notified(
        occluder,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let attribute_value_10 = true;
    park.write_attribute_notified(
        occluder,
        |b| b.is_massive = attribute_value_10,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_12 = (park.retained(dst).position.x + 100.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_12,
        OriginalBallAttribute::Position,
    );
    let id = park.check_visibility(src, dst);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:23
    assert!(id == occluder);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCheckVisibility::test_returns_zero_when_there_is_no_occlusion
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCheckVisibility__test_returns_zero_when_there_is_no_occlusion() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let attribute_value_4 = (park.retained(dst).position.x + 100.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let id = park.check_visibility(src, dst);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:14
    assert!((id) as f64 == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCheckVisibility::test_ball_that_is_not_massive_is_not_an_occlusion
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCheckVisibility__test_ball_that___e0c8093aabe84082ed220fd4() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let occluder = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(occluder) && !park.scope_error);
    let attribute_value_6 = (25.0_f64) as f32;
    park.write_attribute_notified(
        occluder,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (park.retained(occluder).position.x + 50.0_f64);
    park.write_attribute_notified(
        occluder,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let attribute_value_10 = false;
    park.write_attribute_notified(
        occluder,
        |b| b.is_massive = attribute_value_10,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_12 = (park.retained(dst).position.x + 100.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_12,
        OriginalBallAttribute::Position,
    );
    let id = park.check_visibility(src, dst);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:32
    assert!(id != occluder);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCheckVisibility::test_ball_that_is_cloaked_is_not_an_occlusion
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCheckVisibility__test_ball_that_is_cloaked_is_not_an_occlusion() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let occluder = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(occluder) && !park.scope_error);
    let attribute_value_6 = (25.0_f64) as f32;
    park.write_attribute_notified(
        occluder,
        |b| b.radius = attribute_value_6,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_8 = (park.retained(occluder).position.x + 50.0_f64);
    park.write_attribute_notified(
        occluder,
        |b| b.position.x = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let attribute_value_10 = true;
    park.write_attribute_notified(
        occluder,
        |b| b.is_massive = attribute_value_10,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_12 = i8::from(true);
    park.write_attribute_notified(
        occluder,
        |b| b.is_cloaked = attribute_value_12,
        OriginalBallAttribute::NoHandler,
    );
    let attribute_value_14 = (park.retained(dst).position.x + 100.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_14,
        OriginalBallAttribute::Position,
    );
    let id = park.check_visibility(src, dst);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:42
    assert!(id != occluder);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCloaking::test_cloaked_balls_are_cloaked
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCloaking__test_cloaked_balls_are_cloaked() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    assert!(park.cloak_ball(ball, 1_i8, (2000.0_f64) as f32) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:49
    assert!((park.retained(ball).is_cloaked != 0));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCloaking::test_cloaked_balls_are_not_massive
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCloaking__test_cloaked_balls_are_not_massive() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = true;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_2,
        OriginalBallAttribute::Massive,
    );
    assert!(park.cloak_ball(ball, 1_i8, (2000.0_f64) as f32) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:55
    assert!(!(park.retained(ball).is_massive));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCloaking::test_uncloaked_balls_are_not_cloaked
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCloaking__test_uncloaked_balls_are_not_cloaked() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = i8::from(true);
    park.write_attribute_notified(
        ball,
        |b| b.is_cloaked = attribute_value_2,
        OriginalBallAttribute::NoHandler,
    );
    park.uncloak_ball(ball);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:61
    assert!(!(park.retained(ball).is_cloaked != 0));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCloaking::test_uncloaking_makes_a_ball_massive
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestCloaking__test_uncloaking_makes_a_ball_massive() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = false;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_2,
        OriginalBallAttribute::Massive,
    );
    park.uncloak_ball(ball);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:67
    assert!((park.retained(ball).is_massive));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestScanCone::test_scan_cone_finds_nothing_when_there_is_nothing_to_be_found
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestScanCone__test_scan_cone_finds_n__edb60f9b7f646212b0b14e49() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let angle = 45.0_f64;
    let _range = 100.0_f64;
    let x = 1.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let scan_result_7 = park.scan_cone(ball, angle, _range, Vec3 { x: x, y: y, z: z });
    assert!(scan_result_7.is_ok()); // original scan must not raise
    let scan_result_7 = scan_result_7.unwrap_or(None);
    assert!(scan_result_7.is_some()); // original result is a list, not None
    let result = scan_result_7.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:92
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestScanCone::test_scan_cone_finds_ball_when_it_is_in_the_cone
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestScanCone__test_scan_cone_finds_ball_when_it_is_in_the_cone() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let other = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(other) && !park.scope_error);
    let attribute_value_4 = (park.retained(other).position.x + 50.0_f64);
    park.write_attribute_notified(
        other,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let angle = (core::f64::consts::PI / 2.0_f64);
    let _range = 100.0_f64;
    let x = 1.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let scan_result_11 = park.scan_cone(ball, angle, _range, Vec3 { x: x, y: y, z: z });
    assert!(scan_result_11.is_ok()); // original scan must not raise
    let scan_result_11 = scan_result_11.unwrap_or(None);
    assert!(scan_result_11.is_some()); // original result is a list, not None
    let result = scan_result_11.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:101
    assert!(result == id_list(&[other]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestScanCone::test_scan_cone_excludes_balls_not_in_cone
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn visibility__TestScanCone__test_scan_cone_excludes_balls_not_in_cone() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let left = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(left) && !park.scope_error);
    let up = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(up) && !park.scope_error);
    let down = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (4.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(down) && !park.scope_error);
    let forward = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (5.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(forward) && !park.scope_error);
    let back = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (6.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(back) && !park.scope_error);
    let attribute_value_12 = (park.retained(left).position.x - 50.0_f64);
    park.write_attribute_notified(
        left,
        |b| b.position.x = attribute_value_12,
        OriginalBallAttribute::Position,
    );
    let attribute_value_14 = (park.retained(up).position.y + 50.0_f64);
    park.write_attribute_notified(
        up,
        |b| b.position.y = attribute_value_14,
        OriginalBallAttribute::Position,
    );
    let attribute_value_16 = (park.retained(down).position.y - 50.0_f64);
    park.write_attribute_notified(
        down,
        |b| b.position.y = attribute_value_16,
        OriginalBallAttribute::Position,
    );
    let attribute_value_18 = (park.retained(forward).position.z + 50.0_f64);
    park.write_attribute_notified(
        forward,
        |b| b.position.z = attribute_value_18,
        OriginalBallAttribute::Position,
    );
    let attribute_value_20 = (park.retained(back).position.z - 50.0_f64);
    park.write_attribute_notified(
        back,
        |b| b.position.z = attribute_value_20,
        OriginalBallAttribute::Position,
    );
    let angle = (core::f64::consts::PI / 2.0_f64);
    let _range = 100.0_f64;
    let x = 1.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let scan_result_27 = park.scan_cone(ball, angle, _range, Vec3 { x: x, y: y, z: z });
    assert!(scan_result_27.is_ok()); // original scan must not raise
    let scan_result_27 = scan_result_27.unwrap_or(None);
    assert!(scan_result_27.is_some()); // original result is a list, not None
    let result = scan_result_27.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:114
    assert!(result == id_list(&[]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestBallBox::test_box_zero_zero_zero
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestBallBox__test_box_zero_zero_zero() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_2,
        OriginalBallAttribute::Position,
    );
    let attribute_value_4 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.y = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let attribute_value_6 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let box_result_8 = park.get_ball_box(ball);
    assert!(box_result_8.is_ok()); // original box query must not raise
    let box_result_8 = box_result_8.unwrap_or(None);
    assert!(box_result_8.is_some()); // original tuple unpacking requires a box
    let (box_width, (x, y, z)) = box_result_8.unwrap_or((0.0, (0.0, 0.0, 0.0)));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:320
    assert!(box_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:321
    assert!(python_almost_equal(x, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:322
    assert!(python_almost_equal(y, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:323
    assert!(python_almost_equal(z, (480.0_f64 / 2.0_f64), 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestBallBox::test_box_zero_zero_one
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestBallBox__test_box_zero_zero_one() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_2,
        OriginalBallAttribute::Position,
    );
    let attribute_value_4 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.y = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let attribute_value_6 = (480.0_f64 + 1.0_f64);
    park.write_attribute_notified(
        ball,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let box_result_8 = park.get_ball_box(ball);
    assert!(box_result_8.is_ok()); // original box query must not raise
    let box_result_8 = box_result_8.unwrap_or(None);
    assert!(box_result_8.is_some()); // original tuple unpacking requires a box
    let (box_width, (x, y, z)) = box_result_8.unwrap_or((0.0, (0.0, 0.0, 0.0)));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:331
    assert!(box_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:332
    assert!(python_almost_equal(x, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:333
    assert!(python_almost_equal(y, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:334
    assert!(python_almost_equal(z, (480.0_f64 * 1.5_f64), 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetActiveBoxes::test_returns_none_when_there_are_no_active_boxes
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetActiveBoxes__test_retu__ba07096bc5678e4d66149aaf() {
    let mut park = OriginalParkProjection::default();
    let level = 0.0_f64;
    let boxes_result_1 = park.get_single_active_box((level) as i32);
    assert!(boxes_result_1.is_ok()); // native order is represented exactly for zero/one cell
    let active_boxes = boxes_result_1.unwrap_or(None);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:454
    assert!((active_boxes).is_none());
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetActiveBoxes::test_returns_list_of_boxes_when_there_are_active_boxes
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetActiveBoxes__test_retu__b0591051042beb7e4a35eb17() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_2,
        OriginalBallAttribute::Position,
    );
    let attribute_value_4 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.y = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let attribute_value_6 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let level = 7.0_f64;
    let boxes_result_9 = park.get_single_active_box((level) as i32);
    assert!(boxes_result_9.is_ok()); // native order is represented exactly for zero/one cell
    let box_unpack_11 = boxes_result_9.unwrap_or(None);
    assert!(box_unpack_11.is_some()); // original tuple unpacking requires a value
    let (level_width, box_low_coord_list) =
        box_unpack_11.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:463
    assert!(level_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:464
    assert!(points_exact_equal(
        &box_low_coord_list,
        &point_list::<1>(&[Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64
        }])
    ));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBoxCenter::test_get_box_on_topmost_level
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBoxCenter__test_get_box_on_topmost_level() {
    let mut park = OriginalParkProjection::default();
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let topmost_level = 7.0_f64;
    let center_result_4 = park.get_grid_center((topmost_level) as i32, Vec3 { x: x, y: y, z: z });
    assert!(center_result_4.is_ok()); // source level/coordinate conversion must be defined
    let center_of_box = center_result_4.unwrap_or((0.0, 0.0, 0.0));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:639
    assert!(
        center_of_box
            == (
                (480.0_f64 / 2.0_f64),
                (480.0_f64 / 2.0_f64),
                (480.0_f64 / 2.0_f64)
            )
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBubbleAtCoordinates::test_get_bubble_where_none_exists_returns_none
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBubbleAtCoordinates__t__67b1e797cf00e890a0c9b1df() {
    let mut park = OriginalParkProjection::default();
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let query_result_3 = park.query_bubble_at(Vec3 { x: x, y: y, z: z });
    assert!(query_result_3.is_ok()); // original coordinate lookup must not raise
    let bubble = query_result_3.unwrap_or(None);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:646
    assert!(bubble == None::<i64>);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetBubbleAtCoordinates::test_get_existing_bubble
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetBubbleAtCoordinates__test_get_existing_bubble() {
    let mut park = OriginalParkProjection::default();
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        position: Vec3 { x: x, y: y, z: z },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let query_result_5 = park.query_bubble_at(Vec3 { x: x, y: y, z: z });
    assert!(query_result_5.is_ok()); // original coordinate lookup must not raise
    let bubble = query_result_5.unwrap_or(None);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:652
    assert!(bubble == Some(park.retained(ball).new_bubble));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetAndSetBoxObject::test_setting_object_on_existing_box_succeeds
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetAndSetBoxObject__test___f7e6fcaa6815a0eeeec66764() {
    let mut park = OriginalParkProjection::default();
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_5 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_5,
        OriginalBallAttribute::Position,
    );
    let attribute_value_7 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.y = attribute_value_7,
        OriginalBallAttribute::Position,
    );
    let attribute_value_9 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.z = attribute_value_9,
        OriginalBallAttribute::Position,
    );
    let d = box_object(&[(OriginalBoxObjectKey::Foo, ActionValue::Float(3.14_f64))]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:613
    let object_set_13 = park.set_box_object(Vec3 { x: x, y: y, z: z }, d);
    assert!(object_set_13.is_ok()); // original object setter must not raise
    assert!((object_set_13.unwrap_or(false)));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetAndSetBoxObject::test_setting_object_on_nonexisting_box_fails
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetAndSetBoxObject__test___983492f584f1c79b4b32682a() {
    let mut park = OriginalParkProjection::default();
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let d = box_object(&[(OriginalBoxObjectKey::Foo, ActionValue::Float(3.14_f64))]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:618
    let object_set_5 = park.set_box_object(Vec3 { x: x, y: y, z: z }, d);
    assert!(object_set_5.is_ok()); // original object setter must not raise
    assert!(!(object_set_5.unwrap_or(false)));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetAndSetBoxObject::test_getting_object_that_has_not_been_set_returns_none
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetAndSetBoxObject__test___d540d83a64224d07a57f1415() {
    let mut park = OriginalParkProjection::default();
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:621
    let object_get_1 = park.get_box_object(Vec3 {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    });
    assert!(object_get_1.is_ok()); // original object getter must not raise
    assert!(object_get_1.unwrap_or(None) == None::<OriginalBoxObject>);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetAndSetBoxObject::test_can_retrieve_stored_object
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetAndSetBoxObject__test_can_retrieve_stored_object() {
    let mut park = OriginalParkProjection::default();
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_5 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_5,
        OriginalBallAttribute::Position,
    );
    let attribute_value_7 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.y = attribute_value_7,
        OriginalBallAttribute::Position,
    );
    let attribute_value_9 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.z = attribute_value_9,
        OriginalBallAttribute::Position,
    );
    let d = box_object(&[(OriginalBoxObjectKey::Foo, ActionValue::Float(3.14_f64))]);
    let object_set_12 = park.set_box_object(Vec3 { x: x, y: y, z: z }, d);
    assert!(object_set_12.is_ok()); // original object setter must not raise
    let _original_ignored_success = object_set_12.unwrap_or(false); // ignored original return; no-unexpected-exception check retained
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:631
    let object_get_16 = park.get_box_object(Vec3 {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    });
    assert!(object_get_16.is_ok()); // original object getter must not raise
    assert!(object_get_16.unwrap_or(None) == Some(d));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_capsule_in_box_zero_zero_zero
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__t__c365d0bd469e19c8e800fea1() {
    let mut park = OriginalParkProjection::default();
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
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: 1.0_f64
            },
            Vec3 {
                x: 2.0_f64,
                y: 2.0_f64,
                z: 2.0_f64
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_3 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_3.is_some()); // original list index must exist
    let capsule = child_access_3.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let box_result_6 = park.get_static_collidable_box(capsule.id);
    assert!(box_result_6.is_ok()); // original box query must not raise
    let box_result_6 = box_result_6.unwrap_or(None);
    assert!(box_result_6.is_some()); // original tuple unpacking requires a box
    let (box_width, (x, y, z)) = box_result_6.unwrap_or((0.0, (0.0, 0.0, 0.0)));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:347
    assert!(box_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:348
    assert!(python_almost_equal(x, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:349
    assert!(python_almost_equal(y, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:350
    assert!(python_almost_equal(z, (480.0_f64 / 2.0_f64), 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_capsule_in_box_zero_zero_one
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__test_capsule_in_box_zero_zero_one() {
    let mut park = OriginalParkProjection::default();
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
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: (480.0_f64 + 1.0_f64)
            },
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: (480.0_f64 + 3.0_f64)
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let child_access_3 = (park.retained(parentBall).mini_capsules).get(
        (park.retained(parentBall).mini_capsules)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_3.is_some()); // original list index must exist
    let capsule = child_access_3.unwrap_or(OriginalMiniCapsule {
        id: 0,
        parent_id: 0,
        a: Vec3::ZERO,
        b: Vec3::ZERO,
        radius: 0.0,
    });
    let box_result_6 = park.get_static_collidable_box(capsule.id);
    assert!(box_result_6.is_ok()); // original box query must not raise
    let box_result_6 = box_result_6.unwrap_or(None);
    assert!(box_result_6.is_some()); // original tuple unpacking requires a box
    let (box_width, (x, y, z)) = box_result_6.unwrap_or((0.0, (0.0, 0.0, 0.0)));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:361
    assert!(box_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:362
    assert!(python_almost_equal(x, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:363
    assert!(python_almost_equal(y, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:364
    assert!(python_almost_equal(z, (480.0_f64 * 1.5_f64), 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_oriented_box_in_box_zero_zero_zero
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__t__74e7e249be8101bcd6fa234e() {
    let mut park = OriginalParkProjection::default();
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
                x: (480.0_f64 / 4.0_f64),
                y: (480.0_f64 / 4.0_f64),
                z: (480.0_f64 / 4.0_f64)
            },
            Vec3 {
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 1.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 1.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_3 = (park.retained(parentBall).mini_boxes).get(
        (park.retained(parentBall).mini_boxes)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_3.is_some()); // original list index must exist
    let obb = child_access_3.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let box_result_6 = park.get_static_collidable_box(obb.id);
    assert!(box_result_6.is_ok()); // original box query must not raise
    let box_result_6 = box_result_6.unwrap_or(None);
    assert!(box_result_6.is_some()); // original tuple unpacking requires a box
    let (box_width, (x, y, z)) = box_result_6.unwrap_or((0.0, (0.0, 0.0, 0.0)));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:376
    assert!(box_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:377
    assert!(python_almost_equal(x, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:378
    assert!(python_almost_equal(y, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:379
    assert!(python_almost_equal(z, (480.0_f64 / 2.0_f64), 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_oriented_box_in_box_zero_zero_one
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__t__10a654df0f219a6cd4ed2ceb() {
    let mut park = OriginalParkProjection::default();
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
                x: (480.0_f64 / 4.0_f64),
                y: (480.0_f64 / 4.0_f64),
                z: (480.0_f64 * 1.5_f64)
            },
            Vec3 {
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 1.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 1.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let child_access_3 = (park.retained(parentBall).mini_boxes).get(
        (park.retained(parentBall).mini_boxes)
            .len()
            .saturating_sub(1),
    );
    assert!(child_access_3.is_some()); // original list index must exist
    let obb = child_access_3.unwrap_or(OriginalMiniBox {
        id: 0,
        parent_id: 0,
        corner: Vec3::ZERO,
        x: Vec3::ZERO,
        y: Vec3::ZERO,
        z: Vec3::ZERO,
    });
    let box_result_6 = park.get_static_collidable_box(obb.id);
    assert!(box_result_6.is_ok()); // original box query must not raise
    let box_result_6 = box_result_6.unwrap_or(None);
    assert!(box_result_6.is_some()); // original tuple unpacking requires a box
    let (box_width, (x, y, z)) = box_result_6.unwrap_or((0.0, (0.0, 0.0, 0.0)));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:391
    assert!(box_width == 480.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:392
    assert!(python_almost_equal(x, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:393
    assert!(python_almost_equal(y, (480.0_f64 / 2.0_f64), 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:394
    assert!(python_almost_equal(z, (480.0_f64 * 1.5_f64), 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_adding_a_capsule_makes_a_box_active
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__t__8a5a1ee9c90c5367ecae5e92() {
    let mut park = OriginalParkProjection::default();
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
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 2.0_f64,
                y: 2.0_f64,
                z: 2.0_f64
            },
            Vec3 {
                x: 3.0_f64,
                y: 3.0_f64,
                z: 3.0_f64
            },
            (1.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, false);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<1>(&[Vec3 {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    }]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:406
    assert!(points_exact_equal(&active_boxes, &expected_active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_adding_a_large_capsule_makes_adjecent_boxes_active
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__t__5327b1a6bdd7427a3f9b2b0b() {
    let mut park = OriginalParkProjection::default();
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
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: 1.0_f64,
                y: 1.0_f64,
                z: 1.0_f64
            },
            Vec3 {
                x: 2.0_f64,
                y: 2.0_f64,
                z: 2.0_f64
            },
            (2.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<8>(&[
        Vec3 {
            x: -(480.0_f64),
            y: -(480.0_f64),
            z: -(480.0_f64),
        },
        Vec3 {
            x: -(480.0_f64),
            y: -(480.0_f64),
            z: 0.0_f64,
        },
        Vec3 {
            x: -(480.0_f64),
            y: 0.0_f64,
            z: -(480.0_f64),
        },
        Vec3 {
            x: -(480.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: -(480.0_f64),
            z: -(480.0_f64),
        },
        Vec3 {
            x: 0.0_f64,
            y: -(480.0_f64),
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(480.0_f64),
        },
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:427
    assert!(points_exact_equal(
        &sort_points(&active_boxes),
        &expected_active_boxes
    ));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestGetStaticCollidableBox::test_adding_a_long_capsule_makes_adjecent_boxes_active
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestGetStaticCollidableBox__t__532151085bcf0de5e6911225() {
    let mut park = OriginalParkProjection::default();
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
        park.add_mini_capsule(
            parentBall,
            Vec3 {
                x: (480.0_f64 / 2.0_f64),
                y: (480.0_f64 / 2.0_f64),
                z: (480.0_f64 / 2.0_f64)
            },
            Vec3 {
                x: (480.0_f64 * 6.0_f64),
                y: (480.0_f64 / 2.0_f64),
                z: (480.0_f64 / 2.0_f64)
            },
            (480.0_f64 / 3.0_f64) as f32
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<7>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 480.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: (480.0_f64 * 2.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: (480.0_f64 * 3.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: (480.0_f64 * 4.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: (480.0_f64 * 5.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: (480.0_f64 * 6.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:447
    assert!(points_exact_equal(
        &sort_points(&active_boxes),
        &expected_active_boxes
    ));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_adding_a_box_makes_a_box_active
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__ab33ace90f2d3108d183c144() {
    let mut park = OriginalParkProjection::default();
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
        park.add_mini_box(
            parentBall,
            Vec3 {
                x: 2.0_f64,
                y: 2.0_f64,
                z: 2.0_f64
            },
            Vec3 {
                x: 1.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 1.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 1.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, false);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<1>(&[Vec3 {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    }]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:479
    assert!(points_exact_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_tight_fit
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_tight_fit() {
    let mut park = OriginalParkProjection::default();
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
                x: 10.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 10.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 10.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, false);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<1>(&[Vec3 {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    }]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:492
    assert!(points_exact_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_level_six
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_level_six() {
    let mut park = OriginalParkProjection::default();
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
                x: 10.1_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 10.1_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 10.1_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 6.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, false);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<1>(&[Vec3 {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    }]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:505
    assert!(points_exact_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_left
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_left() {
    let mut park = OriginalParkProjection::default();
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
                x: -(1.0_f64),
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<2>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: -(480.0_f64),
            y: 0.0_f64,
            z: 0.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:518
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_right
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_right() {
    let mut park = OriginalParkProjection::default();
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
                x: (480.0_f64 - 4.0_f64),
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<2>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 480.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:531
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_bottom
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_bottom() {
    let mut park = OriginalParkProjection::default();
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
                y: -(1.0_f64),
                z: 0.0_f64
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<2>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: -(480.0_f64),
            z: 0.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:544
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_top
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_top() {
    let mut park = OriginalParkProjection::default();
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
                y: (480.0_f64 - 4.0_f64),
                z: 0.0_f64
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<2>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 480.0_f64,
            z: 0.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:557
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_back
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_back() {
    let mut park = OriginalParkProjection::default();
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
                z: -(1.0_f64)
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<2>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: -(480.0_f64),
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:570
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_front
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_front() {
    let mut park = OriginalParkProjection::default();
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
                z: (480.0_f64 - 4.0_f64)
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<2>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 480.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:583
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_boxes_and_bubbles.py::TestActiveBoxesForOrientedBox::test_intersect_top_front_right
// original-source-sha256: 2b2e238ceba85e3ad6a1d4e3c6a85f8fd8dd7877dda08e93f662950ea3c0100a
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn boxes_and_bubbles__TestActiveBoxesForOrientedBox__test_intersect_top_front_right() {
    let mut park = OriginalParkProjection::default();
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
                x: (480.0_f64 - 4.0_f64),
                y: (480.0_f64 - 4.0_f64),
                z: (480.0_f64 - 4.0_f64)
            },
            Vec3 {
                x: 5.0_f64,
                y: 0.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 5.0_f64,
                z: 0.0_f64
            },
            Vec3 {
                x: 0.0_f64,
                y: 0.0_f64,
                z: 5.0_f64
            }
        ) == Ok(())
    ); // original call must not raise
    let level = 7.0_f64;
    let boxes_result_4 = park.get_active_box_view((level) as i32, true);
    assert!(boxes_result_4.is_ok()); // exact order for zero/one cell; otherwise declared set/sorted view
    let box_unpack_6 = boxes_result_4.unwrap_or(None);
    assert!(box_unpack_6.is_some()); // original tuple unpacking requires a value
    let (level_width, active_boxes) = box_unpack_6.unwrap_or((0.0, BoundedList::<Vec3, 64>::new()));
    let expected_active_boxes = point_list::<8>(&[
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 480.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 480.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 480.0_f64,
        },
        Vec3 {
            x: 0.0_f64,
            y: 480.0_f64,
            z: 480.0_f64,
        },
        Vec3 {
            x: 480.0_f64,
            y: 480.0_f64,
            z: 0.0_f64,
        },
        Vec3 {
            x: 480.0_f64,
            y: 0.0_f64,
            z: 480.0_f64,
        },
        Vec3 {
            x: 480.0_f64,
            y: 480.0_f64,
            z: 480.0_f64,
        },
    ]);
    // original-assertion: python/destiny/test/ballpark/test_boxes_and_bubbles.py:603
    assert!(point_sets_equal(&expected_active_boxes, &active_boxes));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_src_gets_launched_at_dst
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_src_gets_launched_at_dst() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let aimed_launch = false;
    let massive = false;
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:290
    assert!(park.retained(src).mode == OriginalMode::Missile);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:291
    assert!(ids_contain(&park.followers(dst), src));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_owner_of_missile_is_correct
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_owner_of_missile_is_correct() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let aimed_launch = false;
    let massive = false;
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:298
    assert!(park.retained(src).owner_id == owner);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_massive_missile
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_massive_missile() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let aimed_launch = false;
    let massive = true;
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:305
    assert!(park.retained(src).is_massive == true);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_direction_of_non_aimed_missiles
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_direction_of_non_aimed_missiles() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let attribute_value_6 = (park.retained(dst).position.x + 10000.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let aimed_launch = false;
    let massive = false;
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:313
    assert!(park.retained(src).velocity.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:314
    assert!(park.retained(src).velocity.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:315
    assert!(python_almost_equal(park.retained(src).velocity.z, -(150.0_f64), 4) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_additional_velocity_of_missile_is_the_same_as_the_max_velocity_of_its_owner_if_above_limit
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_addit__e644bbb51f63be5235cb007c() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let attribute_value_6 = (park.retained(dst).position.x + 10000.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let aimed_launch = false;
    let massive = false;
    park.set_max_speed(owner, (1234.0_f64) as f32);
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:324
    assert!(park.retained(src).velocity.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:325
    assert!(park.retained(src).velocity.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:326
    assert!(python_almost_equal(park.retained(src).velocity.z, -(1234.0_f64), 3) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_initial_velocity_of_owner_has_impact_of_non_aimed_missile_direction_and_speed
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_initi__8b4e8f1177e0ebdf44b7f320() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let attribute_value_6 = (park.retained(dst).position.x + 10000.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let aimed_launch = false;
    let massive = false;
    let attribute_value_10 = 100.0_f64;
    park.write_attribute_notified(
        owner,
        |b| b.velocity.x = attribute_value_10,
        OriginalBallAttribute::Velocity,
    );
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:335
    assert!(
        (
            park.retained(src).velocity.x,
            park.retained(src).velocity.y,
            park.retained(src).velocity.z
        ) == (250.0_f64, 0.0_f64, 0.0_f64)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestLaunchMissile::test_direction_of_aimed_missiles
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestLaunchMissile__test_direction_of_aimed_missiles() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let dst = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(dst) && !park.scope_error);
    let owner = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (3.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(owner) && !park.scope_error);
    let attribute_value_6 = (park.retained(dst).position.x + 10000.0_f64);
    park.write_attribute_notified(
        dst,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let aimed_launch = true;
    let massive = false;
    assert!(
        park.launch_missile(src, dst, owner, i8::from(aimed_launch), i8::from(massive)) == Ok(())
    ); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:343
    assert!(
        (
            park.retained(src).velocity.x,
            park.retained(src).velocity.y,
            park.retained(src).velocity.z
        ) == (1.0_f64, 0.0_f64, 0.0_f64)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestOrbit::test_can_not_orbit_cloaked_ball
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestOrbit__test_can_not_orbit_cloaked_ball() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    assert!(park.cloak_ball(second, 1_i8, (2000.0_f64) as f32) == Ok(())); // original call must not raise
    park.follow(first, second, (1.0_f64) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:223
    assert!(park.retained(first).follow_id != second);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:224
    assert!(f64::from(park.retained(first).follow_range) != 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:225
    assert!(park.retained(first).mode != OriginalMode::Orbit);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:226
    assert!(!ids_contain(&park.followers(second), first));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestOrbit::test_can_not_orbit_ball_in_different_bubble
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(test, test)]
fn movement_controls__TestOrbit__test_can_not_orbit_ball_in_different_bubble() {
    let mut park = OriginalParkProjection::default();
    let first = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(first) && !park.scope_error);
    let second = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (2.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(second) && !park.scope_error);
    assert!(park.initialize_query_bubbles(second) == Ok(())); // original call must not raise
    park.follow(first, second, (1.0_f64) as f32, true);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:232
    assert!(park.retained(first).follow_id != second);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:233
    assert!(f64::from(park.retained(first).follow_range) != 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:234
    assert!(f64::from(park.retained(first).follow_range) != 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:235
    assert!(park.retained(first).mode != OriginalMode::Orbit);
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:236
    assert!(!ids_contain(&park.followers(second), first));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestEntityWarpIn::test_warps_immediately
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestEntityWarpIn__test_warps_immediately() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let warpFactor = 1.0_f64;
    assert!(park.entity_warp_in(src, Vec3 { x: x, y: y, z: z }, (warpFactor) as i32) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:363
    assert!(park.retained(src).mode == OriginalMode::Warp);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestEntityWarpIn::test_velocity_is_approximately_one_au
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestEntityWarpIn__test_velocity_is_approximately_one_au() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let attribute_value_2 = 0.0_f64;
    park.write_attribute_notified(
        src,
        |b| b.position.x = attribute_value_2,
        OriginalBallAttribute::Position,
    );
    let attribute_value_4 = 0.0_f64;
    park.write_attribute_notified(
        src,
        |b| b.position.y = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let attribute_value_6 = 0.0_f64;
    park.write_attribute_notified(
        src,
        |b| b.position.z = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let x = 1e10_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let warpFactor = 1.0_f64;
    assert!(park.entity_warp_in(src, Vec3 { x: x, y: y, z: z }, (warpFactor) as i32) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:371
    assert!(
        python_almost_equal(park.retained(src).velocity.x, 149_597_870_700.0_f64, 7) == Some(true)
    );
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestEntityWarpIn::test_ball_in_goto_mode_does_not_warp
// original-source-sha256: 8e91759d6ab143adcb64e4889bd597eb48ff2a69ae6ce6dbdd53477f23f2fcf7
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn movement_controls__TestEntityWarpIn__test_ball_in_goto_mode_does_not_warp() {
    let mut park = OriginalParkProjection::default();
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(src) && !park.scope_error);
    let x = 0.0_f64;
    let y = 0.0_f64;
    let z = 0.0_f64;
    let warpFactor = 1.0_f64;
    let attribute_value_6 = 1.0_f64;
    park.write_attribute_notified(
        src,
        |b| b.position.x = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = 0.0_f64;
    park.write_attribute_notified(
        src,
        |b| b.position.y = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let attribute_value_10 = 0.0_f64;
    park.write_attribute_notified(
        src,
        |b| b.position.z = attribute_value_10,
        OriginalBallAttribute::Position,
    );
    park.goto_point(src, Vec3 { x: x, y: y, z: z });
    assert!(park.entity_warp_in(src, Vec3 { x: x, y: y, z: z }, (warpFactor) as i32) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/test_movement_controls.py:380
    assert!(park.retained(src).mode != OriginalMode::Warp);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/test_visibility.py::TestCloaking::test_uncloaking_a_ball_in_warp_does_not_make_it_massive
// original-source-sha256: cce3f8bea2aba26789c89a00675dfa698d90d906f7d87af784900d69fbb0bc66
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn visibility__TestCloaking__test_uncloaking_a_ball__0c28ef41d4ca5862af85cdc4() {
    let mut park = OriginalParkProjection::default();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: (1.0_f64) as i64,
        ..OriginalBallInit::default()
    });
    assert!(park.contains(ball) && !park.scope_error);
    let attribute_value_2 = false;
    park.write_attribute_notified(
        ball,
        |b| b.is_massive = attribute_value_2,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_4 = 0.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.x = attribute_value_4,
        OriginalBallAttribute::Position,
    );
    let attribute_value_6 = 0.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.y = attribute_value_6,
        OriginalBallAttribute::Position,
    );
    let attribute_value_8 = 0.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.position.z = attribute_value_8,
        OriginalBallAttribute::Position,
    );
    let attribute_value_10 = (1.0_f64) as f32;
    park.write_attribute_notified(
        ball,
        |b| b.max_velocity = attribute_value_10,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_12 = 1.0_f64;
    park.write_attribute_notified(
        ball,
        |b| b.velocity.x = attribute_value_12,
        OriginalBallAttribute::Velocity,
    );
    park.set_free(ball, true);
    park.warp_to(
        ball,
        Vec3 {
            x: 1e10_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        20000.0_f64,
        (20.0_f64) as i32,
    );
    assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
    park.uncloak_ball(ball);
    // original-assertion: python/destiny/test/ballpark/test_visibility.py:82
    assert!(!(park.retained(ball).is_massive));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}
