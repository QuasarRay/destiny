// Executable differential oracle: uses ACTUAL original Destiny headers.
// Vector3 is an external Blue type required by Vector3d.h; its minimal
// declaration here supplies only that upstream dependency, not replacements
// for any original Destiny math operators.
#include <cmath>
#include <cstdint>
#include <cstring>
#include <cstddef>
#include <string>
#include <iomanip>
#include <iostream>
struct Vector3 {
    float x, y, z;
    Vector3(float a, float b, float c): x(a), y(b), z(c) {}
};
struct PyObject;
#include "src/Vector3d.h"
#include "src/Quaternion.h"
#include "src/Partition.h"

void output(const char* name, double a, double b, double c, double d) {
    std::cout << name << " " << std::setprecision(17) << a << " " << b << " "
              << c << " " << d << "\n";
}
void output_q(const char* name, QuaternionD q) {
    output(name, q.w, q.v.x, q.v.y, q.v.z);
}
int main() {
    const QuaternionD a(2.0, 3.0, -5.0, 7.0);
    const QuaternionD b(11.0, -13.0, 17.0, 19.0);
    output_q("product", a * b);
    output_q("inverse", -a);
    output_q("sum", a + b);
    output_q("difference", a - b);
    output("length", a.LengthSq(), 0, 0, 0);
    const Vector3d rotated = a.RotateVector(Vector3d(4.0, 5.0, -1.0));
    output("rotate", rotated.x, rotated.y, rotated.z, 0);
    output_q("zero_axis", QuaternionD(Vector3d(0.0, 0.0, 0.0), 1.5));
    output_q("nonunit_axis", QuaternionD(Vector3d(3.0, 4.0, 0.0), 1.5));
    output_q("default", QuaternionD());
    Key key(-1, 99, 2, 8);
    output("clamp", (double)key.ix, (double)key.iy, (double)key.iz, (double)key.n);
    std::cout << "key_hash " << KeyHash{}(key) << " 0 0 0\n";
    Key key_zero(0, 0, 0, 0);
    output("zero_extent", (double)key_zero.ix, (double)key_zero.iy,
           (double)key_zero.iz, (double)key_zero.n);

    output_q("four_scalar_ctor", QuaternionD(1.25, -2.5, 3.75, -4.0));
    output_q("scalar_vector_ctor", QuaternionD(-0.25, Vector3d(1.5, 0.0, -7.0)));
    output_q("unit_axis", QuaternionD(Vector3d(1.0, 0.0, 0.0), 1.25));
    output_q("negative_angle", QuaternionD(Vector3d(-3.0, 4.0, 0.0), -1.75));
    output_q("angle_zero", QuaternionD(Vector3d(3.0, 4.0, 0.0), 0.0));
    QuaternionD mut = a;
    mut += b;
    output_q("add_assign", mut);
    mut -= b;
    output_q("sub_assign", mut);
    output_q("division_positive", a / 2.5);
    output_q("division_negative", a / -3.25);
    output_q("division_zero", a / 0.0);
    output_q("negative_zero_quaternion", -QuaternionD(0.0, 0.0, 0.0, 0.0));
    output("equal_same", a == a ? 1.0 : 0.0, a != a ? 1.0 : 0.0, 0, 0);
    output("equal_other", a == b ? 1.0 : 0.0, a != b ? 1.0 : 0.0, 0, 0);
    const QuaternionD zero_plus(0.0, -0.0, 0.0, -0.0);
    const QuaternionD zero_minus(-0.0, 0.0, -0.0, 0.0);
    output("signed_zero_eq", zero_plus == zero_minus ? 1.0 : 0.0,
           zero_plus != zero_minus ? 1.0 : 0.0, 0, 0);
    output("layout", sizeof(QuaternionD), alignof(QuaternionD),
           offsetof(QuaternionD, v), sizeof(Vector3d));
    std::uint64_t state = 0x9e3779b97f4a7c15ULL;
    auto random = [&state]() -> double {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        const std::uint64_t bits = state * 2685821657736338717ULL;
        const std::int64_t integer = static_cast<std::int64_t>(bits & 0xfffffULL) - 0x80000LL;
        return static_cast<double>(integer) / 16384.0;
    };
    for (int i = 0; i < 96; ++i) {
        const double aw = random(), ax = random(), ay = random(), az = random();
        const double bw = random(), bx = random(), by = random(), bz = random();
        const QuaternionD x(aw, ax, ay, az), y(bw, bx, by, bz);
        const std::string suffix = std::to_string(i);
        output_q(("random_mul_" + suffix).c_str(), x * y);
        output_q(("random_add_" + suffix).c_str(), x + y);
        output_q(("random_sub_" + suffix).c_str(), x - y);
        output_q(("random_inverse_" + suffix).c_str(), -x);
        output_q(("random_div_" + suffix).c_str(), x / 2.5);
        QuaternionD assigned = x;
        assigned += y;
        assigned -= y;
        output_q(("random_assign_" + suffix).c_str(), assigned);
        output(("random_len_" + suffix).c_str(), x.LengthSq(), 0, 0, 0);
        const double vx = random(), vy = random(), vz = random();
        const Vector3d result = x.RotateVector(Vector3d(vx, vy, vz));
        output(("random_rotate_" + suffix).c_str(), result.x, result.y, result.z, 0);
    }
    return 0;
}
