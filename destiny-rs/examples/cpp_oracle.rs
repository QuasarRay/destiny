//! Deterministic values from the Rust side of the original-header oracle.
use bevy::math::DVec3;
use destiny_rs::{Key, KeyHash, QuaternionD};

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
    let rotated = a.RotateVector(&DVec3::new(4.0, 5.0, -1.0));
    output("rotate", rotated.x, rotated.y, rotated.z, 0.0);
    output_q("zero_axis", QuaternionD::from_axis_angle(DVec3::ZERO, 1.5));
    output_q("nonunit_axis", QuaternionD::from_axis_angle(DVec3::new(3.0, 4.0, 0.0), 1.5));
    output_q("default", QuaternionD::default());
    let key = Key::new(-1, 99, 2, 8);
    output("clamp", key.ix as f64, key.iy as f64, key.iz as f64, key.n as f64);
    println!("key_hash {} 0 0 0", KeyHash.hash(&key));
    let key_zero = Key::new(0, 0, 0, 0);
    output("zero_extent", key_zero.ix as f64, key_zero.iy as f64, key_zero.iz as f64, key_zero.n as f64);
    output_q("four_scalar_ctor", QuaternionD::new(1.25, -2.5, 3.75, -4.0));
    output_q("scalar_vector_ctor", QuaternionD::from_scalar_vector(
        -0.25, &DVec3::new(1.5, 0.0, -7.0),
    ));
    output_q("unit_axis", QuaternionD::from_axis_angle(DVec3::new(1.0, 0.0, 0.0), 1.25));
    output_q("negative_angle", QuaternionD::from_axis_angle(DVec3::new(-3.0, 4.0, 0.0), -1.75));
    output_q("angle_zero", QuaternionD::from_axis_angle(DVec3::new(3.0, 4.0, 0.0), 0.0));
    let mut mutation = a;
    mutation += b;
    output_q("add_assign", mutation);
    mutation -= b;
    output_q("sub_assign", mutation);
    output_q("division_positive", a / 2.5);
    output_q("division_negative", a / -3.25);
    output_q("division_zero", a / 0.0);
    output_q("negative_zero_quaternion", -QuaternionD::new(0.0, 0.0, 0.0, 0.0));
    output("equal_same", (a == a) as i32 as f64, (a != a) as i32 as f64, 0.0, 0.0);
    output("equal_other", (a == b) as i32 as f64, (a != b) as i32 as f64, 0.0, 0.0);
    let zero_plus = QuaternionD::new(0.0, -0.0, 0.0, -0.0);
    let zero_minus = QuaternionD::new(-0.0, 0.0, -0.0, 0.0);
    output("signed_zero_eq", (zero_plus == zero_minus) as i32 as f64,
        (zero_plus != zero_minus) as i32 as f64, 0.0, 0.0);
    output("layout", std::mem::size_of::<QuaternionD>() as f64,
        std::mem::align_of::<QuaternionD>() as f64,
        std::mem::offset_of!(QuaternionD, v) as f64,
        std::mem::size_of::<DVec3>() as f64);

    let mut state = 0x9e3779b97f4a7c15_u64;
    fn random(state: &mut u64) -> f64 {
        *state ^= *state >> 12;
        *state ^= *state << 25;
        *state ^= *state >> 27;
        let bits = (*state).wrapping_mul(2685821657736338717);
        let integer = (bits & 0xfffff) as i64 - 0x80000;
        integer as f64 / 16384.0
    }
    for i in 0..96 {
        let aw = random(&mut state); let ax = random(&mut state);
        let ay = random(&mut state); let az = random(&mut state);
        let bw = random(&mut state); let bx = random(&mut state);
        let by = random(&mut state); let bz = random(&mut state);
        let x = QuaternionD::new(aw, ax, ay, az);
        let y = QuaternionD::new(bw, bx, by, bz);
        output_q(&format!("random_mul_{i}"), x * y);
        output_q(&format!("random_add_{i}"), x + y);
        output_q(&format!("random_sub_{i}"), x - y);
        output_q(&format!("random_inverse_{i}"), -x);
        output_q(&format!("random_div_{i}"), x / 2.5);
        let mut assigned = x;
        assigned += y;
        assigned -= y;
        output_q(&format!("random_assign_{i}"), assigned);
        output(&format!("random_len_{i}"), x.LengthSq(), 0.0, 0.0, 0.0);
        let vx = random(&mut state); let vy = random(&mut state); let vz = random(&mut state);
        let result = x.RotateVector(&DVec3::new(vx, vy, vz));
        output(&format!("random_rotate_{i}"), result.x, result.y, result.z, 0.0);
    }
}
