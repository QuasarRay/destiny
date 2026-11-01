// Executable differential oracle: uses ACTUAL original Destiny headers.
// Vector3 is an external Blue type required by Vector3d.h; its minimal
// declaration here supplies only that upstream dependency, not replacements
// for any original Destiny math operators.
#include <cmath>
#include <cstdint>
#include <cstring>
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
    Key key_zero(0, 0, 0, 0);
    output("zero_extent", (double)key_zero.ix, (double)key_zero.iy,
           (double)key_zero.iz, (double)key_zero.n);
    return 0;
}
