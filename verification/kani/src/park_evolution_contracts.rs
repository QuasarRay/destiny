// Copyright © 2015 CCP ehf. MIT license, retained from original Destiny tests.
// Generated from pinned Git blobs by generate_park_harnesses.py.
// Concrete state projection: 8 retained objects, 4096 ordered boundary calls.
// Spatial indexing, Blue dispatch/lifetime, interpreter and native libm
// correspondence remain required; no production refinement is claimed here.
#![allow(non_snake_case, unused_variables)]
use destiny_original_spec::{BoundedList, OriginalBallInit, OriginalBallAttribute, OriginalMode, OriginalParkBall, OriginalParkError, OriginalMiniBall, OriginalMiniCapsule, OriginalMiniBox,
    OriginalParkProjection, OriginalBubbleDelta, OriginalBubbleDeltaMap, OriginalBoxObject,OriginalBoxObjectKey,ActionValue,Vec3, python_almost_equal, python_rounded_equal};

fn box_object(values:&[(OriginalBoxObjectKey,ActionValue)]) -> OriginalBoxObject {
    let mut result=BoundedList::new();let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}

fn user_list(values:&[Option<i64>]) -> BoundedList<Option<i64>,8> {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn proximity_events(values:&[(i64,i64)]) -> BoundedList<(i64,i64),8> {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn transition_list(values:&[(i64,i64,i64)]) -> BoundedList<(i64,i64,i64),8> {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn delta_map(values:&[(i64,OriginalBubbleDelta)]) -> OriginalBubbleDeltaMap {
    let mut result=BoundedList::new(); let mut i=0;
    while i<values.len() { if let Some(value)=values.get(i) { assert!(result.push(*value)); } i+=1; }
    result
}
fn delta_maps_equal(left:&OriginalBubbleDeltaMap,right:&OriginalBubbleDeltaMap) -> bool {
    if left.len()!=right.len() { return false; }
    let mut i=0;
    while i<left.len() {
        let value=left.get(i); let mut matches=0; let mut j=0;
        while j<right.len() { if value==right.get(j) { matches+=1; } j+=1; }
        if matches!=1 { return false; } i+=1;
    }
    true
}
fn store_delta_map(destination:&mut OriginalBubbleDeltaMap,value:OriginalBubbleDeltaMap) {
    *destination=value;
}

fn point_list<const N: usize>(points: &[Vec3]) -> BoundedList<Vec3,N> {
    let mut result=BoundedList::new();
    let mut i=0_usize;
    while i<points.len() {
        if let Some(point)=points.get(i) { assert!(result.push(*point)); }
        i=i.saturating_add(1);
    }
    result
}
fn points_exact_equal<const A:usize,const B:usize>(left:&BoundedList<Vec3,A>,right:&BoundedList<Vec3,B>) -> bool {
    if left.len()!=right.len() { return false; }
    let mut i=0;
    while i<left.len() { if left.get(i)!=right.get(i) { return false; } i+=1; }
    true
}
fn point_sets_equal<const A:usize,const B:usize>(left:&BoundedList<Vec3,A>,right:&BoundedList<Vec3,B>) -> bool {
    let mut i=0;
    while i<left.len() { let mut found=false;let mut j=0;
        while j<right.len() { found|=left.get(i)==right.get(j);j+=1; }if !found { return false; }i+=1;
    }
    i=0;
    while i<right.len() { let mut found=false;let mut j=0;
        while j<left.len() { found|=right.get(i)==left.get(j);j+=1; }if !found { return false; }i+=1;
    }
    true
}
fn sort_points<const N:usize>(values:&BoundedList<Vec3,N>) -> BoundedList<Vec3,N> {
    let mut result=*values;let mut i=1;
    while i<result.len() {
        let point=result.get(i);assert!(point.is_some());
        if let Some(point)=point { let mut j=i;
            while j>0 { let previous=result.get(j-1);assert!(previous.is_some());
                if let Some(previous)=previous {
                    if (previous.x,previous.y,previous.z)<=(point.x,point.y,point.z) { break; }
                    assert!(result.set(j,previous));
                }j-=1;
            }assert!(result.set(j,point));
        }i+=1;
    }result
}
// helpers.py:129..150 compares each rounded coordinate only for unequal
// points, then checks list lengths. Preserve that order and all three axes.
fn assert_points_rounded<const A:usize,const B:usize>(first:&BoundedList<Vec3,A>,second:&BoundedList<Vec3,B>,places:u32) {
    let limit=first.len().min(second.len());
    let mut i=0_usize;
    while i<limit {
        let left=first.get(i); let right=second.get(i);
        assert!(left.is_some() && right.is_some());
        if let (Some(left),Some(right))=(left,right) {
            if left!=right {
                assert!(python_rounded_equal(left.x,right.x,places)==Some(true));
                assert!(python_rounded_equal(left.y,right.y,places)==Some(true));
                assert!(python_rounded_equal(left.z,right.z,places)==Some(true));
            }
        }
        i=i.saturating_add(1);
    }
    assert!(first.len()==second.len());
}

fn id_list(values: &[i64]) -> BoundedList<i64, 8> {
    let mut result = BoundedList::new();
    let mut i = 0_usize;
    while i < values.len() {
        if let Some(value) = values.get(i) { assert!(result.push(*value)); }
        i = i.saturating_add(1);
    }
    result
}
fn ids_contain(values: &BoundedList<i64, 8>, id: i64) -> bool {
    let mut i = 0_usize;
    while i < values.len() {
        if values.get(i) == Some(id) { return true; }
        i = i.saturating_add(1);
    }
    false
}
fn distance_list(values:&[(f64,i64)]) -> BoundedList<(f64,i64),8> {
    let mut result=BoundedList::new(); let mut i=0_usize;
    while i<values.len() {
        if let Some(value)=values.get(i) { assert!(result.push(*value)); }
        i=i.saturating_add(1);
    }
    result
}
fn query_ids(values:&BoundedList<(f64,i64),8>) -> BoundedList<i64,8> {
    let mut result=BoundedList::new(); let mut i=0_usize;
    while i<values.len() {
        if let Some((_,id))=values.get(i) { assert!(result.push(id)); }
        i=i.saturating_add(1);
    }
    result
}
fn id_sets_equal(left:&BoundedList<i64,8>,right:&BoundedList<i64,8>) -> bool {
    let mut i=0_usize;
    while i<left.len() {
        if let Some(id)=left.get(i) { if !ids_contain(right,id) { return false; } }
        i=i.saturating_add(1);
    }
    i=0;
    while i<right.len() {
        if let Some(id)=right.get(i) { if !ids_contain(left,id) { return false; } }
        i=i.saturating_add(1);
    }
    true
}

// original-case: python/destiny/test/ballpark/evolve/test_stop.py::TestStop::test_stopped_ball_is_stopped
// original-source-sha256: 85a31d0d1ec984a5ae1dc2feef72b73d1eff97616f6599911787e58db20e89de
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn stop__TestStop__test_stopped_ball_is_stopped() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id = 1_i64;
    let stopped_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit {
        id: next_space_object_id,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        ..OriginalBallInit::default()
    });
    assert!(park.contains(stopped_ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(stopped_ball, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(
        stopped_ball,
        |b| b.max_velocity = attribute_value_8,
        OriginalBallAttribute::MaxSpeed,
    );
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(
        stopped_ball,
        |b| b.agility = attribute_value_10,
        OriginalBallAttribute::Agility,
    );
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(
        stopped_ball,
        |b| b.mass = attribute_value_12,
        OriginalBallAttribute::Mass,
    );
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(
        stopped_ball,
        |b| b.radius = attribute_value_14,
        OriginalBallAttribute::Radius,
    );
    let attribute_value_16 = true;
    park.write_attribute_notified(
        stopped_ball,
        |b| b.is_massive = attribute_value_16,
        OriginalBallAttribute::Massive,
    );
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(
        stopped_ball,
        |b| b.speed_fraction = attribute_value_18,
        OriginalBallAttribute::SpeedFraction,
    );
    let mut loop_iteration_20 = 0_i64;
    let mut i = 0_i64;
    while loop_iteration_20 < 10_i64 {
        i = loop_iteration_20;
        assert!(park.evolve_single_body_linear() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_stop.py:11
        assert!(park.retained(stopped_ball).position.x == 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_stop.py:12
        assert!(park.retained(stopped_ball).position.y == 0.0_f64);
        // original-assertion: python/destiny/test/ballpark/evolve/test_stop.py:13
        assert!(park.retained(stopped_ball).position.z == 0.0_f64);
        loop_iteration_20 = loop_iteration_20.saturating_add(1);
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original park projection scenario and teardown completed"
    );
}

// original-case: python/destiny/test/ballpark/evolve/test_stop.py::TestStop::test_stop_ball_in_goto_mode
// original-source-sha256: 85a31d0d1ec984a5ae1dc2feef72b73d1eff97616f6599911787e58db20e89de
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn stop__TestStop__test_stop_ball_in_goto_mode() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let stopped_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(stopped_ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(stopped_ball, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(stopped_ball, |b| b.max_velocity = attribute_value_8, OriginalBallAttribute::MaxSpeed);
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(stopped_ball, |b| b.agility = attribute_value_10, OriginalBallAttribute::Agility);
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(stopped_ball, |b| b.mass = attribute_value_12, OriginalBallAttribute::Mass);
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(stopped_ball, |b| b.radius = attribute_value_14, OriginalBallAttribute::Radius);
    let attribute_value_16 = true;
    park.write_attribute_notified(stopped_ball, |b| b.is_massive = attribute_value_16, OriginalBallAttribute::Massive);
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(stopped_ball, |b| b.speed_fraction = attribute_value_18, OriginalBallAttribute::SpeedFraction);
    let mut stopped_velocity_list=BoundedList::<Vec3,10>::new();
    let attribute_value_21 = 10.0_f64;
    park.write_attribute_notified(stopped_ball, |b| b.velocity.x = attribute_value_21, OriginalBallAttribute::Velocity);
    let attribute_value_23 = 20.0_f64;
    park.write_attribute_notified(stopped_ball, |b| b.velocity.y = attribute_value_23, OriginalBallAttribute::Velocity);
    let attribute_value_25 = 30.0_f64;
    park.write_attribute_notified(stopped_ball, |b| b.velocity.z = attribute_value_25, OriginalBallAttribute::Velocity);
    park.stop(stopped_ball);
    let mut loop_iteration_28=0_i64;
    let mut i=0_i64;
    while loop_iteration_28 < 10_i64 {
        i=loop_iteration_28;
        assert!(park.evolve_single_body_linear() == Ok(())); // original call must not raise
        assert!(stopped_velocity_list.push(Vec3 { x: park.retained(stopped_ball).velocity.x, y: park.retained(stopped_ball).velocity.y, z: park.retained(stopped_ball).velocity.z })); // every original trace element retained
        loop_iteration_28=loop_iteration_28.saturating_add(1);
    }
    let expected_location_list = point_list::<10>(&[Vec3 { x: 9.180806045144438_f64, y: 15.959158171933325_f64, z: 27.542418135433316_f64 }, Vec3 { x: 8.428719963856068_f64, y: 12.73473647783931_f64, z: 25.286159891568197_f64 }, Vec3 { x: 7.738244319699939_f64, y: 10.161783686386327_f64, z: 23.21473295909981_f64 }, Vec3 { x: 7.104332022910581_f64, y: 8.108675658000553_f64, z: 21.312996068731735_f64 }, Vec3 { x: 6.522349438265067_f64, y: 6.470381869546817_f64, z: 19.567048314795194_f64 }, Vec3 { x: 5.9880425151368355_f64, y: 5.163092384445365_f64, z: 17.9641275454105_f64 }, Vec3 { x: 5.497505692155016_f64, y: 4.119930400983397_f64, z: 16.49251707646504_f64 }, Vec3 { x: 5.047153349175273_f64, y: 3.2875310463325356_f64, z: 15.141460047525813_f64 }, Vec3 { x: 4.633693597887935_f64, y: 2.6233113981781195_f64, z: 13.901080793663798_f64 }, Vec3 { x: 4.254104219483663_f64, y: 2.0932920768880083_f64, z: 12.762312658450982_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_stop.py:42
    assert_points_rounded(&stopped_velocity_list, &expected_location_list, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_goto.py::TestGoto::test_goto_direction
// original-source-sha256: db09fe76bbe6f2f888e54d54ddbc59a58e4a319a60736a0faeaa7fc0eb79b172
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn goto__TestGoto__test_goto_direction() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let mut locations=BoundedList::<Vec3,10>::new();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let x = 10.0_f64;
    let y = 20.0_f64;
    let z = 30.0_f64;
    park.goto_direction(ball, Vec3 { x: x, y: y, z: z });
    let mut loop_iteration_25=0_i64;
    let mut i=0_i64;
    while loop_iteration_25 < 10_i64 {
        i=loop_iteration_25;
        assert!(park.evolve_single_body_linear() == Ok(())); // original call must not raise
        assert!(locations.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_25=loop_iteration_25.saturating_add(1);
    }
    let expected_locations = point_list::<10>(&[Vec3 { x: 0.10547716886968704_f64, y: 0.21095433773937408_f64, z: 0.31643150660906116_f64 }, Vec3 { x: 0.4103055632728434_f64, y: 0.8206111265456868_f64, z: 1.2309166898185293_f64 }, Vec3 { x: 0.8981544513244618_f64, y: 1.7963089026489236_f64, z: 2.6944633539733838_f64 }, Vec3 { x: 1.5540309048233936_f64, y: 3.108061809646787_f64, z: 4.662092714470173_f64 }, Vec3 { x: 2.364170207183284_f64, y: 4.728340414366568_f64, z: 7.0925106215498435_f64 }, Vec3 { x: 3.315935239079564_f64, y: 6.631870478159128_f64, z: 9.947805717238685_f64 }, Vec3 { x: 4.397724106363411_f64, y: 8.795448212726821_f64, z: 13.19317231909023_f64 }, Vec3 { x: 5.598885335041164_f64, y: 11.197770670082328_f64, z: 16.796656005123484_f64 }, Vec3 { x: 6.909640013429747_f64, y: 13.819280026859493_f64, z: 20.728920040289236_f64 }, Vec3 { x: 8.321010312379675_f64, y: 16.64202062475935_f64, z: 24.96303093713902_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_goto.py:34
    assert_points_rounded(&locations, &expected_locations, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_goto.py::TestGoto::test_goto_point
// original-source-sha256: db09fe76bbe6f2f888e54d54ddbc59a58e4a319a60736a0faeaa7fc0eb79b172
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(19))]
#[cfg_attr(test, test)]
fn goto__TestGoto__test_goto_point() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let mut locations=BoundedList::<Vec3,18>::new();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let x = 10.0_f64;
    let y = 20.0_f64;
    let z = 30.0_f64;
    park.goto_point(ball, Vec3 { x: x, y: y, z: z });
    let mut loop_iteration_25=0_i64;
    let mut i=0_i64;
    while loop_iteration_25 < 18_i64 {
        i=loop_iteration_25;
        assert!(park.evolve_single_body_linear() == Ok(())); // original call must not raise
        assert!(locations.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_25=loop_iteration_25.saturating_add(1);
    }
    let expected_locations = point_list::<18>(&[Vec3 { x: 0.10547716886968671_f64, y: 0.21095433773937342_f64, z: 0.31643150660906116_f64 }, Vec3 { x: 0.4103055632728427_f64, y: 0.8206111265456854_f64, z: 1.2309166898185306_f64 }, Vec3 { x: 0.898154451324462_f64, y: 1.796308902648924_f64, z: 2.694463353973384_f64 }, Vec3 { x: 1.5540309048233925_f64, y: 3.108061809646785_f64, z: 4.662092714470175_f64 }, Vec3 { x: 2.364170207183283_f64, y: 4.728340414366566_f64, z: 7.092510621549848_f64 }, Vec3 { x: 3.3159352390795633_f64, y: 6.631870478159127_f64, z: 9.947805717238683_f64 }, Vec3 { x: 4.397724106363411_f64, y: 8.795448212726821_f64, z: 13.193172319090227_f64 }, Vec3 { x: 5.598885335041161_f64, y: 11.197770670082322_f64, z: 16.79665600512348_f64 }, Vec3 { x: 6.909640013429744_f64, y: 13.819280026859488_f64, z: 20.728920040289225_f64 }, Vec3 { x: 8.321010312379672_f64, y: 16.642020624759343_f64, z: 24.96303093713901_f64 }, Vec3 { x: 9.739446987259553_f64, y: 19.478893974519107_f64, z: 29.218340961778658_f64 }, Vec3 { x: 11.061301666724546_f64, y: 22.122603333449092_f64, z: 33.18390500017364_f64 }, Vec3 { x: 12.271662033494039_f64, y: 24.543324066988077_f64, z: 36.814986100482116_f64 }, Vec3 { x: 13.312148751580072_f64, y: 26.624297503160143_f64, z: 39.93644625474022_f64 }, Vec3 { x: 14.096228724360666_f64, y: 28.19245744872133_f64, z: 42.288686173082_f64 }, Vec3 { x: 14.608085488313941_f64, y: 29.216170976627883_f64, z: 43.82425646494182_f64 }, Vec3 { x: 14.870019404144168_f64, y: 29.740038808288336_f64, z: 44.610058212432506_f64 }, Vec3 { x: 14.902504000487617_f64, y: 29.805008000975235_f64, z: 44.70751200146285_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_goto.py:68
    assert_points_rounded(&locations, &expected_locations, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_minis.py::TestMiniCapsule::test_add
// original-source-sha256: c26676e43f9557c49b25ed404895c0ba0bfd5ab5d25b4b34b7d2037c527d4ebe
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn minis__TestMiniCapsule__test_add() {
    let mut park = OriginalParkProjection::default();
    let parent = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(parent) && !park.scope_error);
    assert!(park.add_mini_capsule(parent, Vec3 { x: -(1.0_f64), y: -(2.0_f64), z: -(3.0_f64) }, Vec3 { x: 1.0_f64, y: 2.0_f64, z: 3.0_f64 }, (10.0_f64) as f32) == Ok(())); // original call must not raise
    let child_access_3 = (park.retained(parent).mini_capsules).get(0);
    assert!(child_access_3.is_some()); // original list index must exist
    let mini_capsule = child_access_3.unwrap_or(OriginalMiniCapsule { id: 0, parent_id: 0, a: Vec3::ZERO, b: Vec3::ZERO, radius: 0.0 });
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:11
    assert!(mini_capsule.a.x == -(1.0_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:12
    assert!(mini_capsule.a.y == -(2.0_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:13
    assert!(mini_capsule.a.z == -(3.0_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:14
    assert!(mini_capsule.b.x == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:15
    assert!(mini_capsule.b.y == 2.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:16
    assert!(mini_capsule.b.z == 3.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_minis.py::TestMiniBox::test_add
// original-source-sha256: c26676e43f9557c49b25ed404895c0ba0bfd5ab5d25b4b34b7d2037c527d4ebe
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn minis__TestMiniBox__test_add() {
    let mut park = OriginalParkProjection::default();
    let parent = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(parent) && !park.scope_error);
    let attribute_value_2 = 100.0_f64;
    park.write_attribute_notified(parent, |b| b.position.x = attribute_value_2, OriginalBallAttribute::Position);
    let attribute_value_4 = 200.0_f64;
    park.write_attribute_notified(parent, |b| b.position.y = attribute_value_4, OriginalBallAttribute::Position);
    let attribute_value_6 = 300.0_f64;
    park.write_attribute_notified(parent, |b| b.position.z = attribute_value_6, OriginalBallAttribute::Position);
    assert!(park.add_mini_box(parent, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 }) == Ok(())); // original call must not raise
    let child_access_9 = (park.retained(parent).mini_boxes).get(0);
    assert!(child_access_9.is_some()); // original list index must exist
    let mini_box = child_access_9.unwrap_or(OriginalMiniBox { id: 0, parent_id: 0, corner: Vec3::ZERO, x: Vec3::ZERO, y: Vec3::ZERO, z: Vec3::ZERO });
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:33
    assert!(mini_box.corner.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:34
    assert!(mini_box.corner.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:35
    assert!(mini_box.corner.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:37
    assert!(mini_box.x.x == 100.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:38
    assert!(mini_box.x.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:39
    assert!(mini_box.x.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:41
    assert!(mini_box.y.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:42
    assert!(mini_box.y.y == 100.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:43
    assert!(mini_box.y.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:45
    assert!(mini_box.z.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:46
    assert!(mini_box.z.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:47
    assert!(mini_box.z.z == 100.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_minis.py::TestMiniBox::test_add_with_rotation
// original-source-sha256: c26676e43f9557c49b25ed404895c0ba0bfd5ab5d25b4b34b7d2037c527d4ebe
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn minis__TestMiniBox__test_add_with_rotation() {
    let mut park = OriginalParkProjection::default();
    let parent = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(parent) && !park.scope_error);
    let attribute_value_2 = 100.0_f64;
    park.write_attribute_notified(parent, |b| b.position.x = attribute_value_2, OriginalBallAttribute::Position);
    let attribute_value_4 = 200.0_f64;
    park.write_attribute_notified(parent, |b| b.position.y = attribute_value_4, OriginalBallAttribute::Position);
    let attribute_value_6 = 300.0_f64;
    park.write_attribute_notified(parent, |b| b.position.z = attribute_value_6, OriginalBallAttribute::Position);
    assert!(park.add_mini_box(parent, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 1.0_f64, y: 1.0_f64, z: 1.0_f64 }, Vec3 { x: -(1.0_f64), y: 0.0_f64, z: 1.0_f64 }, Vec3 { x: 1.0_f64, y: -(2.0_f64), z: 1.0_f64 }) == Ok(())); // original call must not raise
    let child_access_9 = (park.retained(parent).mini_boxes).get(0);
    assert!(child_access_9.is_some()); // original list index must exist
    let mini_box = child_access_9.unwrap_or(OriginalMiniBox { id: 0, parent_id: 0, corner: Vec3::ZERO, x: Vec3::ZERO, y: Vec3::ZERO, z: Vec3::ZERO });
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:63
    assert!(mini_box.corner.x == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:64
    assert!(mini_box.corner.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:65
    assert!(mini_box.corner.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:67
    assert!(mini_box.x.x == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:68
    assert!(mini_box.x.y == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:69
    assert!(mini_box.x.z == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:71
    assert!(mini_box.y.x == -(1.0_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:72
    assert!(mini_box.y.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:73
    assert!(mini_box.y.z == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:75
    assert!(mini_box.z.x == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:76
    assert!(mini_box.z.y == -(2.0_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:77
    assert!(mini_box.z.z == 1.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_minis.py::TestMiniBall::test_add
// original-source-sha256: c26676e43f9557c49b25ed404895c0ba0bfd5ab5d25b4b34b7d2037c527d4ebe
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn minis__TestMiniBall__test_add() {
    let mut park = OriginalParkProjection::default();
    let parent = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(parent) && !park.scope_error);
    assert!(park.add_mini_ball(parent, Vec3 { x: 1.0_f64, y: 2.0_f64, z: 3.0_f64 }, (10.0_f64) as f32) == Ok(())); // original call must not raise
    let child_access_3 = (park.retained(parent).mini_balls).get(0);
    assert!(child_access_3.is_some()); // original list index must exist
    let mini_ball = child_access_3.unwrap_or(OriginalMiniBall { id: 0, position: Vec3::ZERO, radius: 0.0 });
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:85
    assert!(mini_ball.position.x == 1.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:86
    assert!(mini_ball.position.y == 2.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:87
    assert!(mini_ball.position.z == 3.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCollision::test_stopped_balls_with_same_location
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCollision__test_stopped_balls_with_same_location() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.max_velocity = attribute_value_8, OriginalBallAttribute::MaxSpeed);
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.agility = attribute_value_10, OriginalBallAttribute::Agility);
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(ball_a, |b| b.mass = attribute_value_12, OriginalBallAttribute::Mass);
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.radius = attribute_value_14, OriginalBallAttribute::Radius);
    let attribute_value_16 = true;
    park.write_attribute_notified(ball_a, |b| b.is_massive = attribute_value_16, OriginalBallAttribute::Massive);
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.speed_fraction = attribute_value_18, OriginalBallAttribute::SpeedFraction);
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.max_velocity = attribute_value_26, OriginalBallAttribute::MaxSpeed);
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.agility = attribute_value_28, OriginalBallAttribute::Agility);
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(ball_b, |b| b.mass = attribute_value_30, OriginalBallAttribute::Mass);
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.radius = attribute_value_32, OriginalBallAttribute::Radius);
    let attribute_value_34 = true;
    park.write_attribute_notified(ball_b, |b| b.is_massive = attribute_value_34, OriginalBallAttribute::Massive);
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.speed_fraction = attribute_value_36, OriginalBallAttribute::SpeedFraction);
    let mut a_coordinates=BoundedList::<Vec3,10>::new();
    let mut b_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_40=0_i64;
    let mut i=0_i64;
    while loop_iteration_40 < 10_i64 {
        i=loop_iteration_40;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(a_coordinates.push(Vec3 { x: park.retained(ball_a).position.x, y: park.retained(ball_a).position.y, z: park.retained(ball_a).position.z })); // every original trace element retained
        assert!(b_coordinates.push(Vec3 { x: park.retained(ball_b).position.x, y: park.retained(ball_b).position.y, z: park.retained(ball_b).position.z })); // every original trace element retained
        loop_iteration_40=loop_iteration_40.saturating_add(1);
    }
    let expected_a_coordinates = point_list::<10>(&[Vec3 { x: 1.121144335565424_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 3.2401005010556143_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 5.185473058408457_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 6.9714818718687654_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 8.61118192299853_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 10.116558737162107_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 11.498615992731432_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 12.76745595339809_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 13.93235331151902_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 15.001822982259961_f64, y: 0.0_f64, z: 0.0_f64 }]);
    let expected_b_coordinates = point_list::<10>(&[Vec3 { x: -(1.121144335565424_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(3.2401005010556143_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(5.185473058408457_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(6.9714818718687654_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(8.61118192299853_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(10.116558737162107_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(11.498615992731432_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(12.76745595339809_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(13.93235331151902_f64), y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: -(15.001822982259961_f64), y: 0.0_f64, z: 0.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:49
    assert_points_rounded(&a_coordinates, &expected_a_coordinates, 4);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:50
    assert_points_rounded(&b_coordinates, &expected_b_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCollision::test_stopped_balls_intersecting
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCollision__test_stopped_balls_intersecting() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 1.9_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (10.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.max_velocity = attribute_value_8, OriginalBallAttribute::MaxSpeed);
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.agility = attribute_value_10, OriginalBallAttribute::Agility);
    let attribute_value_12 = 13000000.0_f64;
    park.write_attribute_notified(ball_a, |b| b.mass = attribute_value_12, OriginalBallAttribute::Mass);
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.radius = attribute_value_14, OriginalBallAttribute::Radius);
    let attribute_value_16 = true;
    park.write_attribute_notified(ball_a, |b| b.is_massive = attribute_value_16, OriginalBallAttribute::Massive);
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.speed_fraction = attribute_value_18, OriginalBallAttribute::SpeedFraction);
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (10.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.max_velocity = attribute_value_26, OriginalBallAttribute::MaxSpeed);
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.agility = attribute_value_28, OriginalBallAttribute::Agility);
    let attribute_value_30 = 13000000.0_f64;
    park.write_attribute_notified(ball_b, |b| b.mass = attribute_value_30, OriginalBallAttribute::Mass);
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.radius = attribute_value_32, OriginalBallAttribute::Radius);
    let attribute_value_34 = true;
    park.write_attribute_notified(ball_b, |b| b.is_massive = attribute_value_34, OriginalBallAttribute::Massive);
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.speed_fraction = attribute_value_36, OriginalBallAttribute::SpeedFraction);
    let mut a_coordinates=BoundedList::<Vec3,10>::new();
    let mut b_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_40=0_i64;
    let mut i=0_i64;
    while loop_iteration_40 < 10_i64 {
        i=loop_iteration_40;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(a_coordinates.push(Vec3 { x: park.retained(ball_a).position.x, y: park.retained(ball_a).position.y, z: park.retained(ball_a).position.z })); // every original trace element retained
        assert!(b_coordinates.push(Vec3 { x: park.retained(ball_b).position.x, y: park.retained(ball_b).position.y, z: park.retained(ball_b).position.z })); // every original trace element retained
        loop_iteration_40=loop_iteration_40.saturating_add(1);
    }
    let expected_a_coordinates = point_list::<10>(&[Vec3 { x: 0.0_f64, y: 2.5951094880505643_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 3.736969417977334_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 4.648125579572035_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 5.375189844683108_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 5.955356525086511_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 6.418305116018683_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 6.787718605426691_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 7.082495020842102_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 7.317714192790605_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 7.505409191300473_f64, z: 0.0_f64 }]);
    let expected_b_coordinates = point_list::<10>(&[Vec3 { x: 0.0_f64, y: -(0.6951094880505645_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(1.836969417977334_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(2.7481255795720347_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(3.4751898446831078_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(4.055356525086512_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(4.5183051160186825_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(4.8877186054266915_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(5.182495020842103_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(5.417714192790607_f64), z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: -(5.605409191300473_f64), z: 0.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:91
    assert_points_rounded(&a_coordinates, &expected_a_coordinates, 4);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:92
    assert_points_rounded(&b_coordinates, &expected_b_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCollision::test_goto_collision
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(21))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCollision__test_goto_collision() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let mut a_coordinates=BoundedList::<Vec3,20>::new();
    let mut b_coordinates=BoundedList::<Vec3,20>::new();
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_10 = (10.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.max_velocity = attribute_value_10, OriginalBallAttribute::MaxSpeed);
    let attribute_value_12 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.agility = attribute_value_12, OriginalBallAttribute::Agility);
    let attribute_value_14 = 13000000.0_f64;
    park.write_attribute_notified(ball_a, |b| b.mass = attribute_value_14, OriginalBallAttribute::Mass);
    let attribute_value_16 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.radius = attribute_value_16, OriginalBallAttribute::Radius);
    let attribute_value_18 = true;
    park.write_attribute_notified(ball_a, |b| b.is_massive = attribute_value_18, OriginalBallAttribute::Massive);
    let attribute_value_20 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.speed_fraction = attribute_value_20, OriginalBallAttribute::SpeedFraction);
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 10.0_f64, y: 10.0_f64, z: 10.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_28 = (10.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.max_velocity = attribute_value_28, OriginalBallAttribute::MaxSpeed);
    let attribute_value_30 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.agility = attribute_value_30, OriginalBallAttribute::Agility);
    let attribute_value_32 = 13000000.0_f64;
    park.write_attribute_notified(ball_b, |b| b.mass = attribute_value_32, OriginalBallAttribute::Mass);
    let attribute_value_34 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.radius = attribute_value_34, OriginalBallAttribute::Radius);
    let attribute_value_36 = true;
    park.write_attribute_notified(ball_b, |b| b.is_massive = attribute_value_36, OriginalBallAttribute::Massive);
    let attribute_value_38 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.speed_fraction = attribute_value_38, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball_a, Vec3 { x: 10.0_f64, y: 10.0_f64, z: 10.0_f64 });
    park.goto_point(ball_b, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 });
    let mut loop_iteration_42=0_i64;
    let mut i=0_i64;
    while loop_iteration_42 < 20_i64 {
        i=loop_iteration_42;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(a_coordinates.push(Vec3 { x: park.retained(ball_a).position.x, y: park.retained(ball_a).position.y, z: park.retained(ball_a).position.z })); // every original trace element retained
        assert!(b_coordinates.push(Vec3 { x: park.retained(ball_b).position.x, y: park.retained(ball_b).position.y, z: park.retained(ball_b).position.z })); // every original trace element retained
        loop_iteration_42=loop_iteration_42.saturating_add(1);
    }
    let expected_a_coordinates = point_list::<20>(&[Vec3 { x: 0.22785672701553972_f64, y: 0.22785672701553972_f64, z: 0.22785672701553972_f64 }, Vec3 { x: 0.8863613208951594_f64, y: 0.8863613208951594_f64, z: 0.8863613208951594_f64 }, Vec3 { x: 1.9402353687182086_f64, y: 1.9402353687182086_f64, z: 1.9402353687182086_f64 }, Vec3 { x: 3.3570904438241858_f64, y: 3.3570904438241858_f64, z: 3.3570904438241858_f64 }, Vec3 { x: 3.745389410116848_f64, y: 3.745389410116848_f64, z: 3.745389410116848_f64 }, Vec3 { x: 3.2276392358401793_f64, y: 3.2276392358401793_f64, z: 3.2276392358401793_f64 }, Vec3 { x: 3.2016165950513056_f64, y: 3.2016165950513056_f64, z: 3.2016165950513056_f64 }, Vec3 { x: 3.627039465463305_f64, y: 3.627039465463305_f64, z: 3.627039465463305_f64 }, Vec3 { x: 3.8640182292625984_f64, y: 3.8640182292625984_f64, z: 3.8640182292625984_f64 }, Vec3 { x: 3.9449236847646176_f64, y: 3.9449236847646176_f64, z: 3.9449236847646176_f64 }, Vec3 { x: 4.014836947038442_f64, y: 4.014836947038442_f64, z: 4.014836947038442_f64 }, Vec3 { x: 4.087400730922078_f64, y: 4.087400730922078_f64, z: 4.087400730922078_f64 }, Vec3 { x: 4.152795682608074_f64, y: 4.152795682608074_f64, z: 4.152795682608074_f64 }, Vec3 { x: 4.2242631173879985_f64, y: 4.2242631173879985_f64, z: 4.2242631173879985_f64 }, Vec3 { x: 4.289057014884843_f64, y: 4.289057014884843_f64, z: 4.289057014884843_f64 }, Vec3 { x: 4.360366727534607_f64, y: 4.360366727534607_f64, z: 4.360366727534607_f64 }, Vec3 { x: 4.425074016382081_f64, y: 4.425074016382081_f64, z: 4.425074016382081_f64 }, Vec3 { x: 4.496360773851808_f64, y: 4.496360773851808_f64, z: 4.496360773851808_f64 }, Vec3 { x: 4.561055454563248_f64, y: 4.561055454563248_f64, z: 4.561055454563248_f64 }, Vec3 { x: 4.624809527703686_f64, y: 4.624809527703686_f64, z: 4.624809527703686_f64 }]);
    let expected_b_coordinates = point_list::<20>(&[Vec3 { x: 9.772143272984462_f64, y: 9.772143272984462_f64, z: 9.772143272984462_f64 }, Vec3 { x: 9.113638679104842_f64, y: 9.113638679104842_f64, z: 9.113638679104842_f64 }, Vec3 { x: 8.059764631281793_f64, y: 8.059764631281793_f64, z: 8.059764631281793_f64 }, Vec3 { x: 6.642909556175815_f64, y: 6.642909556175815_f64, z: 6.642909556175815_f64 }, Vec3 { x: 6.254610589883153_f64, y: 6.254610589883153_f64, z: 6.254610589883153_f64 }, Vec3 { x: 6.772360764159822_f64, y: 6.772360764159822_f64, z: 6.772360764159822_f64 }, Vec3 { x: 6.7983834049486935_f64, y: 6.7983834049486935_f64, z: 6.7983834049486935_f64 }, Vec3 { x: 6.372960534536694_f64, y: 6.372960534536694_f64, z: 6.372960534536694_f64 }, Vec3 { x: 6.1359817707374_f64, y: 6.1359817707374_f64, z: 6.1359817707374_f64 }, Vec3 { x: 6.055076315235381_f64, y: 6.055076315235381_f64, z: 6.055076315235381_f64 }, Vec3 { x: 5.985163052961557_f64, y: 5.985163052961557_f64, z: 5.985163052961557_f64 }, Vec3 { x: 5.912599269077921_f64, y: 5.912599269077921_f64, z: 5.912599269077921_f64 }, Vec3 { x: 5.847204317391924_f64, y: 5.847204317391924_f64, z: 5.847204317391924_f64 }, Vec3 { x: 5.775736882612_f64, y: 5.775736882612_f64, z: 5.775736882612_f64 }, Vec3 { x: 5.710942985115155_f64, y: 5.710942985115155_f64, z: 5.710942985115155_f64 }, Vec3 { x: 5.639633272465392_f64, y: 5.639633272465392_f64, z: 5.639633272465392_f64 }, Vec3 { x: 5.574925983617919_f64, y: 5.574925983617919_f64, z: 5.574925983617919_f64 }, Vec3 { x: 5.503639226148192_f64, y: 5.503639226148192_f64, z: 5.503639226148192_f64 }, Vec3 { x: 5.438944545436752_f64, y: 5.438944545436752_f64, z: 5.438944545436752_f64 }, Vec3 { x: 5.375190472296313_f64, y: 5.375190472296313_f64, z: 5.375190472296313_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:155
    assert_points_rounded(&a_coordinates, &expected_a_coordinates, 4);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:156
    assert_points_rounded(&b_coordinates, &expected_b_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_follow.py::TestFollow::test_follow_stopped_ball
// original-source-sha256: c343e312ca36a9b7ecc3d162889086cc965803fc4d364e38a82eb4da80e5b1b9
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn follow__TestFollow__test_follow_stopped_ball() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    // unittest maxDiff=None changes failure diagnostics, not simulation state.
    let follower = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(follower) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(follower, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(follower, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(follower, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(follower, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(follower, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let leader = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 100.0_f64, y: 200.0_f64, z: 300.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(leader) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(leader, true);
    let attribute_value_27 = (10.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.max_velocity = attribute_value_27, OriginalBallAttribute::MaxSpeed);
    let attribute_value_29 = (0.9_f64) as f32;
    park.write_attribute_notified(leader, |b| b.agility = attribute_value_29, OriginalBallAttribute::Agility);
    let attribute_value_31 = 13000000.0_f64;
    park.write_attribute_notified(leader, |b| b.mass = attribute_value_31, OriginalBallAttribute::Mass);
    let attribute_value_33 = (2.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.radius = attribute_value_33, OriginalBallAttribute::Radius);
    let attribute_value_35 = true;
    park.write_attribute_notified(leader, |b| b.is_massive = attribute_value_35, OriginalBallAttribute::Massive);
    let attribute_value_37 = (0.95_f64) as f32;
    park.write_attribute_notified(leader, |b| b.speed_fraction = attribute_value_37, OriginalBallAttribute::SpeedFraction);
    park.follow(follower, leader, (1.0_f64) as f32, false);
    let mut follower_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_41=0_i64;
    let mut i=0_i64;
    while loop_iteration_41 < 10_i64 {
        i=loop_iteration_41;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(follower_coordinates.push(Vec3 { x: park.retained(follower).position.x, y: park.retained(follower).position.y, z: park.retained(follower).position.z })); // every original trace element retained
        loop_iteration_41=loop_iteration_41.saturating_add(1);
    }
    let expected_follower_coordinates = point_list::<10>(&[Vec3 { x: 0.10547716886968704_f64, y: 0.21095433773937408_f64, z: 0.3164315066090625_f64 }, Vec3 { x: 0.41030556327284373_f64, y: 0.8206111265456875_f64, z: 1.2309166898185306_f64 }, Vec3 { x: 0.8981544513244621_f64, y: 1.7963089026489243_f64, z: 2.694463353973384_f64 }, Vec3 { x: 1.5540309048233925_f64, y: 3.108061809646785_f64, z: 4.662092714470175_f64 }, Vec3 { x: 2.364170207183283_f64, y: 4.728340414366566_f64, z: 7.092510621549848_f64 }, Vec3 { x: 3.3159352390795633_f64, y: 6.631870478159127_f64, z: 9.94780571723868_f64 }, Vec3 { x: 4.397724106363411_f64, y: 8.795448212726821_f64, z: 13.193172319090223_f64 }, Vec3 { x: 5.59888533504116_f64, y: 11.19777067008232_f64, z: 16.796656005123477_f64 }, Vec3 { x: 6.909640013429743_f64, y: 13.819280026859486_f64, z: 20.72892004028922_f64 }, Vec3 { x: 8.321010312379672_f64, y: 16.642020624759343_f64, z: 24.963030937139006_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_follow.py:101
    assert_points_rounded(&follower_coordinates, &expected_follower_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_follow.py::TestFollow::test_follow_moving_ball
// original-source-sha256: c343e312ca36a9b7ecc3d162889086cc965803fc4d364e38a82eb4da80e5b1b9
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn follow__TestFollow__test_follow_moving_ball() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    // unittest maxDiff=None changes failure diagnostics, not simulation state.
    let follower = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(follower) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(follower, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(follower, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(follower, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(follower, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(follower, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let leader = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(leader) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(leader, true);
    let attribute_value_27 = (10.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.max_velocity = attribute_value_27, OriginalBallAttribute::MaxSpeed);
    let attribute_value_29 = (0.9_f64) as f32;
    park.write_attribute_notified(leader, |b| b.agility = attribute_value_29, OriginalBallAttribute::Agility);
    let attribute_value_31 = 13000000.0_f64;
    park.write_attribute_notified(leader, |b| b.mass = attribute_value_31, OriginalBallAttribute::Mass);
    let attribute_value_33 = (2.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.radius = attribute_value_33, OriginalBallAttribute::Radius);
    let attribute_value_35 = true;
    park.write_attribute_notified(leader, |b| b.is_massive = attribute_value_35, OriginalBallAttribute::Massive);
    let attribute_value_37 = (0.95_f64) as f32;
    park.write_attribute_notified(leader, |b| b.speed_fraction = attribute_value_37, OriginalBallAttribute::SpeedFraction);
    park.follow(follower, leader, (1.0_f64) as f32, false);
    park.goto_point(leader, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 200.0_f64 });
    let mut follower_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_42=0_i64;
    let mut i=0_i64;
    while loop_iteration_42 < 10_i64 {
        i=loop_iteration_42;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(follower_coordinates.push(Vec3 { x: park.retained(follower).position.x, y: park.retained(follower).position.y, z: park.retained(follower).position.z })); // every original trace element retained
        loop_iteration_42=loop_iteration_42.saturating_add(1);
    }
    let expected_follower_coordinates = point_list::<10>(&[Vec3 { x: 0.3946594280372685_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 1.5352202516956626_f64, y: 0.0006394210607791958_f64, z: 0.0012788421215583917_f64 }, Vec3 { x: 3.360538351919784_f64, y: 0.00437328037960026_f64, z: 0.00874656075920052_f64 }, Vec3 { x: 5.814319316187089_f64, y: 0.01591817574944254_f64, z: 0.03183635149888508_f64 }, Vec3 { x: 8.844475669456193_f64, y: 0.042139697272792834_f64, z: 0.08427939454558567_f64 }, Vec3 { x: 12.402376916827746_f64, y: 0.09207177010911027_f64, z: 0.18414354021822055_f64 }, Vec3 { x: 16.441932909387862_f64, y: 0.17705502464375017_f64, z: 0.35411004928750034_f64 }, Vec3 { x: 20.918410162782422_f64, y: 0.31098291710272036_f64, z: 0.6219658342054407_f64 }, Vec3 { x: 25.786827618186656_f64, y: 0.5106319284615736_f64, z: 1.0212638569231471_f64 }, Vec3 { x: 30.99971869544917_f64, y: 0.796013087027414_f64, z: 1.592026174054828_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_follow.py:129
    assert_points_rounded(&follower_coordinates, &expected_follower_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_orbit.py::TestOldOrbit::test_orbit_ball
// original-source-sha256: 813029bedae13ff9928a9055c6f4e59334924c1048d28074be134e8b1b242466
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn orbit__TestOldOrbit__test_orbit_ball() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    // original-assertion: python/destiny/test/ballpark/evolve/test_orbit.py:11
    assert!((park.time) as f64 == 0.0_f64);
    // unittest maxDiff=None changes failure diagnostics, not simulation state.
    next_space_object_id=1_i64; // original global helper reset; IDs seed the orbit plane
    let orbitee = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(orbitee) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(orbitee, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(orbitee, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(orbitee, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(orbitee, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(orbitee, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(orbitee, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(orbitee, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    let orbiter = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 5.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(orbiter) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(orbiter, true);
    let attribute_value_30 = (10.0_f64) as f32;
    park.write_attribute_notified(orbiter, |b| b.max_velocity = attribute_value_30, OriginalBallAttribute::MaxSpeed);
    let attribute_value_32 = (0.9_f64) as f32;
    park.write_attribute_notified(orbiter, |b| b.agility = attribute_value_32, OriginalBallAttribute::Agility);
    let attribute_value_34 = 13000000.0_f64;
    park.write_attribute_notified(orbiter, |b| b.mass = attribute_value_34, OriginalBallAttribute::Mass);
    let attribute_value_36 = (2.0_f64) as f32;
    park.write_attribute_notified(orbiter, |b| b.radius = attribute_value_36, OriginalBallAttribute::Radius);
    let attribute_value_38 = true;
    park.write_attribute_notified(orbiter, |b| b.is_massive = attribute_value_38, OriginalBallAttribute::Massive);
    let attribute_value_40 = (0.95_f64) as f32;
    park.write_attribute_notified(orbiter, |b| b.speed_fraction = attribute_value_40, OriginalBallAttribute::SpeedFraction);
    park.follow(orbiter, orbitee, (1.0_f64) as f32, true);
    let mut orbiter_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_44=0_i64;
    let mut i=0_i64;
    while loop_iteration_44 < 10_i64 {
        i=loop_iteration_44;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(orbiter_coordinates.push(Vec3 { x: park.retained(orbiter).position.x, y: park.retained(orbiter).position.y, z: park.retained(orbiter).position.z })); // every original trace element retained
        loop_iteration_44=loop_iteration_44.saturating_add(1);
    }
    let expected_orbiter_coordinates = point_list::<10>(&[Vec3 { x: 5.0_f64, y: 0.0_f64, z: 0.3946594280372685_f64 }, Vec3 { x: 4.968965087530276_f64, y: -(0.014060509698578017_f64), z: 1.533749375235321_f64 }, Vec3 { x: 4.7948197558341885_f64, y: -(0.09342506603769896_f64), z: 3.3353413772468783_f64 }, Vec3 { x: 4.303462228867957_f64, y: -(0.3182911817228117_f64), z: 5.664861650309143_f64 }, Vec3 { x: 3.341010762290748_f64, y: -(0.7601346870799338_f64), z: 8.325917825178466_f64 }, Vec3 { x: 1.826888038965309_f64, y: -(1.4570571575293851_f64), z: 11.103680716692217_f64 }, Vec3 { x: -(0.25072581197636695_f64), y: -(2.415534207332392_f64), z: 13.808614450861453_f64 }, Vec3 { x: -(2.860769892110699_f64), y: -(3.6221094985835713_f64), z: 16.289471089613752_f64 }, Vec3 { x: -(5.948838546575331_f64), y: -(5.052297043579673_f64), z: 18.42935795458875_f64 }, Vec3 { x: -(9.447930869309866_f64), y: -(6.675507070626672_f64), z: 20.138672333159825_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_orbit.py:36
    assert_points_rounded(&orbiter_coordinates, &expected_orbiter_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCapsuleCollision::test_collision_from_left
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCapsuleCollision__test_collision_from_left() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_capsule(parentBall, Vec3 { x: 100.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 200.0_f64, y: 100.0_f64, z: 0.0_f64 }, (1.0_f64) as f32)==Ok(())); // original fixture call must not raise
    let _original_fixture_capsule_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 90.0_f64, y: 100.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 300.0_f64, y: 100.0_f64, z: 0.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 90.39465942803726_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 91.53522284165427_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 93.36058623726207_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 95.81465121430743_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 96.19428373412633_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 94.74389513390018_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 94.19055573837095_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 94.46077981896904_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 95.48710155358613_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 95.26270051345809_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 93.94466009572491_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 93.5128269995852_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 93.89460365697916_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 95.02333964873613_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 96.8378445177328_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 97.3081193532411_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 96.15310072010158_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 95.43685904668816_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 95.55752570552956_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 96.44654167197064_f64, y: 100.0_f64, z: 0.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:212
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCapsuleCollision::test_collision_from_right
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCapsuleCollision__test_collision_from_right() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_capsule(parentBall, Vec3 { x: 100.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 200.0_f64, y: 100.0_f64, z: 0.0_f64 }, (1.0_f64) as f32)==Ok(())); // original fixture call must not raise
    let _original_fixture_capsule_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 210.0_f64, y: 100.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 209.60534057196273_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 208.46477715834573_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 206.63941376273795_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 204.18534878569255_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 203.80571626587366_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 205.25610486609978_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 205.80944426162898_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 205.53922018103088_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 204.51289844641377_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 204.73729948654176_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 206.05533990427483_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 206.48717300041446_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 206.10539634302043_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 204.9766603512634_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 203.16215548226666_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 202.45322116730054_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 203.45863658621496_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 204.33052522945047_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 204.35275503475415_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 203.59492954049838_f64, y: 100.0_f64, z: 0.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:240
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCapsuleCollision::test_collision_from_top
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCapsuleCollision__test_collision_from_top() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_capsule(parentBall, Vec3 { x: 100.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 200.0_f64, y: 100.0_f64, z: 0.0_f64 }, (1.0_f64) as f32)==Ok(())); // original fixture call must not raise
    let _original_fixture_capsule_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 150.0_f64, y: 110.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 150.0_f64, y: 0.0_f64, z: 0.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 150.0_f64, y: 109.60534057196273_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 108.46477715834571_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 106.63941376273792_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 104.18534878569254_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.56551380937034_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 104.56192118699425_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 104.69846922723862_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 104.04559708722422_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.88074751531305_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 104.12987940232064_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.58036830840118_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.26349609203893_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.2443564826648_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 102.94843147069491_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.20462217476765_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.56098365081996_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.60239882749173_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.8500067373003_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.79396656673549_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 103.76288953683304_f64, z: 0.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:268
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCapsuleCollision::test_collision_from_bottom
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCapsuleCollision__test_collision_from_bottom() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_capsule(parentBall, Vec3 { x: 100.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 200.0_f64, y: 100.0_f64, z: 0.0_f64 }, (1.0_f64) as f32)==Ok(())); // original fixture call must not raise
    let _original_fixture_capsule_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 150.0_f64, y: 90.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 150.0_f64, y: 300.0_f64, z: 0.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 150.0_f64, y: 90.39465942803726_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 91.53522284165427_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 93.36058623726207_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 95.81465121430743_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.43448619062963_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 95.43807881300572_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 95.30153077276132_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 95.95440291277573_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.11925248468687_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 95.87012059767925_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.41963169159868_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.73650390796085_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.75564351733463_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 97.05156852930398_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.79537782523148_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.43901634918006_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.3976011725089_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.14999326270136_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.20603343326782_f64, z: 0.0_f64 }, Vec3 { x: 150.0_f64, y: 96.23711046317295_f64, z: 0.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:296
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleBoxCollision::test_collision_from_left
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleBoxCollision__test_collision_from_left() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_box(parentBall, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 })==Ok(())); // original fixture call must not raise
    let _original_fixture_box_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: -(10.0_f64), y: 50.0_f64, z: 50.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 50.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: -(9.605340571962731_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(8.46477715834572_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(6.639413762737928_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(4.185348785692558_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(3.809252356728292_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(5.266324147762671_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(5.825799251149228_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(5.561208244929905_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(4.540058126643755_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.824325761400378_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.3169360278041253_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.86705381202771_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.5938720325746143_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.46888358211965_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.4545569738496824_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.3691238030650834_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.1985812857514397_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(1.847589525055808_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.2264577298771377_f64), y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: -(2.6823664830129617_f64), y: 50.0_f64, z: 50.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:336
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleBoxCollision::test_collision_from_right
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleBoxCollision__test_collision_from_right() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_box(parentBall, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 })==Ok(())); // original fixture call must not raise
    let _original_fixture_box_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 110.0_f64, y: 50.0_f64, z: 50.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 50.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 109.60534057196273_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 108.46477715834571_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 106.63941376273792_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 104.18534878569254_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 103.8092523567283_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 105.26632414776267_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 105.82579925114923_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 105.5612082449299_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 104.54005812664376_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.82432576140037_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.31693602780412_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.8670538120277_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.59387203257461_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.46888358211964_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.45455697384968_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.36912380306507_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.19858128575139_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 101.84758952505571_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.22645772987704_f64, y: 50.0_f64, z: 50.0_f64 }, Vec3 { x: 102.68236648301296_f64, y: 50.0_f64, z: 50.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:364
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleBoxCollision::test_collision_from_top
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleBoxCollision__test_collision_from_top() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_box(parentBall, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 })==Ok(())); // original fixture call must not raise
    let _original_fixture_box_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 50.0_f64, y: 110.0_f64, z: 50.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 50.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 50.0_f64, y: 109.60534057196273_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 108.46477715834571_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 106.63941376273792_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 104.18534878569254_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 103.8092523567283_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 105.26632414776267_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 105.82579925114923_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 105.5612082449299_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 104.54005812664376_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.82432576140037_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.31693602780412_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.8670538120277_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.59387203257461_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.46888358211964_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.45455697384968_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.36912380306507_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.19858128575139_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 101.84758952505571_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.22645772987704_f64, z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: 102.68236648301296_f64, z: 50.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:392
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleBoxCollision::test_collision_from_bottom
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleBoxCollision__test_collision_from_bottom() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_box(parentBall, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 })==Ok(())); // original fixture call must not raise
    let _original_fixture_box_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 50.0_f64, y: -(50.0_f64), z: 50.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 50.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 50.0_f64, y: -(49.605340571962735_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(48.464777158345726_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(46.63941376273794_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(44.18534878569257_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(41.154085080701805_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(37.59290641863395_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(33.54522311243153_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(29.050889328442953_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(24.1464944038018_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(18.865630299262452_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(13.239137142461129_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(7.295328656421343_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(6.449286068036103_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(10.132042794594934_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(12.73487606904647_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(14.346252567758015_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(15.047391830448245_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(14.912859941240473_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(14.0111145757229_f64), z: 50.0_f64 }, Vec3 { x: 50.0_f64, y: -(12.405005398083203_f64), z: 50.0_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:420
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleBoxCollision::test_collision_from_front
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleBoxCollision__test_collision_from_front() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_box(parentBall, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 })==Ok(())); // original fixture call must not raise
    let _original_fixture_box_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 50.0_f64, y: 50.0_f64, z: 110.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 50.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 50.0_f64, y: 50.0_f64, z: 109.60534057196273_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 108.46477715834571_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 106.63941376273792_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 104.18534878569254_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 103.8092523567283_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 105.26632414776267_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 105.82579925114923_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 105.5612082449299_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 104.54005812664376_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.82432576140037_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.31693602780412_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.8670538120277_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.59387203257461_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.46888358211964_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.45455697384968_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.36912380306507_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.19858128575139_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 101.84758952505571_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.22645772987704_f64 }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 102.68236648301296_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:448
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleBoxCollision::test_collision_from_back
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleBoxCollision__test_collision_from_back() {
    let mut park = OriginalParkProjection::default();
    let parentBall = park.add_nonpositive_time_factor_ball(OriginalBallInit { radius: (500.0_f64) as f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(parentBall) && !park.scope_error);
    assert!(park.add_mini_box(parentBall, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 100.0_f64 })==Ok(())); // original fixture call must not raise
    let _original_fixture_box_return=(); // preserve the original Python None member
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(10.0_f64) }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_12 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_12, OriginalBallAttribute::MaxSpeed);
    let attribute_value_14 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_14, OriginalBallAttribute::Agility);
    let attribute_value_16 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_16, OriginalBallAttribute::Mass);
    let attribute_value_18 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_18, OriginalBallAttribute::Radius);
    let attribute_value_20 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_20, OriginalBallAttribute::Massive);
    let attribute_value_22 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_22, OriginalBallAttribute::SpeedFraction);
    park.goto_point(ball, Vec3 { x: 50.0_f64, y: 50.0_f64, z: 50.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_26=0_i64;
    let mut i=0_i64;
    while loop_iteration_26 < 20_i64 {
        i=loop_iteration_26;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_26=loop_iteration_26.saturating_add(1);
    }
    let coordinates = coordinates;
    let expected_coordinates = point_list::<20>(&[Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(9.605340571962731_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(8.46477715834572_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(6.639413762737928_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(4.185348785692558_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(3.809252356728292_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(5.266324147762671_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(5.825799251149228_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(5.561208244929905_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(4.540058126643755_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.824325761400378_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.3169360278041253_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.86705381202771_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.5938720325746143_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.46888358211965_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.4545569738496824_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.3691238030650834_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.1985812857514397_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(1.847589525055808_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.2264577298771377_f64) }, Vec3 { x: 50.0_f64, y: 50.0_f64, z: -(2.6823664830129617_f64) }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:476
    assert_points_rounded(&coordinates, &expected_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_follow.py::TestFormation::test_formation_follow_stopped_ball
// original-source-sha256: c343e312ca36a9b7ecc3d162889086cc965803fc4d364e38a82eb4da80e5b1b9
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn follow__TestFormation__test_formation_follow_stopped_ball() {
    let mut park = OriginalParkProjection::default();
    park.load_original_formations();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let follower = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(follower) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(follower, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(follower, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(follower, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(follower, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(follower, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let leader = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: -(5.0_f64), y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(leader) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(leader, true);
    let attribute_value_27 = (10.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.max_velocity = attribute_value_27, OriginalBallAttribute::MaxSpeed);
    let attribute_value_29 = (0.9_f64) as f32;
    park.write_attribute_notified(leader, |b| b.agility = attribute_value_29, OriginalBallAttribute::Agility);
    let attribute_value_31 = 13000000.0_f64;
    park.write_attribute_notified(leader, |b| b.mass = attribute_value_31, OriginalBallAttribute::Mass);
    let attribute_value_33 = (2.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.radius = attribute_value_33, OriginalBallAttribute::Radius);
    let attribute_value_35 = true;
    park.write_attribute_notified(leader, |b| b.is_massive = attribute_value_35, OriginalBallAttribute::Massive);
    let attribute_value_37 = (0.95_f64) as f32;
    park.write_attribute_notified(leader, |b| b.speed_fraction = attribute_value_37, OriginalBallAttribute::SpeedFraction);
    park.set_formation(leader, (1.0_f64) as i8);
    let slot = park.reserve_original_formation_slot(leader);
    park.formation_follow(follower, leader, slot);
    let mut follower_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_43=0_i64;
    let mut i=0_i64;
    while loop_iteration_43 < 10_i64 {
        i=loop_iteration_43;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(follower_coordinates.push(Vec3 { x: park.retained(follower).position.x, y: park.retained(follower).position.y, z: park.retained(follower).position.z })); // every original trace element retained
        loop_iteration_43=loop_iteration_43.saturating_add(1);
    }
    let expected_follower_coordinates = point_list::<10>(&[Vec3 { x: 0.34937874984850925_f64, y: 0.0_f64, z: -(0.18354986595744605_f64) }, Vec3 { x: 0.8278427298581066_f64, y: -(0.3459978313028692_f64), z: -(0.584942714251052_f64) }, Vec3 { x: 0.8983700252647063_f64, y: -(1.344611204434962_f64), z: -(1.0160119154083649_f64) }, Vec3 { x: 0.5844941405575219_f64, y: -(2.8926128310477566_f64), z: -(1.5975939889522262_f64) }, Vec3 { x: -(0.08489523178080371_f64), y: -(4.784247012129277_f64), z: -(2.593859202902083_f64) }, Vec3 { x: -(1.0834106420909224_f64), y: -(6.787529292534025_f64), z: -(4.122561424197542_f64) }, Vec3 { x: -(2.3834969832762254_f64), y: -(8.744771342586718_f64), z: -(6.190397079773381_f64) }, Vec3 { x: -(3.958565430928698_f64), y: -(10.569491075166201_f64), z: -(8.76574805927726_f64) }, Vec3 { x: -(5.783982011387288_f64), y: -(12.21826200033105_f64), z: -(11.808792734080857_f64) }, Vec3 { x: -(7.837195912453384_f64), y: -(13.670582854838065_f64), z: -(15.280267553697655_f64) }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_follow.py:42
    assert_points_rounded(&follower_coordinates, &expected_follower_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_follow.py::TestFormation::test_formation_follow_moving_ball
// original-source-sha256: c343e312ca36a9b7ecc3d162889086cc965803fc4d364e38a82eb4da80e5b1b9
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn follow__TestFormation__test_formation_follow_moving_ball() {
    let mut park = OriginalParkProjection::default();
    park.load_original_formations();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let follower = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(follower) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(follower, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(follower, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(follower, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(follower, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(follower, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(follower, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let leader = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: -(5.0_f64), y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(leader) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(leader, true);
    let attribute_value_27 = (10.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.max_velocity = attribute_value_27, OriginalBallAttribute::MaxSpeed);
    let attribute_value_29 = (0.9_f64) as f32;
    park.write_attribute_notified(leader, |b| b.agility = attribute_value_29, OriginalBallAttribute::Agility);
    let attribute_value_31 = 13000000.0_f64;
    park.write_attribute_notified(leader, |b| b.mass = attribute_value_31, OriginalBallAttribute::Mass);
    let attribute_value_33 = (2.0_f64) as f32;
    park.write_attribute_notified(leader, |b| b.radius = attribute_value_33, OriginalBallAttribute::Radius);
    let attribute_value_35 = true;
    park.write_attribute_notified(leader, |b| b.is_massive = attribute_value_35, OriginalBallAttribute::Massive);
    let attribute_value_37 = (0.95_f64) as f32;
    park.write_attribute_notified(leader, |b| b.speed_fraction = attribute_value_37, OriginalBallAttribute::SpeedFraction);
    park.set_formation(leader, (1.0_f64) as i8);
    let slot = park.reserve_original_formation_slot(leader);
    park.formation_follow(follower, leader, slot);
    park.goto_point(leader, Vec3 { x: 0.0_f64, y: 100.0_f64, z: 0.0_f64 });
    let mut follower_coordinates=BoundedList::<Vec3,10>::new();
    let mut loop_iteration_44=0_i64;
    let mut i=0_i64;
    while loop_iteration_44 < 10_i64 {
        i=loop_iteration_44;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(follower_coordinates.push(Vec3 { x: park.retained(follower).position.x, y: park.retained(follower).position.y, z: park.retained(follower).position.z })); // every original trace element retained
        loop_iteration_44=loop_iteration_44.saturating_add(1);
    }
    let expected_src_coordinates = point_list::<10>(&[Vec3 { x: 0.34937874956616805_f64, y: 0.0_f64, z: -(0.18354986649486965_f64) }, Vec3 { x: 1.2522700311290196_f64, y: -(0.2984878365701446_f64), z: -(0.442015514428003_f64) }, Vec3 { x: 2.6428549859837505_f64, y: -(1.083812623793305_f64), z: -(0.568440079311451_f64) }, Vec3 { x: 4.525369455660191_f64, y: -(2.2133606150700387_f64), z: -(0.8465910315244277_f64) }, Vec3 { x: 6.698838038625631_f64, y: -(3.6099869836316993_f64), z: -(1.5982896364292383_f64) }, Vec3 { x: 8.916087445091419_f64, y: -(5.209637520522771_f64), z: -(2.9541942809113397_f64) }, Vec3 { x: 11.011849619430125_f64, y: -(6.955575729117526_f64), z: -(4.920658718422647_f64) }, Vec3 { x: 12.897996631839323_f64, y: -(8.79576410566157_f64), z: -(7.465035973264715_f64) }, Vec3 { x: 14.532033823223955_f64, y: -(10.680120838194423_f64), z: -(10.547372880541618_f64) }, Vec3 { x: 15.895577914276311_f64, y: -(12.55839407965528_f64), z: -(14.128291692594965_f64) }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_follow.py:72
    assert_points_rounded(&follower_coordinates, &expected_src_coordinates, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_warp.py::TestWarp::test_warpto
// original-source-sha256: 7e7fd14f284fdc576b1f83ff1a2fa79545908a1edde5176b7145193676e72c01
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn warp__TestWarp__test_warpto() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let mut locations=BoundedList::<Vec3,10>::new();
    let ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball, true);
    let attribute_value_9 = (10.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.max_velocity = attribute_value_9, OriginalBallAttribute::MaxSpeed);
    let attribute_value_11 = (0.9_f64) as f32;
    park.write_attribute_notified(ball, |b| b.agility = attribute_value_11, OriginalBallAttribute::Agility);
    let attribute_value_13 = 13000000.0_f64;
    park.write_attribute_notified(ball, |b| b.mass = attribute_value_13, OriginalBallAttribute::Mass);
    let attribute_value_15 = (2.0_f64) as f32;
    park.write_attribute_notified(ball, |b| b.radius = attribute_value_15, OriginalBallAttribute::Radius);
    let attribute_value_17 = true;
    park.write_attribute_notified(ball, |b| b.is_massive = attribute_value_17, OriginalBallAttribute::Massive);
    let attribute_value_19 = (0.95_f64) as f32;
    park.write_attribute_notified(ball, |b| b.speed_fraction = attribute_value_19, OriginalBallAttribute::SpeedFraction);
    let attribute_value_21 = 1.0_f64;
    park.write_attribute_notified(ball, |b| b.velocity.x = attribute_value_21, OriginalBallAttribute::Velocity);
    let attribute_value_23 = 2.0_f64;
    park.write_attribute_notified(ball, |b| b.velocity.y = attribute_value_23, OriginalBallAttribute::Velocity);
    let attribute_value_25 = 3.0_f64;
    park.write_attribute_notified(ball, |b| b.velocity.z = attribute_value_25, OriginalBallAttribute::Velocity);
    let x = 100000.0_f64;
    let y = 200000.0_f64;
    let z = 300000.0_f64;
    park.warp_to(ball, Vec3 { x: x, y: y, z: z }, 20000.0_f64, (20.0_f64) as i32);
    let mut loop_iteration_31=0_i64;
    let mut i=0_i64;
    while loop_iteration_31 < 10_i64 {
        i=loop_iteration_31;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        // original-assertion: python/destiny/test/ballpark/evolve/test_warp.py:25
        assert!(park.retained(ball).mode == OriginalMode::Warp);
        assert!(locations.push(Vec3 { x: park.retained(ball).position.x, y: park.retained(ball).position.y, z: park.retained(ball).position.z })); // every original trace element retained
        loop_iteration_31=loop_iteration_31.saturating_add(1);
    }
    let expected_locations = point_list::<10>(&[Vec3 { x: 1.0639340706602571_f64, y: 2.1278681413205143_f64, z: 3.1918022119807685_f64 }, Vec3 { x: 2.248703156860341_f64, y: 4.497406313720682_f64, z: 6.746109470581021_f64 }, Vec3 { x: 3.544408527173739_f64, y: 7.088817054347478_f64, z: 10.633225581521215_f64 }, Vec3 { x: 4.941962348268478_f64, y: 9.883924696536956_f64, z: 14.825887044805434_f64 }, Vec3 { x: 6.433021256625409_f64, y: 12.866042513250818_f64, z: 19.299063769876224_f64 }, Vec3 { x: 8.00992537202119_f64, y: 16.01985074404238_f64, z: 24.029776116063566_f64 }, Vec3 { x: 9.665642306989863_f64, y: 19.331284613979726_f64, z: 28.99692692096958_f64 }, Vec3 { x: 11.393715762995505_f64, y: 22.78743152599101_f64, z: 34.18114728898651_f64 }, Vec3 { x: 13.188218337575332_f64, y: 26.376436675150664_f64, z: 39.56465501272599_f64 }, Vec3 { x: 15.043708197493105_f64, y: 30.08741639498621_f64, z: 45.13112459247932_f64 }]);
    // original-assertion: python/destiny/test/ballpark/evolve/test_warp.py:41
    assert_points_rounded(&locations, &expected_locations, 4);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_simple_collision.py::TestSimpleCollision::test_balls_in_warp_should_not_collide
// original-source-sha256: 7a19dd4c81f42fec1888207329bff1cd63a748d9bcc22c2f1ae83e4cd1c294f5
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(81))]
#[cfg_attr(test, test)]
fn simple_collision__TestSimpleCollision__test_balls_in_warp_should_not_collide() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let ball_a = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: -(4.0_f64), y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 1000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_a) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_a, true);
    let attribute_value_8 = (1000.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.max_velocity = attribute_value_8, OriginalBallAttribute::MaxSpeed);
    let attribute_value_10 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.agility = attribute_value_10, OriginalBallAttribute::Agility);
    let attribute_value_12 = 1.0_f64;
    park.write_attribute_notified(ball_a, |b| b.mass = attribute_value_12, OriginalBallAttribute::Mass);
    let attribute_value_14 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.radius = attribute_value_14, OriginalBallAttribute::Radius);
    let attribute_value_16 = true;
    park.write_attribute_notified(ball_a, |b| b.is_massive = attribute_value_16, OriginalBallAttribute::Massive);
    let attribute_value_18 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.speed_fraction = attribute_value_18, OriginalBallAttribute::SpeedFraction);
    let ball_b = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 4.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: -(1000.0_f64), y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(ball_b) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(ball_b, true);
    let attribute_value_26 = (1000.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.max_velocity = attribute_value_26, OriginalBallAttribute::MaxSpeed);
    let attribute_value_28 = (0.9_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.agility = attribute_value_28, OriginalBallAttribute::Agility);
    let attribute_value_30 = 1.0_f64;
    park.write_attribute_notified(ball_b, |b| b.mass = attribute_value_30, OriginalBallAttribute::Mass);
    let attribute_value_32 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.radius = attribute_value_32, OriginalBallAttribute::Radius);
    let attribute_value_34 = true;
    park.write_attribute_notified(ball_b, |b| b.is_massive = attribute_value_34, OriginalBallAttribute::Massive);
    let attribute_value_36 = (0.95_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.speed_fraction = attribute_value_36, OriginalBallAttribute::SpeedFraction);
    let attribute_value_38 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_a, |b| b.radius = attribute_value_38, OriginalBallAttribute::Radius);
    let attribute_value_40 = (2.0_f64) as f32;
    park.write_attribute_notified(ball_b, |b| b.radius = attribute_value_40, OriginalBallAttribute::Radius);
    park.warp_to(ball_a, Vec3 { x: 100000.0_f64, y: 0.0_f64, z: 0.0_f64 }, 20000.0_f64, (20.0_f64) as i32);
    park.warp_to(ball_b, Vec3 { x: -(100000.0_f64), y: 0.0_f64, z: 0.0_f64 }, 20000.0_f64, (20.0_f64) as i32);
    let mut loop_iteration_44=0_i64;
    let mut i=0_i64;
    while loop_iteration_44 < 80_i64 {
        i=loop_iteration_44;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        loop_iteration_44=loop_iteration_44.saturating_add(1);
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:170
    assert!(park.retained(ball_a).position.x > park.retained(ball_b).position.x);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:171
    assert!(park.retained(ball_a).position.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:172
    assert!(park.retained(ball_a).position.z == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:173
    assert!(park.retained(ball_b).position.y == 0.0_f64);
    // original-assertion: python/destiny/test/ballpark/evolve/test_simple_collision.py:174
    assert!(park.retained(ball_b).position.z == 0.0_f64);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_missile.py::TestMissile::test_aimed_not_massive
// original-source-sha256: 54318296ae3b22357998fc71df774ecdbdd177f9ac5a29df9dd472716d40aa67
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn missile__TestMissile__test_aimed_not_massive() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id=1_i64;
    let fixture_dst = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 1000000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_dst) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_dst, true);
    let attribute_value_7 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.max_velocity = attribute_value_7, OriginalBallAttribute::MaxSpeed);
    let attribute_value_9 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.agility = attribute_value_9, OriginalBallAttribute::Agility);
    let attribute_value_11 = 13000000.0_f64;
    park.write_attribute_notified(fixture_dst, |b| b.mass = attribute_value_11, OriginalBallAttribute::Mass);
    let attribute_value_13 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.radius = attribute_value_13, OriginalBallAttribute::Radius);
    let attribute_value_15 = true;
    park.write_attribute_notified(fixture_dst, |b| b.is_massive = attribute_value_15, OriginalBallAttribute::Massive);
    let attribute_value_17 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.speed_fraction = attribute_value_17, OriginalBallAttribute::SpeedFraction);
    let fixture_owner = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_owner) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_owner, true);
    let attribute_value_25 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.max_velocity = attribute_value_25, OriginalBallAttribute::MaxSpeed);
    let attribute_value_27 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.agility = attribute_value_27, OriginalBallAttribute::Agility);
    let attribute_value_29 = 13000000.0_f64;
    park.write_attribute_notified(fixture_owner, |b| b.mass = attribute_value_29, OriginalBallAttribute::Mass);
    let attribute_value_31 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.radius = attribute_value_31, OriginalBallAttribute::Radius);
    let attribute_value_33 = true;
    park.write_attribute_notified(fixture_owner, |b| b.is_massive = attribute_value_33, OriginalBallAttribute::Massive);
    let attribute_value_35 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.speed_fraction = attribute_value_35, OriginalBallAttribute::SpeedFraction);
    let fixture_collider = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 2000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_collider) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_collider, true);
    let attribute_value_43 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.max_velocity = attribute_value_43, OriginalBallAttribute::MaxSpeed);
    let attribute_value_45 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.agility = attribute_value_45, OriginalBallAttribute::Agility);
    let attribute_value_47 = 13000000.0_f64;
    park.write_attribute_notified(fixture_collider, |b| b.mass = attribute_value_47, OriginalBallAttribute::Mass);
    let attribute_value_49 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_49, OriginalBallAttribute::Radius);
    let attribute_value_51 = true;
    park.write_attribute_notified(fixture_collider, |b| b.is_massive = attribute_value_51, OriginalBallAttribute::Massive);
    let attribute_value_53 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.speed_fraction = attribute_value_53, OriginalBallAttribute::SpeedFraction);
    let attribute_value_55 = (1900.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_55, OriginalBallAttribute::Radius);
    let fixture_missile = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_missile) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_missile, true);
    let attribute_value_63 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_63, OriginalBallAttribute::MaxSpeed);
    let attribute_value_65 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_65, OriginalBallAttribute::Agility);
    let attribute_value_67 = 13000000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_67, OriginalBallAttribute::Mass);
    let attribute_value_69 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.radius = attribute_value_69, OriginalBallAttribute::Radius);
    let attribute_value_71 = true;
    park.write_attribute_notified(fixture_missile, |b| b.is_massive = attribute_value_71, OriginalBallAttribute::Massive);
    let attribute_value_73 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.speed_fraction = attribute_value_73, OriginalBallAttribute::SpeedFraction);
    let attribute_value_75 = (300.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_75, OriginalBallAttribute::MaxSpeed);
    let attribute_value_77 = (0.01_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_77, OriginalBallAttribute::Agility);
    // Pinned Ball_Blue.cpp exposes both rotational fields; retain its accepted path.
    let attribute_value_80 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_angular_velocity = attribute_value_80, OriginalBallAttribute::MaxAngularSpeed);
    let attribute_value_82 = (0.5_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.rotational_agility = attribute_value_82, OriginalBallAttribute::RotationalAgility);
    let attribute_value_84 = 10000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_84, OriginalBallAttribute::Mass);
    assert!(park.launch_missile(fixture_missile, fixture_dst, fixture_owner, i8::from(true), i8::from(false)) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:84
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, 1.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:85
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:86
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.z, 0.0_f64, 7) == Some(true));
    let mut loop_iteration_93=0_i64;
    let mut i=0_i64;
    while loop_iteration_93 < 20_i64 {
        i=loop_iteration_93;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        loop_iteration_93=loop_iteration_93.saturating_add(1);
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:92
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, (f64::from(park.retained(fixture_missile).max_velocity) * f64::from(park.retained(fixture_missile).speed_fraction)), 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:93
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.y, 0.0_f64, 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:94
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.z, 0.0_f64, 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:97
    assert!(python_almost_equal(park.retained(fixture_collider).position.x, 2000.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:98
    assert!(python_almost_equal(park.retained(fixture_collider).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:99
    assert!(python_almost_equal(park.retained(fixture_collider).position.z, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:102
    assert!(park.retained(fixture_missile).position.x > park.retained(fixture_collider).position.x);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_missile.py::TestMissile::test_aimed_massive
// original-source-sha256: 54318296ae3b22357998fc71df774ecdbdd177f9ac5a29df9dd472716d40aa67
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn missile__TestMissile__test_aimed_massive() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id=1_i64;
    let fixture_dst = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 1000000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_dst) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_dst, true);
    let attribute_value_7 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.max_velocity = attribute_value_7, OriginalBallAttribute::MaxSpeed);
    let attribute_value_9 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.agility = attribute_value_9, OriginalBallAttribute::Agility);
    let attribute_value_11 = 13000000.0_f64;
    park.write_attribute_notified(fixture_dst, |b| b.mass = attribute_value_11, OriginalBallAttribute::Mass);
    let attribute_value_13 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.radius = attribute_value_13, OriginalBallAttribute::Radius);
    let attribute_value_15 = true;
    park.write_attribute_notified(fixture_dst, |b| b.is_massive = attribute_value_15, OriginalBallAttribute::Massive);
    let attribute_value_17 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.speed_fraction = attribute_value_17, OriginalBallAttribute::SpeedFraction);
    let fixture_owner = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_owner) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_owner, true);
    let attribute_value_25 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.max_velocity = attribute_value_25, OriginalBallAttribute::MaxSpeed);
    let attribute_value_27 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.agility = attribute_value_27, OriginalBallAttribute::Agility);
    let attribute_value_29 = 13000000.0_f64;
    park.write_attribute_notified(fixture_owner, |b| b.mass = attribute_value_29, OriginalBallAttribute::Mass);
    let attribute_value_31 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.radius = attribute_value_31, OriginalBallAttribute::Radius);
    let attribute_value_33 = true;
    park.write_attribute_notified(fixture_owner, |b| b.is_massive = attribute_value_33, OriginalBallAttribute::Massive);
    let attribute_value_35 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.speed_fraction = attribute_value_35, OriginalBallAttribute::SpeedFraction);
    let fixture_collider = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 2000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_collider) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_collider, true);
    let attribute_value_43 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.max_velocity = attribute_value_43, OriginalBallAttribute::MaxSpeed);
    let attribute_value_45 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.agility = attribute_value_45, OriginalBallAttribute::Agility);
    let attribute_value_47 = 13000000.0_f64;
    park.write_attribute_notified(fixture_collider, |b| b.mass = attribute_value_47, OriginalBallAttribute::Mass);
    let attribute_value_49 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_49, OriginalBallAttribute::Radius);
    let attribute_value_51 = true;
    park.write_attribute_notified(fixture_collider, |b| b.is_massive = attribute_value_51, OriginalBallAttribute::Massive);
    let attribute_value_53 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.speed_fraction = attribute_value_53, OriginalBallAttribute::SpeedFraction);
    let attribute_value_55 = (1900.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_55, OriginalBallAttribute::Radius);
    let fixture_missile = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_missile) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_missile, true);
    let attribute_value_63 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_63, OriginalBallAttribute::MaxSpeed);
    let attribute_value_65 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_65, OriginalBallAttribute::Agility);
    let attribute_value_67 = 13000000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_67, OriginalBallAttribute::Mass);
    let attribute_value_69 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.radius = attribute_value_69, OriginalBallAttribute::Radius);
    let attribute_value_71 = true;
    park.write_attribute_notified(fixture_missile, |b| b.is_massive = attribute_value_71, OriginalBallAttribute::Massive);
    let attribute_value_73 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.speed_fraction = attribute_value_73, OriginalBallAttribute::SpeedFraction);
    let attribute_value_75 = (300.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_75, OriginalBallAttribute::MaxSpeed);
    let attribute_value_77 = (0.01_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_77, OriginalBallAttribute::Agility);
    // Pinned Ball_Blue.cpp exposes both rotational fields; retain its accepted path.
    let attribute_value_80 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_angular_velocity = attribute_value_80, OriginalBallAttribute::MaxAngularSpeed);
    let attribute_value_82 = (0.5_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.rotational_agility = attribute_value_82, OriginalBallAttribute::RotationalAgility);
    let attribute_value_84 = 10000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_84, OriginalBallAttribute::Mass);
    assert!(park.launch_missile(fixture_missile, fixture_dst, fixture_owner, i8::from(true), i8::from(true)) == Ok(())); // original call must not raise
    let mut loop_iteration_87=0_i64;
    let mut i=0_i64;
    while loop_iteration_87 < 20_i64 {
        i=loop_iteration_87;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        loop_iteration_87=loop_iteration_87.saturating_add(1);
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:111
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, (f64::from(park.retained(fixture_missile).max_velocity) * f64::from(park.retained(fixture_missile).speed_fraction)), 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:112
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.y, 0.0_f64, 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:113
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.z, 0.0_f64, 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:116
    assert!(python_almost_equal(park.retained(fixture_collider).position.x, 2000.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:117
    assert!(python_almost_equal(park.retained(fixture_collider).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:118
    assert!(python_almost_equal(park.retained(fixture_collider).position.z, 0.0_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_missile.py::TestMissile::test_not_aimed_not_massive_from_stationary
// original-source-sha256: 54318296ae3b22357998fc71df774ecdbdd177f9ac5a29df9dd472716d40aa67
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn missile__TestMissile__test_not_aimed_not_massive_from_stationary() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id=1_i64;
    let fixture_dst = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 1000000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_dst) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_dst, true);
    let attribute_value_7 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.max_velocity = attribute_value_7, OriginalBallAttribute::MaxSpeed);
    let attribute_value_9 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.agility = attribute_value_9, OriginalBallAttribute::Agility);
    let attribute_value_11 = 13000000.0_f64;
    park.write_attribute_notified(fixture_dst, |b| b.mass = attribute_value_11, OriginalBallAttribute::Mass);
    let attribute_value_13 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.radius = attribute_value_13, OriginalBallAttribute::Radius);
    let attribute_value_15 = true;
    park.write_attribute_notified(fixture_dst, |b| b.is_massive = attribute_value_15, OriginalBallAttribute::Massive);
    let attribute_value_17 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.speed_fraction = attribute_value_17, OriginalBallAttribute::SpeedFraction);
    let fixture_owner = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_owner) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_owner, true);
    let attribute_value_25 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.max_velocity = attribute_value_25, OriginalBallAttribute::MaxSpeed);
    let attribute_value_27 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.agility = attribute_value_27, OriginalBallAttribute::Agility);
    let attribute_value_29 = 13000000.0_f64;
    park.write_attribute_notified(fixture_owner, |b| b.mass = attribute_value_29, OriginalBallAttribute::Mass);
    let attribute_value_31 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.radius = attribute_value_31, OriginalBallAttribute::Radius);
    let attribute_value_33 = true;
    park.write_attribute_notified(fixture_owner, |b| b.is_massive = attribute_value_33, OriginalBallAttribute::Massive);
    let attribute_value_35 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.speed_fraction = attribute_value_35, OriginalBallAttribute::SpeedFraction);
    let fixture_collider = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 2000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_collider) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_collider, true);
    let attribute_value_43 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.max_velocity = attribute_value_43, OriginalBallAttribute::MaxSpeed);
    let attribute_value_45 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.agility = attribute_value_45, OriginalBallAttribute::Agility);
    let attribute_value_47 = 13000000.0_f64;
    park.write_attribute_notified(fixture_collider, |b| b.mass = attribute_value_47, OriginalBallAttribute::Mass);
    let attribute_value_49 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_49, OriginalBallAttribute::Radius);
    let attribute_value_51 = true;
    park.write_attribute_notified(fixture_collider, |b| b.is_massive = attribute_value_51, OriginalBallAttribute::Massive);
    let attribute_value_53 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.speed_fraction = attribute_value_53, OriginalBallAttribute::SpeedFraction);
    let attribute_value_55 = (1900.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_55, OriginalBallAttribute::Radius);
    let fixture_missile = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_missile) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_missile, true);
    let attribute_value_63 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_63, OriginalBallAttribute::MaxSpeed);
    let attribute_value_65 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_65, OriginalBallAttribute::Agility);
    let attribute_value_67 = 13000000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_67, OriginalBallAttribute::Mass);
    let attribute_value_69 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.radius = attribute_value_69, OriginalBallAttribute::Radius);
    let attribute_value_71 = true;
    park.write_attribute_notified(fixture_missile, |b| b.is_massive = attribute_value_71, OriginalBallAttribute::Massive);
    let attribute_value_73 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.speed_fraction = attribute_value_73, OriginalBallAttribute::SpeedFraction);
    let attribute_value_75 = (300.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_75, OriginalBallAttribute::MaxSpeed);
    let attribute_value_77 = (0.01_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_77, OriginalBallAttribute::Agility);
    // Pinned Ball_Blue.cpp exposes both rotational fields; retain its accepted path.
    let attribute_value_80 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_angular_velocity = attribute_value_80, OriginalBallAttribute::MaxAngularSpeed);
    let attribute_value_82 = (0.5_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.rotational_agility = attribute_value_82, OriginalBallAttribute::RotationalAgility);
    let attribute_value_84 = 10000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_84, OriginalBallAttribute::Mass);
    assert!(park.launch_missile(fixture_missile, fixture_dst, fixture_owner, i8::from(false), i8::from(false)) == Ok(())); // original call must not raise
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:39
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:40
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:41
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.z, -(150.0_f64), 7) == Some(true));
    let mut loop_iteration_93=0_i64;
    let mut i=0_i64;
    while loop_iteration_93 < 20_i64 {
        i=loop_iteration_93;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        loop_iteration_93=loop_iteration_93.saturating_add(1);
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:47
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, (f64::from(park.retained(fixture_missile).max_velocity) * f64::from(park.retained(fixture_missile).speed_fraction)), 2) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:48
    assert!(destiny_original_spec::python_almost_equal_delta(park.retained(fixture_missile).velocity.y,0.0_f64,0.5_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:49
    assert!(destiny_original_spec::python_almost_equal_delta(park.retained(fixture_missile).velocity.z,0.0_f64,0.5_f64));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:52
    assert!(python_almost_equal(park.retained(fixture_collider).position.x, 2000.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:53
    assert!(python_almost_equal(park.retained(fixture_collider).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:54
    assert!(python_almost_equal(park.retained(fixture_collider).position.z, 0.0_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_missile.py::TestMissile::test_not_aimed_not_massive_from_moving
// original-source-sha256: 54318296ae3b22357998fc71df774ecdbdd177f9ac5a29df9dd472716d40aa67
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn missile__TestMissile__test_not_aimed_not_massive_from_moving() {
    let mut park = OriginalParkProjection::default();
    let mut next_space_object_id=1_i64;
    let fixture_dst = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 1000000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_dst) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_dst, true);
    let attribute_value_7 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.max_velocity = attribute_value_7, OriginalBallAttribute::MaxSpeed);
    let attribute_value_9 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.agility = attribute_value_9, OriginalBallAttribute::Agility);
    let attribute_value_11 = 13000000.0_f64;
    park.write_attribute_notified(fixture_dst, |b| b.mass = attribute_value_11, OriginalBallAttribute::Mass);
    let attribute_value_13 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.radius = attribute_value_13, OriginalBallAttribute::Radius);
    let attribute_value_15 = true;
    park.write_attribute_notified(fixture_dst, |b| b.is_massive = attribute_value_15, OriginalBallAttribute::Massive);
    let attribute_value_17 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_dst, |b| b.speed_fraction = attribute_value_17, OriginalBallAttribute::SpeedFraction);
    let fixture_owner = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_owner) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_owner, true);
    let attribute_value_25 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.max_velocity = attribute_value_25, OriginalBallAttribute::MaxSpeed);
    let attribute_value_27 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.agility = attribute_value_27, OriginalBallAttribute::Agility);
    let attribute_value_29 = 13000000.0_f64;
    park.write_attribute_notified(fixture_owner, |b| b.mass = attribute_value_29, OriginalBallAttribute::Mass);
    let attribute_value_31 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.radius = attribute_value_31, OriginalBallAttribute::Radius);
    let attribute_value_33 = true;
    park.write_attribute_notified(fixture_owner, |b| b.is_massive = attribute_value_33, OriginalBallAttribute::Massive);
    let attribute_value_35 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_owner, |b| b.speed_fraction = attribute_value_35, OriginalBallAttribute::SpeedFraction);
    let fixture_collider = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 2000.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_collider) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_collider, true);
    let attribute_value_43 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.max_velocity = attribute_value_43, OriginalBallAttribute::MaxSpeed);
    let attribute_value_45 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.agility = attribute_value_45, OriginalBallAttribute::Agility);
    let attribute_value_47 = 13000000.0_f64;
    park.write_attribute_notified(fixture_collider, |b| b.mass = attribute_value_47, OriginalBallAttribute::Mass);
    let attribute_value_49 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_49, OriginalBallAttribute::Radius);
    let attribute_value_51 = true;
    park.write_attribute_notified(fixture_collider, |b| b.is_massive = attribute_value_51, OriginalBallAttribute::Massive);
    let attribute_value_53 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.speed_fraction = attribute_value_53, OriginalBallAttribute::SpeedFraction);
    let attribute_value_55 = (1900.0_f64) as f32;
    park.write_attribute_notified(fixture_collider, |b| b.radius = attribute_value_55, OriginalBallAttribute::Radius);
    let fixture_missile = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, ..OriginalBallInit::default() });
    assert!(park.contains(fixture_missile) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(fixture_missile, true);
    let attribute_value_63 = (10.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_63, OriginalBallAttribute::MaxSpeed);
    let attribute_value_65 = (0.9_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_65, OriginalBallAttribute::Agility);
    let attribute_value_67 = 13000000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_67, OriginalBallAttribute::Mass);
    let attribute_value_69 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.radius = attribute_value_69, OriginalBallAttribute::Radius);
    let attribute_value_71 = true;
    park.write_attribute_notified(fixture_missile, |b| b.is_massive = attribute_value_71, OriginalBallAttribute::Massive);
    let attribute_value_73 = (0.95_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.speed_fraction = attribute_value_73, OriginalBallAttribute::SpeedFraction);
    let attribute_value_75 = (300.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_velocity = attribute_value_75, OriginalBallAttribute::MaxSpeed);
    let attribute_value_77 = (0.01_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.agility = attribute_value_77, OriginalBallAttribute::Agility);
    // Pinned Ball_Blue.cpp exposes both rotational fields; retain its accepted path.
    let attribute_value_80 = (2.0_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.max_angular_velocity = attribute_value_80, OriginalBallAttribute::MaxAngularSpeed);
    let attribute_value_82 = (0.5_f64) as f32;
    park.write_attribute_notified(fixture_missile, |b| b.rotational_agility = attribute_value_82, OriginalBallAttribute::RotationalAgility);
    let attribute_value_84 = 10000.0_f64;
    park.write_attribute_notified(fixture_missile, |b| b.mass = attribute_value_84, OriginalBallAttribute::Mass);
    let attribute_value_86 = 10.0_f64;
    park.write_attribute_notified(fixture_owner, |b| b.velocity.z = attribute_value_86, OriginalBallAttribute::Velocity);
    assert!(park.launch_missile(fixture_missile, fixture_dst, fixture_owner, i8::from(false), i8::from(false)) == Ok(())); // original call must not raise
    let attribute_value_89 = 0.0_f64;
    park.write_attribute_notified(fixture_owner, |b| b.velocity.z = attribute_value_89, OriginalBallAttribute::Velocity);
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:63
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:64
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:65
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.z, 160.0_f64, 7) == Some(true));
    let mut loop_iteration_97=0_i64;
    let mut i=0_i64;
    while loop_iteration_97 < 20_i64 {
        i=loop_iteration_97;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        loop_iteration_97=loop_iteration_97.saturating_add(1);
    }
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:71
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.x, (f64::from(park.retained(fixture_missile).max_velocity) * f64::from(park.retained(fixture_missile).speed_fraction)), 4) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:72
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.y, 0.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:73
    assert!(python_almost_equal(park.retained(fixture_missile).velocity.z, 0.0_f64, 0) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:76
    assert!(python_almost_equal(park.retained(fixture_collider).position.x, 2000.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:77
    assert!(python_almost_equal(park.retained(fixture_collider).position.y, 0.0_f64, 7) == Some(true));
    // original-assertion: python/destiny/test/ballpark/evolve/test_missile.py:78
    assert!(python_almost_equal(park.retained(fixture_collider).position.z, 0.0_f64, 7) == Some(true));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_minis.py::TestMiniBall::test_collide
// original-source-sha256: c26676e43f9557c49b25ed404895c0ba0bfd5ab5d25b4b34b7d2037c527d4ebe
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn minis__TestMiniBall__test_collide() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let parent = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(parent) && !park.scope_error);
    let attribute_value_4 = 100.0_f64;
    park.write_attribute_notified(parent, |b| b.position.x = attribute_value_4, OriginalBallAttribute::Position);
    let attribute_value_6 = 0.0_f64;
    park.write_attribute_notified(parent, |b| b.position.y = attribute_value_6, OriginalBallAttribute::Position);
    let attribute_value_8 = 0.0_f64;
    park.write_attribute_notified(parent, |b| b.position.z = attribute_value_8, OriginalBallAttribute::Position);
    assert!(park.add_mini_ball(parent, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, (10.0_f64) as f32) == Ok(())); // original call must not raise
    next_space_object_id=2_i64; // original global helper reset; preserve its exact value
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 200.0_f64, y: 0.0_f64, z: -(30.0_f64) }, ..OriginalBallInit::default() });
    assert!(park.contains(src) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(src, true);
    let attribute_value_18 = (10.0_f64) as f32;
    park.write_attribute_notified(src, |b| b.max_velocity = attribute_value_18, OriginalBallAttribute::MaxSpeed);
    let attribute_value_20 = (0.9_f64) as f32;
    park.write_attribute_notified(src, |b| b.agility = attribute_value_20, OriginalBallAttribute::Agility);
    let attribute_value_22 = 13000000.0_f64;
    park.write_attribute_notified(src, |b| b.mass = attribute_value_22, OriginalBallAttribute::Mass);
    let attribute_value_24 = (2.0_f64) as f32;
    park.write_attribute_notified(src, |b| b.radius = attribute_value_24, OriginalBallAttribute::Radius);
    let attribute_value_26 = true;
    park.write_attribute_notified(src, |b| b.is_massive = attribute_value_26, OriginalBallAttribute::Massive);
    let attribute_value_28 = (0.95_f64) as f32;
    park.write_attribute_notified(src, |b| b.speed_fraction = attribute_value_28, OriginalBallAttribute::SpeedFraction);
    let attribute_value_30 = (10.0_f64) as f32;
    park.write_attribute_notified(src, |b| b.radius = attribute_value_30, OriginalBallAttribute::Radius);
    park.goto_point(src, Vec3 { x: 200.0_f64, y: 0.0_f64, z: 0.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_34=0_i64;
    let mut i=0_i64;
    while loop_iteration_34 < 20_i64 {
        i=loop_iteration_34;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(src).position.x, y: park.retained(src).position.y, z: park.retained(src).position.z })); // every original trace element retained
        loop_iteration_34=loop_iteration_34.saturating_add(1);
    }
    let coords = coordinates;
    let mut point_iteration_43=0_usize;
    while point_iteration_43<coords.len() {
        let point_access_43=coords.get(point_iteration_43);
        assert!(point_access_43.is_some()); // original complete ordered list traversal
        let coord=point_access_43.unwrap_or(Vec3::ZERO);
        // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:105
        assert!(python_almost_equal(coord.x, 200.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:106
        assert!(python_almost_equal(coord.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:107
        let _diagnostic_arguments=(coord.z, -(20.0_f64)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!((((1.1_f64 * coord.z) < -(20.0_f64))));
        point_iteration_43=point_iteration_43.saturating_add(1);
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}

// original-case: python/destiny/test/ballpark/evolve/test_minis.py::TestMiniBall::test_collide_with_miniballs
// original-source-sha256: c26676e43f9557c49b25ed404895c0ba0bfd5ab5d25b4b34b7d2037c527d4ebe
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn minis__TestMiniBall__test_collide_with_miniballs() {
    let mut park = OriginalParkProjection::default();
    // Fresh Python helper module starts its next_object_id counter at 1.
    let mut next_space_object_id=1_i64;
    let parent = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(parent) && !park.scope_error);
    let attribute_value_4 = 100.0_f64;
    park.write_attribute_notified(parent, |b| b.position.x = attribute_value_4, OriginalBallAttribute::Position);
    let attribute_value_6 = 0.0_f64;
    park.write_attribute_notified(parent, |b| b.position.y = attribute_value_6, OriginalBallAttribute::Position);
    let attribute_value_8 = 0.0_f64;
    park.write_attribute_notified(parent, |b| b.position.z = attribute_value_8, OriginalBallAttribute::Position);
    assert!(park.add_mini_ball(parent, Vec3 { x: 100.0_f64, y: 0.0_f64, z: 0.0_f64 }, (10.0_f64) as f32) == Ok(())); // original call must not raise
    next_space_object_id=2_i64; // original global helper reset; preserve its exact value
    let src = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: next_space_object_id, position: Vec3 { x: 200.0_f64, y: 0.0_f64, z: -(30.0_f64) }, ..OriginalBallInit::default() });
    assert!(park.contains(src) && !park.scope_error);
    let next_id = next_space_object_id.checked_add(1);
    assert!(next_id.is_some()); // explicit int64 adapter scope, no iteration omitted
    next_space_object_id = next_id.unwrap_or(next_space_object_id);
    park.set_free(src, true);
    let attribute_value_18 = (10.0_f64) as f32;
    park.write_attribute_notified(src, |b| b.max_velocity = attribute_value_18, OriginalBallAttribute::MaxSpeed);
    let attribute_value_20 = (0.9_f64) as f32;
    park.write_attribute_notified(src, |b| b.agility = attribute_value_20, OriginalBallAttribute::Agility);
    let attribute_value_22 = 13000000.0_f64;
    park.write_attribute_notified(src, |b| b.mass = attribute_value_22, OriginalBallAttribute::Mass);
    let attribute_value_24 = (2.0_f64) as f32;
    park.write_attribute_notified(src, |b| b.radius = attribute_value_24, OriginalBallAttribute::Radius);
    let attribute_value_26 = true;
    park.write_attribute_notified(src, |b| b.is_massive = attribute_value_26, OriginalBallAttribute::Massive);
    let attribute_value_28 = (0.95_f64) as f32;
    park.write_attribute_notified(src, |b| b.speed_fraction = attribute_value_28, OriginalBallAttribute::SpeedFraction);
    let attribute_value_30 = (10.0_f64) as f32;
    park.write_attribute_notified(src, |b| b.radius = attribute_value_30, OriginalBallAttribute::Radius);
    assert!(park.add_mini_ball(src, Vec3 { x: 4.0_f64, y: 0.0_f64, z: 4.0_f64 }, (3.0_f64) as f32) == Ok(())); // original call must not raise
    assert!(park.add_mini_ball(src, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 5.0_f64 }, (2.0_f64) as f32) == Ok(())); // original call must not raise
    assert!(park.add_mini_ball(src, Vec3 { x: -(4.0_f64), y: 0.0_f64, z: 4.0_f64 }, (3.0_f64) as f32) == Ok(())); // original call must not raise
    park.goto_point(src, Vec3 { x: 200.0_f64, y: 0.0_f64, z: 0.0_f64 });
    let mut coordinates=BoundedList::<Vec3,20>::new();
    let mut loop_iteration_37=0_i64;
    let mut i=0_i64;
    while loop_iteration_37 < 20_i64 {
        i=loop_iteration_37;
        assert!(park.evolve_two_spherical_bodies() == Ok(())); // original call must not raise
        assert!(coordinates.push(Vec3 { x: park.retained(src).position.x, y: park.retained(src).position.y, z: park.retained(src).position.z })); // every original trace element retained
        loop_iteration_37=loop_iteration_37.saturating_add(1);
    }
    let coords = coordinates;
    let mut point_iteration_46=0_usize;
    while point_iteration_46<coords.len() {
        let point_access_46=coords.get(point_iteration_46);
        assert!(point_access_46.is_some()); // original complete ordered list traversal
        let coord=point_access_46.unwrap_or(Vec3::ZERO);
        // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:129
        assert!(python_almost_equal(coord.x, 200.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:130
        assert!(python_almost_equal(coord.y, 0.0_f64, 7) == Some(true));
        // original-assertion: python/destiny/test/ballpark/evolve/test_minis.py:131
        let _diagnostic_arguments=(coord.z, -(17.0_f64)); // Python %f formatting/allocation remains a diagnostic boundary
        assert!((((1.01_f64 * coord.z) < -(17.0_f64))));
        point_iteration_46=point_iteration_46.saturating_add(1);
    }
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original park projection scenario and teardown completed");
}
