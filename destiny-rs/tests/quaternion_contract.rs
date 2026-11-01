use bevy::math::{DQuat, DVec3};
use destiny_rs::QuaternionD;
use std::mem::{align_of, offset_of, size_of};

#[test]
fn quaternion_cpp_field_order_and_binary_layout() {
    assert_eq!(size_of::<QuaternionD>(), 32);
    assert_eq!(align_of::<QuaternionD>(), 8);
    assert_eq!(offset_of!(QuaternionD, w), 0);
    assert_eq!(offset_of!(QuaternionD, v), 8);
    assert_eq!(size_of::<DVec3>(), 24);
    assert_eq!(align_of::<DVec3>(), 8);
}

#[test]
fn all_four_original_cpp_constructors() {
    assert_eq!(QuaternionD::default(), QuaternionD::new(1.0, 0.0, 0.0, 0.0));
    let v = DVec3::new(2.0, -3.0, 4.0);
    assert_eq!(QuaternionD::from_scalar_vector(5.0, &v), QuaternionD::new(5.0, 2.0, -3.0, 4.0));
    let unit = QuaternionD::from_axis_angle(DVec3::X, 0.0);
    assert_eq!(unit, QuaternionD::default());
    let zero = QuaternionD::from_axis_angle(DVec3::ZERO, 1.5);
    assert_eq!(zero.v, DVec3::ZERO);
    assert!(zero.w.is_finite());
}

#[test]
fn nine_original_cpp_operator_overloads() {
    let a = QuaternionD::new(2.0, 3.0, 5.0, 7.0);
    let b = QuaternionD::new(11.0, 13.0, 17.0, 19.0);
    let mut result = a;
    result += b;
    assert_eq!(result, a + b);
    result -= b;
    assert_eq!(result, a);
    assert_eq!(a / 2.0, QuaternionD::new(1.0, 1.5, 2.5, 3.5));
    assert_ne!(a * b, b * a);
    assert_eq!(a - a, QuaternionD::new(0.0, 0.0, 0.0, 0.0));
    assert!(a == a);
    assert!(!(a != a));
    let inv = -a;
    assert!(inv.w.is_finite());
    assert!(((a * inv).w - 1.0).abs() < 1e-14);
}

#[test]
fn source_ieee754_edge_behavior() {
    let origin = QuaternionD::new(0.0, 0.0, 0.0, 0.0);
    let inverse = -origin;
    assert!(inverse.w.is_nan());
    assert!(inverse.v.x.is_nan() && inverse.v.y.is_nan() && inverse.v.z.is_nan());
    let divided = QuaternionD::new(1.0, 2.0, 3.0, 4.0) / 0.0;
    assert!(divided.w.is_infinite());
    let a = QuaternionD::new(0.0, -0.0, 0.0, -0.0);
    let b = QuaternionD::new(-0.0, 0.0, -0.0, 0.0);
    assert!(a == b);
    assert!(!(a != b));
    let nan = QuaternionD::new(f64::NAN, 0.0, 0.0, 0.0);
    assert!(!(nan == nan));
    assert!(nan != nan);
}

#[test]
fn native_bevy_boundary_preserves_exact_stored_components() {
    let original = QuaternionD::new(2.5, -3.0, 5.25, -7.0);
    let native: DQuat = original.into();
    let roundtrip = QuaternionD::from(native);
    assert_eq!(roundtrip.w, original.w);
    assert_eq!(roundtrip.v, original.v);
}

#[test]
fn named_methods_have_original_owner_and_vector_by_reference() {
    let q = QuaternionD::new(2.0, 1.0, -3.0, 4.0);
    assert_eq!(q.LengthSq(), 30.0);
    let v = DVec3::new(4.0, 5.0, -1.0);
    assert!(q.RotateVector(&v).is_finite());
}
