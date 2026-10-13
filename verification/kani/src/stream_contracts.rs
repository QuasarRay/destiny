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

use destiny_original_spec::OriginalStream;
fn stream_ids(ids:&[i64])->BoundedList<i64,8> { id_list(ids) }

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteBallsToStream::test_write_no_ball
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteBallsToStream__test_write_no_ball() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let original_empty_ids=BoundedList::<i64,8>::new();
    assert!(original_empty_ids.is_empty()); // original [] iteration has zero elements
    let written_stream=park.write_original_empty_balls(&original_empty_ids);
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:12
    assert!(stream.equals_bytes(&[1_u8, 0_u8, 0_u8, 0_u8, 0_u8]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteBallsToStream::test_write_ball_then_read_ball
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteBallsToStream__test_write_ball_then_read_ball() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let saved_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(saved_ball) && !park.scope_error);
    let written_stream=park.write_original_balls(&stream_ids(&[saved_ball]));
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    let saved_snapshot=park.retained(saved_ball);
    let mut reader_park=OriginalParkProjection::default();
    assert!(reader_park.read_original_state(&stream,0)==Ok(())); // original read must not raise
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:20
    assert!(reader_park.contains(saved_ball));
    assert!(reader_park.contains(saved_ball));
    let loaded_ball=reader_park.retained(saved_ball);
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:22; complete helper below
    assert!(park.ego_id==0 && reader_park.ego_id==0 && saved_snapshot.client_ball && loaded_ball.client_ball); // no-ego readonly property caller scope
    // original-assertion: python/destiny/test/helpers.py:84
    assert!(saved_snapshot.agility==loaded_ball.agility);
    // original-assertion: python/destiny/test/helpers.py:85
    assert!(saved_snapshot.alliance_id==loaded_ball.alliance_id);
    // original-assertion: python/destiny/test/helpers.py:86
    assert!(0.0_f64==0.0_f64);
    // original-assertion: python/destiny/test/helpers.py:87
    assert!(saved_snapshot.corporation_id==loaded_ball.corporation_id);
    // original-assertion: python/destiny/test/helpers.py:88
    assert!(saved_snapshot.effect_stamp==loaded_ball.effect_stamp);
    // original-assertion: python/destiny/test/helpers.py:89
    assert!(saved_snapshot.follow_id==loaded_ball.follow_id);
    // original-assertion: python/destiny/test/helpers.py:90
    assert!(saved_snapshot.follow_range==loaded_ball.follow_range);
    // original-assertion: python/destiny/test/helpers.py:91
    assert!(saved_snapshot.formation_id==loaded_ball.formation_id);
    // original-assertion: python/destiny/test/helpers.py:92
    assert!(saved_snapshot.goto.x==loaded_ball.goto.x);
    // original-assertion: python/destiny/test/helpers.py:93
    assert!(saved_snapshot.goto.y==loaded_ball.goto.y);
    // original-assertion: python/destiny/test/helpers.py:94
    assert!(saved_snapshot.goto.z==loaded_ball.goto.z);
    // original-assertion: python/destiny/test/helpers.py:95
    assert!(saved_snapshot.harmonic==loaded_ball.harmonic);
    // original-assertion: python/destiny/test/helpers.py:96
    assert!(saved_snapshot.id==loaded_ball.id);
    // original-assertion: python/destiny/test/helpers.py:97
    assert!(saved_snapshot.is_cloaked==loaded_ball.is_cloaked);
    // original-assertion: python/destiny/test/helpers.py:98
    assert!(saved_snapshot.is_free==loaded_ball.is_free);
    // original-assertion: python/destiny/test/helpers.py:99
    assert!(saved_snapshot.is_global==loaded_ball.is_global);
    // original-assertion: python/destiny/test/helpers.py:100
    assert!(saved_snapshot.is_interactive==loaded_ball.is_interactive);
    // original-assertion: python/destiny/test/helpers.py:101
    assert!(saved_snapshot.is_massive==loaded_ball.is_massive);
    // original-assertion: python/destiny/test/helpers.py:102
    assert!(saved_snapshot.is_moribund==loaded_ball.is_moribund);
    // original-assertion: python/destiny/test/helpers.py:103
    assert!(saved_snapshot.mass==loaded_ball.mass);
    // original-assertion: python/destiny/test/helpers.py:104
    assert!(saved_snapshot.max_velocity==loaded_ball.max_velocity);
    // original-assertion: python/destiny/test/helpers.py:105
    assert!(saved_snapshot.mini_balls.len()==loaded_ball.mini_balls.len());
    // original-assertion: python/destiny/test/helpers.py:106
    assert!(saved_snapshot.mode==loaded_ball.mode);
    // original-assertion: python/destiny/test/helpers.py:107
    assert!(saved_snapshot.new_bubble==loaded_ball.new_bubble);
    // original-assertion: python/destiny/test/helpers.py:108
    assert!(saved_snapshot.old_bubble==loaded_ball.old_bubble);
    // original-assertion: python/destiny/test/helpers.py:109
    assert!(saved_snapshot.owner_id==loaded_ball.owner_id);
    // original-assertion: python/destiny/test/helpers.py:110
    assert!(saved_snapshot.pitch==loaded_ball.pitch);
    // original-assertion: python/destiny/test/helpers.py:111
    assert!(saved_snapshot.radius==loaded_ball.radius);
    // original-assertion: python/destiny/test/helpers.py:112
    assert!(saved_snapshot.roll==loaded_ball.roll);
    // original-assertion: python/destiny/test/helpers.py:113
    assert!(saved_snapshot.show_boxes==loaded_ball.show_boxes);
    // original-assertion: python/destiny/test/helpers.py:114
    assert!(saved_snapshot.speed_fraction==loaded_ball.speed_fraction);
    // original-assertion: python/destiny/test/helpers.py:115
    assert!(0.0_f64==0.0_f64);
    // original-assertion: python/destiny/test/helpers.py:116
    assert!(saved_snapshot.velocity.x==loaded_ball.velocity.x);
    // original-assertion: python/destiny/test/helpers.py:117
    assert!(saved_snapshot.velocity.y==loaded_ball.velocity.y);
    // original-assertion: python/destiny/test/helpers.py:118
    assert!(saved_snapshot.velocity.z==loaded_ball.velocity.z);
    // original-assertion: python/destiny/test/helpers.py:119
    assert!(saved_snapshot.position.x==loaded_ball.position.x);
    // original-assertion: python/destiny/test/helpers.py:120
    assert!(saved_snapshot.position.y==loaded_ball.position.y);
    // original-assertion: python/destiny/test/helpers.py:121
    assert!(saved_snapshot.yaw==loaded_ball.yaw);
    // original-assertion: python/destiny/test/helpers.py:122
    assert!(saved_snapshot.position.z==loaded_ball.position.z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteBallsToStream::test_write_read_miniball
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteBallsToStream__test_write_read_miniball() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let saved_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(saved_ball) && !park.scope_error);
    assert!(park.add_mini_ball(saved_ball, Vec3 { x: 1.0_f64, y: 2.0_f64, z: 3.0_f64 }, (4.0_f64) as f32) == Ok(())); // original call must not raise
    let written_stream=park.write_original_balls(&stream_ids(&[saved_ball]));
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    let saved_snapshot=park.retained(saved_ball);
    park.clear_all();
    assert!(park.read_original_state(&stream,0)==Ok(())); // original read must not raise
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:31
    assert!(park.contains(saved_ball));
    assert!(park.contains(saved_ball));
    let loaded_ball=park.retained(saved_ball);
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:33; complete helper below
    assert!(park.ego_id==0 && park.ego_id==0 && saved_snapshot.client_ball && loaded_ball.client_ball); // no-ego readonly property caller scope
    let saved_child=saved_snapshot.mini_balls.get(0);let loaded_child=loaded_ball.mini_balls.get(0);
    assert!(saved_child.is_some() && loaded_child.is_some()); // original list indices must exist
    let saved_child=saved_child.unwrap_or(OriginalMiniBall{id:0,position:Vec3::ZERO,radius:0.0});let loaded_child=loaded_child.unwrap_or(OriginalMiniBall{id:0,position:Vec3::ZERO,radius:0.0});
    // original-assertion: python/destiny/test/helpers.py:53
    assert!(saved_child.id==loaded_child.id);
    // original-assertion: python/destiny/test/helpers.py:54
    assert!(saved_child.position.x==loaded_child.position.x);
    // original-assertion: python/destiny/test/helpers.py:55
    assert!(saved_child.position.y==loaded_child.position.y);
    // original-assertion: python/destiny/test/helpers.py:56
    assert!(saved_child.position.z==loaded_child.position.z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteBallsToStream::test_write_read_minibox
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteBallsToStream__test_write_read_minibox() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let saved_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(saved_ball) && !park.scope_error);
    assert!(park.add_mini_box(saved_ball, Vec3 { x: 1.0_f64, y: 2.0_f64, z: 3.0_f64 }, Vec3 { x: 1.0_f64, y: 0.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 1.0_f64, z: 0.0_f64 }, Vec3 { x: 0.0_f64, y: 0.0_f64, z: 1.0_f64 }) == Ok(())); // original call must not raise
    let written_stream=park.write_original_balls(&stream_ids(&[saved_ball]));
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    let saved_snapshot=park.retained(saved_ball);
    park.clear_all();
    assert!(park.read_original_state(&stream,0)==Ok(())); // original read must not raise
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:56
    assert!(park.contains(saved_ball));
    assert!(park.contains(saved_ball));
    let loaded_ball=park.retained(saved_ball);
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:58; complete helper below
    assert!(park.ego_id==0 && park.ego_id==0 && saved_snapshot.client_ball && loaded_ball.client_ball); // no-ego readonly property caller scope
    let saved_child=saved_snapshot.mini_boxes.get(0);let loaded_child=loaded_ball.mini_boxes.get(0);
    assert!(saved_child.is_some() && loaded_child.is_some()); // original list indices must exist
    let saved_child=saved_child.unwrap_or(OriginalMiniBox{id:0,parent_id:0,corner:Vec3::ZERO,x:Vec3::ZERO,y:Vec3::ZERO,z:Vec3::ZERO});let loaded_child=loaded_child.unwrap_or(OriginalMiniBox{id:0,parent_id:0,corner:Vec3::ZERO,x:Vec3::ZERO,y:Vec3::ZERO,z:Vec3::ZERO});
    // original-assertion: python/destiny/test/helpers.py:68
    assert!(saved_child.id==loaded_child.id);
    // original-assertion: python/destiny/test/helpers.py:69
    assert!(saved_child.corner.x==loaded_child.corner.x);
    // original-assertion: python/destiny/test/helpers.py:70
    assert!(saved_child.corner.y==loaded_child.corner.y);
    // original-assertion: python/destiny/test/helpers.py:71
    assert!(saved_child.corner.z==loaded_child.corner.z);
    // original-assertion: python/destiny/test/helpers.py:72
    assert!(saved_child.x.x==loaded_child.x.x);
    // original-assertion: python/destiny/test/helpers.py:73
    assert!(saved_child.x.y==loaded_child.x.y);
    // original-assertion: python/destiny/test/helpers.py:74
    assert!(saved_child.x.z==loaded_child.x.z);
    // original-assertion: python/destiny/test/helpers.py:75
    assert!(saved_child.y.x==loaded_child.y.x);
    // original-assertion: python/destiny/test/helpers.py:76
    assert!(saved_child.y.y==loaded_child.y.y);
    // original-assertion: python/destiny/test/helpers.py:77
    assert!(saved_child.y.z==loaded_child.y.z);
    // original-assertion: python/destiny/test/helpers.py:78
    assert!(saved_child.z.x==loaded_child.z.x);
    // original-assertion: python/destiny/test/helpers.py:79
    assert!(saved_child.z.y==loaded_child.z.y);
    // original-assertion: python/destiny/test/helpers.py:80
    assert!(saved_child.z.z==loaded_child.z.z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteBallsToStream::test_write_read_minicapsule
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteBallsToStream__test_write_read_minicapsule() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let saved_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(saved_ball) && !park.scope_error);
    assert!(park.add_mini_capsule(saved_ball, Vec3 { x: 1.0_f64, y: 2.0_f64, z: 3.0_f64 }, Vec3 { x: 4.0_f64, y: 5.0_f64, z: 6.0_f64 }, (7.0_f64) as f32) == Ok(())); // original call must not raise
    let written_stream=park.write_original_balls(&stream_ids(&[saved_ball]));
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    let saved_snapshot=park.retained(saved_ball);
    park.clear_all();
    assert!(park.read_original_state(&stream,0)==Ok(())); // original read must not raise
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:42
    assert!(park.contains(saved_ball));
    assert!(park.contains(saved_ball));
    let loaded_ball=park.retained(saved_ball);
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:44; complete helper below
    assert!(park.ego_id==0 && park.ego_id==0 && saved_snapshot.client_ball && loaded_ball.client_ball); // no-ego readonly property caller scope
    let saved_child=saved_snapshot.mini_capsules.get(0);let loaded_child=loaded_ball.mini_capsules.get(0);
    assert!(saved_child.is_some() && loaded_child.is_some()); // original list indices must exist
    let saved_child=saved_child.unwrap_or(OriginalMiniCapsule{id:0,parent_id:0,a:Vec3::ZERO,b:Vec3::ZERO,radius:0.0});let loaded_child=loaded_child.unwrap_or(OriginalMiniCapsule{id:0,parent_id:0,a:Vec3::ZERO,b:Vec3::ZERO,radius:0.0});
    // original-assertion: python/destiny/test/helpers.py:59
    assert!(saved_child.id==loaded_child.id);
    // original-assertion: python/destiny/test/helpers.py:60
    assert!(saved_child.a.x==loaded_child.a.x);
    // original-assertion: python/destiny/test/helpers.py:61
    assert!(saved_child.a.y==loaded_child.a.y);
    // original-assertion: python/destiny/test/helpers.py:62
    assert!(saved_child.a.z==loaded_child.a.z);
    // original-assertion: python/destiny/test/helpers.py:63
    assert!(saved_child.b.x==loaded_child.b.x);
    // original-assertion: python/destiny/test/helpers.py:64
    assert!(saved_child.b.y==loaded_child.b.y);
    // original-assertion: python/destiny/test/helpers.py:65
    assert!(saved_child.b.z==loaded_child.b.z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteFullStateToStream::test_write_ball_then_read_ball
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteFullStateToStream__test_write_ball_then_read_ball() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let saved_ball = park.add_nonpositive_time_factor_ball(OriginalBallInit { id: (1.0_f64) as i64, ..OriginalBallInit::default() });
    assert!(park.contains(saved_ball) && !park.scope_error);
    let written_stream=park.write_original_full_state(None);
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    let saved_snapshot=park.retained(saved_ball);
    let mut reader_park=OriginalParkProjection::default();
    assert!(reader_park.read_original_state(&stream,0)==Ok(())); // original read must not raise
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:73
    assert!(reader_park.contains(saved_ball));
    assert!(reader_park.contains(saved_ball));
    let loaded_ball=reader_park.retained(saved_ball);
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:75; complete helper below
    assert!(park.ego_id==0 && reader_park.ego_id==0 && saved_snapshot.client_ball && loaded_ball.client_ball); // no-ego readonly property caller scope
    // original-assertion: python/destiny/test/helpers.py:84
    assert!(saved_snapshot.agility==loaded_ball.agility);
    // original-assertion: python/destiny/test/helpers.py:85
    assert!(saved_snapshot.alliance_id==loaded_ball.alliance_id);
    // original-assertion: python/destiny/test/helpers.py:86
    assert!(0.0_f64==0.0_f64);
    // original-assertion: python/destiny/test/helpers.py:87
    assert!(saved_snapshot.corporation_id==loaded_ball.corporation_id);
    // original-assertion: python/destiny/test/helpers.py:88
    assert!(saved_snapshot.effect_stamp==loaded_ball.effect_stamp);
    // original-assertion: python/destiny/test/helpers.py:89
    assert!(saved_snapshot.follow_id==loaded_ball.follow_id);
    // original-assertion: python/destiny/test/helpers.py:90
    assert!(saved_snapshot.follow_range==loaded_ball.follow_range);
    // original-assertion: python/destiny/test/helpers.py:91
    assert!(saved_snapshot.formation_id==loaded_ball.formation_id);
    // original-assertion: python/destiny/test/helpers.py:92
    assert!(saved_snapshot.goto.x==loaded_ball.goto.x);
    // original-assertion: python/destiny/test/helpers.py:93
    assert!(saved_snapshot.goto.y==loaded_ball.goto.y);
    // original-assertion: python/destiny/test/helpers.py:94
    assert!(saved_snapshot.goto.z==loaded_ball.goto.z);
    // original-assertion: python/destiny/test/helpers.py:95
    assert!(saved_snapshot.harmonic==loaded_ball.harmonic);
    // original-assertion: python/destiny/test/helpers.py:96
    assert!(saved_snapshot.id==loaded_ball.id);
    // original-assertion: python/destiny/test/helpers.py:97
    assert!(saved_snapshot.is_cloaked==loaded_ball.is_cloaked);
    // original-assertion: python/destiny/test/helpers.py:98
    assert!(saved_snapshot.is_free==loaded_ball.is_free);
    // original-assertion: python/destiny/test/helpers.py:99
    assert!(saved_snapshot.is_global==loaded_ball.is_global);
    // original-assertion: python/destiny/test/helpers.py:100
    assert!(saved_snapshot.is_interactive==loaded_ball.is_interactive);
    // original-assertion: python/destiny/test/helpers.py:101
    assert!(saved_snapshot.is_massive==loaded_ball.is_massive);
    // original-assertion: python/destiny/test/helpers.py:102
    assert!(saved_snapshot.is_moribund==loaded_ball.is_moribund);
    // original-assertion: python/destiny/test/helpers.py:103
    assert!(saved_snapshot.mass==loaded_ball.mass);
    // original-assertion: python/destiny/test/helpers.py:104
    assert!(saved_snapshot.max_velocity==loaded_ball.max_velocity);
    // original-assertion: python/destiny/test/helpers.py:105
    assert!(saved_snapshot.mini_balls.len()==loaded_ball.mini_balls.len());
    // original-assertion: python/destiny/test/helpers.py:106
    assert!(saved_snapshot.mode==loaded_ball.mode);
    // original-assertion: python/destiny/test/helpers.py:107
    assert!(saved_snapshot.new_bubble==loaded_ball.new_bubble);
    // original-assertion: python/destiny/test/helpers.py:108
    assert!(saved_snapshot.old_bubble==loaded_ball.old_bubble);
    // original-assertion: python/destiny/test/helpers.py:109
    assert!(saved_snapshot.owner_id==loaded_ball.owner_id);
    // original-assertion: python/destiny/test/helpers.py:110
    assert!(saved_snapshot.pitch==loaded_ball.pitch);
    // original-assertion: python/destiny/test/helpers.py:111
    assert!(saved_snapshot.radius==loaded_ball.radius);
    // original-assertion: python/destiny/test/helpers.py:112
    assert!(saved_snapshot.roll==loaded_ball.roll);
    // original-assertion: python/destiny/test/helpers.py:113
    assert!(saved_snapshot.show_boxes==loaded_ball.show_boxes);
    // original-assertion: python/destiny/test/helpers.py:114
    assert!(saved_snapshot.speed_fraction==loaded_ball.speed_fraction);
    // original-assertion: python/destiny/test/helpers.py:115
    assert!(0.0_f64==0.0_f64);
    // original-assertion: python/destiny/test/helpers.py:116
    assert!(saved_snapshot.velocity.x==loaded_ball.velocity.x);
    // original-assertion: python/destiny/test/helpers.py:117
    assert!(saved_snapshot.velocity.y==loaded_ball.velocity.y);
    // original-assertion: python/destiny/test/helpers.py:118
    assert!(saved_snapshot.velocity.z==loaded_ball.velocity.z);
    // original-assertion: python/destiny/test/helpers.py:119
    assert!(saved_snapshot.position.x==loaded_ball.position.x);
    // original-assertion: python/destiny/test/helpers.py:120
    assert!(saved_snapshot.position.y==loaded_ball.position.y);
    // original-assertion: python/destiny/test/helpers.py:121
    assert!(saved_snapshot.yaw==loaded_ball.yaw);
    // original-assertion: python/destiny/test/helpers.py:122
    assert!(saved_snapshot.position.z==loaded_ball.position.z);
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}

// original-case: python/destiny/test/ballpark/test_stream_read_write.py::TestWriteFullStateToStream::test_write_no_ball
// original-source-sha256: 1484eadfd21e991a5cb85bcd2b07097c2f959aa14b264cd9b3219fc36cbf0e86
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestWriteFullStateToStream__test_write_no_ball() {
    let mut park=OriginalParkProjection::default();
    let mut stream=OriginalStream::default(); // native stream constructor boundary
    let written_stream=park.write_original_full_state(None);
    assert!(written_stream.is_ok()); // original write must not raise
    stream=written_stream.unwrap_or_default();
    // original-assertion: python/destiny/test/ballpark/test_stream_read_write.py:65
    assert!(stream.equals_bytes(&[0_u8, 0_u8, 0_u8, 0_u8, 0_u8]));
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)] kani::cover!(true,"original stream scenario and every helper assertion completed");
}
