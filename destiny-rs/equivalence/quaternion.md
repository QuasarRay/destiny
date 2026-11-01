# QuaternionD — original header correspondence

Original source: src/Quaternion.h; vector operations: src/Vector3d.h.
Rust source: destiny-rs/src/quaternion.rs.

**Declaration-level scope:** one original value type maps to exactly one Rust value type, with the same two fields. The original header provides four constructors, nine operator overloads and two named methods; all are represented by a Rust constructor/default, operator trait or original-named method.

| Original C++ operation | Rust counterpart |
|---|---|
| QuaternionD() | Default::default() |
| QuaternionD(double,double,double,double) | QuaternionD::new |
| QuaternionD(Vector3d,double) | QuaternionD::from_axis_angle |
| QuaternionD(double,const Vector3d&) | QuaternionD::from_scalar_vector |
| operator+= | AddAssign::add_assign |
| operator-= | SubAssign::sub_assign |
| unary operator- | Neg::neg (INVERSE, not negative quaternion) |
| operator/(double) | Div<f64>::div |
| operator*(QuaternionD) | Mul::mul |
| binary operator+ | Add::add |
| binary operator- | Sub::sub |
| operator== | PartialEq::eq |
| operator!= | PartialEq::ne |
| LengthSq() | QuaternionD::LengthSq |
| RotateVector(const Vector3d&) | QuaternionD::RotateVector |

**Bevy boundary:** The C++ Vector3d field maps to Bevy DVec3. The source has no ECS Component, System, Event or Relationship, so adding one would violate structural correspondence. Native DQuat conversion trait implementations are explicit Bevy interop adapters and never replace QuaternionD arithmetic.

**Structural layout:** On the CI target, the Rust type is repr(C), with w at byte 0, v at byte 8, and size 32. Both C++ and Rust report size, alignment and offset in the differential test. This is a checked target-specific ABI observation, not a universal cross-platform guarantee.

**Validation:** The source guard in tools/tests/test_quaternion_header.py checks all header declarations. The original C++/Rust differential runner compares every operation and a deterministic 96-input arithmetic corpus; the Rust quaternion_contract.rs suite covers special cases. Finite comparison uses numeric tolerances and NaNs are checked by category. These tests are not exhaustive mathematical proofs; 100% universal runtime equivalence remains unproven for all floating-point environments and compiler targets.

Rust constructor overloading and C++ operator declaration syntax are language-level exceptions: original constructor/method ownership remains one-to-one despite distinct syntax.
