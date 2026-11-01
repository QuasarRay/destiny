use bevy::math::DVec3;
use destiny_rs::{Key, QuaternionD};

#[test]
fn key_constructor_and_clamp_preserve_cpp_order_and_branching() {
    let mut key = Key::new(-1, 99, 2, 8);
    assert_eq!((key.ix, key.iy, key.iz, key.n), (0, 7, 2, 8));
    key.ix = 10;
    key.iy = -23;
    key.iz = 6;
    key.Clamp();
    assert_eq!((key.ix, key.iy, key.iz), (7, 0, 6));
    assert_eq!(Key::default(), Key { ix: 0, iy: 0, iz: 0, n: 0 });
    // Key(int64_t,int64_t,int64_t,n=0) invokes Clamp even though default Key() does not.
    assert_eq!(Key::new(0, 0, 0, 0), Key { ix: -1, iy: -1, iz: -1, n: 0 });
}

#[test]
fn key_order_matches_cpp_key_operator_less_than() {
    let sorted = [
        Key { ix: 0, iy: 0, iz: 0, n: 1 },
        Key { ix: 0, iy: 0, iz: 0, n: 2 },
        Key { ix: 0, iy: 0, iz: 1, n: 2 },
        Key { ix: 0, iy: 1, iz: 0, n: 2 },
        Key { ix: 1, iy: 0, iz: 0, n: 2 },
    ];
    assert!(sorted.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn quaternion_original_method_names_are_callable() {
    let q = QuaternionD::new(2.0, -1.0, 3.0, -4.0);
    assert_eq!(q.LengthSq(), 30.0);
    assert_eq!(q.length_squared(), q.LengthSq());
    let v = DVec3::new(4.0, 5.0, -1.0);
    assert_eq!(q.RotateVector(v), q.rotate_vector(v));
}

#[test]
fn quaternion_hamilton_product_follows_cpp_scalar_formula() {
    let lhs = QuaternionD::new(2.0, 3.0, -5.0, 7.0);
    let rhs = QuaternionD::new(11.0, -13.0, 17.0, 19.0);
    let expected = QuaternionD::new(
        2.0 * 11.0 - 3.0 * -13.0 - -5.0 * 17.0 - 7.0 * 19.0,
        2.0 * -13.0 + 3.0 * 11.0 + -5.0 * 19.0 - 7.0 * 17.0,
        2.0 * 17.0 + -5.0 * 11.0 + 7.0 * -13.0 - 3.0 * 19.0,
        2.0 * 19.0 + 7.0 * 11.0 + 3.0 * 17.0 - -5.0 * -13.0,
    );
    assert_eq!(lhs * rhs, expected);
}

#[test]
fn unary_minus_is_cpp_inverse_not_quaternion_negation() {
    let q = QuaternionD::new(1.0, 2.0, 3.0, 4.0);
    let inverted = -q;
    assert_eq!(inverted, QuaternionD::new(1.0 / 30.0, -2.0 / 30.0, -3.0 / 30.0, -4.0 / 30.0));
    assert_ne!(inverted, QuaternionD::new(-1.0, -2.0, -3.0, -4.0));
}
