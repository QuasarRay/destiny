//! Deterministic values from the Rust side of the original-header oracle.
use bevy::math::DVec3;
use destiny_rs::{Key, QuaternionD};

fn output(name: &str, a: f64, b: f64, c: f64, d: f64) {
    println!("{name} {a:.17} {b:.17} {c:.17} {d:.17}");
}
fn output_q(name: &str, q: QuaternionD) {
    output(name, q.w, q.v.x, q.v.y, q.v.z);
}
fn main() {
    let a = QuaternionD::new(2.0, 3.0, -5.0, 7.0);
    let b = QuaternionD::new(11.0, -13.0, 17.0, 19.0);
    output_q("product", a * b);
    output_q("inverse", -a);
    output_q("sum", a + b);
    output_q("difference", a - b);
    output("length", a.LengthSq(), 0.0, 0.0, 0.0);
    let rotated = a.RotateVector(DVec3::new(4.0, 5.0, -1.0));
    output("rotate", rotated.x, rotated.y, rotated.z, 0.0);
    output_q("zero_axis", QuaternionD::from_axis_angle(DVec3::ZERO, 1.5));
    output_q("nonunit_axis", QuaternionD::from_axis_angle(DVec3::new(3.0, 4.0, 0.0), 1.5));
    output_q("default", QuaternionD::default());
    let key = Key::new(-1, 99, 2, 8);
    output("clamp", key.ix as f64, key.iy as f64, key.iz as f64, key.n as f64);
    let key_zero = Key::new(0, 0, 0, 0);
    output("zero_extent", key_zero.ix as f64, key_zero.iy as f64, key_zero.iz as f64, key_zero.n as f64);
}
