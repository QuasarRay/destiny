// Copyright © 2015 CCP ehf. Original tests' MIT license retained.
// Typed Rust obligations for native construction. Python/Blue factory dispatch,
// module registration and wrapper type identity require actual boundary runs.
#![allow(non_snake_case)]
use destiny_original_spec::{OriginalLifetimeProjection,OriginalObjectClass,OriginalBallInit,Vec3,
    OriginalPythonVectorLists,OriginalParkBall,python_almost_equal};

// original-case: python/destiny/test/test_ball.py::BallTest::test_can_construct
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn BallTest__test_can_construct() {
    let mut world=OriginalLifetimeProjection::default();
    world.register_destiny_classes();
    assert!(world.create_park(false)==Ok(())); // actual fixture default client park
    let b=world.create_detached_ball(OriginalObjectClass::Ball);
    assert!(b.is_ok()); // original assertIsNotNone + no construction exception
    let b=b.expect("checked concrete constructor result");
    assert!(world.native_class(b)==Some(OriginalObjectClass::Ball));
    assert!(world.drop_ball_reference(b)==Ok(()));
    assert!(world.finish_scope()==Ok(())); // fixture ClearAll and local releases
    #[cfg(kani)] kani::cover!(true,"typed original Ball constructor scenario complete; native boundary pending");
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBallpark::test_can_create_ballpark
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestBallpark__test_can_create_ballpark() {
    let mut world=OriginalLifetimeProjection::default();
    world.register_destiny_classes();
    let result=world.create_park(false); // BallparkTestCase.setUp uses native default
    assert!(result==Ok(()));
    assert!(world.has_park()); // original assertIsNotNone(self.park)
    assert!(world.finish_scope()==Ok(())); // original tearDown ClearAll
    #[cfg(kani)] kani::cover!(true,"typed original Ballpark constructor scenario complete; native boundary pending");
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestBallpark::test_add_ball_creates_a_client_ball
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn TestBallpark__test_add_ball_creates_a_client_ball() {
    let mut world=OriginalLifetimeProjection::default();
    world.register_destiny_classes();
    assert!(world.create_park(false)==Ok(()));
    // Original helpers.BallPOPO defaults (helpers.py:158..176), including id
    // zero, detached sentinel coordinates, and the zero-agility caller input.
    let ball=world.add_ball(OriginalBallInit{id:0,mass:0.0,radius:0.0,max_velocity:0.0,
        is_free:false,is_global:false,is_massive:false,is_interactive:false,is_space_junk:false,
        position:Vec3{x:10_000_000_000.0,y:10_000_000_000.0,z:10_000_000_000.0},
        velocity:Vec3::ZERO,agility:0.0,speed_fraction:0.0});
    assert!(ball.is_ok());
    let ball=ball.expect("checked original helper AddBall result");
    let comparator=world.create_detached_ball(OriginalObjectClass::ClientBall);
    assert!(comparator.is_ok());
    let comparator=comparator.expect("checked original ClientBall constructor");
    assert!(world.native_class(ball)==world.native_class(comparator)); // type(ball) == type(ClientBall())
    assert!(world.drop_ball_reference(comparator)==Ok(())); // temporary comparison object
    assert!(world.drop_ball_reference(ball)==Ok(()));
    assert!(world.finish_scope()==Ok(()));
    #[cfg(kani)] kani::cover!(true,"source-derived native class selection and equality scenario complete; native boundary pending");
}

// original-case: python/destiny/test/test_ball.py::BallTest::test_get_rotated_vector_returns_original_vector_if_there_is_no_rotation
#[cfg_attr(kani,kani::proof)]
#[cfg_attr(kani,kani::unwind(129))]
#[cfg_attr(test,test)]
fn BallTest__test_get_rotated_vector_returns_original_vector_if_there_is_no_rotation() {
    let mut world=OriginalLifetimeProjection::default();
    world.register_destiny_classes();
    assert!(world.create_park(false)==Ok(()));
    let b=world.create_detached_ball(OriginalObjectClass::Ball);
    assert!(b.is_ok());
    let b=b.expect("checked original detached Ball construction");
    let mut lists=OriginalPythonVectorLists::default();
    let original_vector=lists.allocate([1.0,2.0,3.0]);
    assert!(original_vector.is_ok());
    let original_vector=original_vector.expect("checked concrete original float list");
    let rotated_vector=lists.get_rotated_vector(&OriginalParkBall::default(),original_vector);
    assert!(rotated_vector.is_ok());
    let rotated_vector=rotated_vector.expect("checked original vector call");
    assert!(rotated_vector==original_vector); // source alias relation, stronger than original numeric check
    // Complete helpers.py:124..127, after both original names observe the
    // same list's actual binary32 rotation writes. Do not compare a saved copy.
    let first=lists.values(original_vector);let second=lists.values(rotated_vector);
    assert!(first.is_some() && second.is_some());
    let first=first.expect("checked original shared list");let second=second.expect("checked returned shared list");
    assert!(first.len()==second.len());
    let mut i=0;
    while i<first.len() { assert!(python_almost_equal(first[i],second[i],7)==Some(true));i+=1; }
    assert!(world.drop_ball_reference(b)==Ok(()));
    assert!(world.finish_scope()==Ok(()));
    #[cfg(kani)] kani::cover!(true,"original in-place rotation and full shared-list assertion scenario complete; native boundary pending");
}
