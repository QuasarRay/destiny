use destiny_original_spec::{Aabb, Plane, Triangle, Vec3};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

// original-test: tests/TestVector3d.cpp::InitializesToZero
#[cfg(kani)]
#[kani::proof]
fn vector_initializes_to_zero() {
    assert_eq!(Vec3::default(), v(0.0, 0.0, 0.0));
}

// original-test: tests/TestVector3d.cpp::Subtraction
#[cfg(kani)]
#[kani::proof]
fn vector_subtraction_matches_original_case() {
    assert_eq!(v(1.0, 2.0, 3.0).sub(v(0.0, 1.0, 2.0)), v(1.0, 1.0, 1.0));
}

// original-test: tests/TestVector3d.cpp::ParallelVectors
#[cfg(kani)]
#[kani::proof]
fn parallel_vector_cross_product_is_zero() {
    assert_eq!(v(2.0, 4.0, 6.0).cross(v(1.0, 2.0, 3.0)), Vec3::ZERO);
}

// original-test: tests/TestVector3d.cpp::OrthogonalVectors
#[cfg(kani)]
#[kani::proof]
fn orthogonal_vector_cross_product_is_positive_z() {
    assert_eq!(v(1.0, 0.0, 0.0).cross(v(0.0, 1.0, 0.0)), v(0.0, 0.0, 1.0));
}

// original-test: tests/TestAABB.cpp::DefaultConstructor
#[cfg(kani)]
#[kani::proof]
fn aabb_default_constructor_is_zero_box() {
    let b = Aabb::default();
    assert_eq!(b.low, Vec3::ZERO);
    assert_eq!(b.high, Vec3::ZERO);
}

// original-test: tests/TestAABB.cpp::ConstructorWithParameters
#[cfg(kani)]
#[kani::proof]
fn aabb_parameter_constructor_preserves_corners() {
    let b = Aabb::new(v(1.0, 2.0, 3.0), v(4.0, 5.0, 6.0));
    assert_eq!(b.low, v(1.0, 2.0, 3.0));
    assert_eq!(b.high, v(4.0, 5.0, 6.0));
}

// original-test: tests/TestAABB.cpp::High
#[cfg(kani)]
#[kani::proof]
fn aabb_update_expands_high_corner() {
    let mut b = Aabb::default();
    b.update(v(1.0, 2.0, 3.0));
    assert_eq!(b.high, v(1.0, 2.0, 3.0));
}

// original-test: tests/TestAABB.cpp::Low
#[cfg(kani)]
#[kani::proof]
fn aabb_update_expands_low_corner() {
    let mut b = Aabb::default();
    b.update(v(-1.0, -2.0, -3.0));
    assert_eq!(b.low, v(-1.0, -2.0, -3.0));
}

// original-test: tests/TestAABB.cpp::HighIgnore
#[cfg(kani)]
#[kani::proof]
fn aabb_update_inside_does_not_reduce_high_corner() {
    let mut b = Aabb::new(Vec3::ZERO, v(5.0, 5.0, 5.0));
    b.update(v(1.0, 2.0, 3.0));
    assert_eq!(b.high, v(5.0, 5.0, 5.0));
}

// original-test: tests/TestAABB.cpp::LowIgnore
#[cfg(kani)]
#[kani::proof]
fn aabb_update_inside_does_not_raise_low_corner() {
    let mut b = Aabb::new(v(-5.0, -5.0, -5.0), Vec3::ZERO);
    b.update(v(-1.0, -2.0, -3.0));
    assert_eq!(b.low, v(-5.0, -5.0, -5.0));
}

fn unit_aabb() -> Aabb {
    Aabb::new(Vec3::ZERO, v(1.0, 1.0, 1.0))
}

// original-test: tests/TestAABB.cpp::ExcludeReturnsFalseWhenRadiusIntersects
#[cfg(kani)]
#[kani::proof]
fn aabb_does_not_exclude_when_swept_radius_intersects() {
    assert!(!unit_aabb().can_exclude_collision(
        v(-1.0, -1.0, 0.0), v(-1.0, 1.0, 0.0), 1.0
    ));
}

// original-test: tests/TestAABB.cpp::ExcludeLeft
#[cfg(kani)]
#[kani::proof]
fn aabb_excludes_sweep_left() {
    assert!(unit_aabb().can_exclude_collision(
        v(-1.0, -1.0, 0.0), v(-1.0, 1.0, 0.0), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::ExcludeRight
#[cfg(kani)]
#[kani::proof]
fn aabb_excludes_sweep_right() {
    assert!(unit_aabb().can_exclude_collision(
        v(2.0, -1.0, 0.0), v(2.0, 1.0, 0.0), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::ExcludeTop
#[cfg(kani)]
#[kani::proof]
fn aabb_excludes_sweep_top() {
    assert!(unit_aabb().can_exclude_collision(
        v(-1.0, 2.0, 0.0), v(1.0, 2.0, 0.0), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::ExcludeBottom
#[cfg(kani)]
#[kani::proof]
fn aabb_excludes_sweep_bottom() {
    assert!(unit_aabb().can_exclude_collision(
        v(-1.0, -1.0, 0.0), v(1.0, -1.0, 0.0), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::ExcludeFront
#[cfg(kani)]
#[kani::proof]
fn aabb_excludes_sweep_front() {
    assert!(unit_aabb().can_exclude_collision(
        v(-1.0, 0.0, 2.0), v(1.0, 0.0, 2.0), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::ExcludeBack
#[cfg(kani)]
#[kani::proof]
fn aabb_excludes_sweep_back() {
    assert!(unit_aabb().can_exclude_collision(
        v(-1.0, 0.0, -1.0), v(1.0, 0.0, -1.0), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::InsideBox
#[cfg(kani)]
#[kani::proof]
fn aabb_does_not_exclude_sphere_inside_box() {
    assert!(!unit_aabb().can_exclude_collision(
        v(0.5, 0.5, 0.5), v(0.5, 0.5, 0.5), 0.5
    ));
}

// original-test: tests/TestAABB.cpp::ZeroSize
#[cfg(kani)]
#[kani::proof]
fn zero_aabb_longest_width_is_zero() {
    assert_eq!(Aabb::default().longest_width().to_bits(), 0.0_f64.to_bits());
}

// original-test: tests/TestAABB.cpp::XAxisIsLongest
#[cfg(kani)]
#[kani::proof]
fn aabb_reports_x_as_longest_width_case() {
    let b = Aabb::new(v(-0.5, -0.5, -0.5), v(1.0, 0.5, 0.5));
    assert_eq!(b.longest_width().to_bits(), 1.5_f64.to_bits());
}

// original-test: tests/TestAABB.cpp::YAxisIsLongest
#[cfg(kani)]
#[kani::proof]
fn aabb_reports_y_as_longest_width_case() {
    let b = Aabb::new(v(-0.5, -0.5, -0.5), v(0.5, 1.0, 0.5));
    assert_eq!(b.longest_width().to_bits(), 1.5_f64.to_bits());
}

// original-test: tests/TestAABB.cpp::ZAxisIsLongest
#[cfg(kani)]
#[kani::proof]
fn aabb_reports_z_as_longest_width_case() {
    let b = Aabb::new(v(-0.5, -0.5, -0.5), v(0.5, 0.5, 1.0));
    assert_eq!(b.longest_width().to_bits(), 1.5_f64.to_bits());
}

// original-test: tests/TestPlane.cpp::CanCreate
#[cfg(kani)]
#[kani::proof]
fn plane_constructor_matches_original() {
    let normal = v(0.0, 1.0, 0.0);
    let plane = Plane::new(v(1.0, 2.0, 3.0), normal);
    assert_eq!(plane.d.to_bits(), 2.0_f64.to_bits());
    assert_eq!(plane.normal, normal);
}

fn original_triangle() -> Triangle {
    Triangle::new(v(0.0, 0.0, 1.0), v(1.0, 0.0, 2.0), v(2.0, 0.0, 1.0))
}

// original-test: tests/TestTriangle.cpp::CanCreate
#[cfg(kani)]
#[kani::proof]
fn triangle_constructor_preserves_vertices() {
    let a = v(1.0, 2.0, 3.0);
    let b = v(4.0, 5.0, 6.0);
    let c = v(7.0, 8.0, 9.0);
    let t = Triangle::new(a, b, c);
    assert_eq!(t.a, a);
    assert_eq!(t.b, b);
    assert_eq!(t.c, c);
}

// original-test: tests/TestTriangle.cpp::GetNormal
#[cfg(kani)]
#[kani::proof]
fn triangle_normal_matches_original_case() {
    assert_eq!(original_triangle().normal(), v(0.0, 1.0, 0.0));
}

// original-test: tests/TestTriangle.cpp::CenterOfTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_contains_center_case() {
    assert!(original_triangle().contains_point(v(1.0, 0.0, 1.0)));
}

// original-test: tests/TestTriangle.cpp::OutsideAB
#[cfg(kani)]
#[kani::proof]
fn triangle_rejects_outside_ab_case() {
    assert!(!original_triangle().contains_point(v(1.5, 0.0, 2.0)));
}

// original-test: tests/TestTriangle.cpp::OutsideAC
#[cfg(kani)]
#[kani::proof]
fn triangle_rejects_outside_ac_case() {
    assert!(!original_triangle().contains_point(v(1.0, 0.0, 0.5)));
}

// original-test: tests/TestTriangle.cpp::OutsideBC
#[cfg(kani)]
#[kani::proof]
fn triangle_rejects_outside_bc_case() {
    assert!(!original_triangle().contains_point(v(2.5, 2.0, 0.0)));
}

// original-test: tests/TestTriangle.cpp::PointAboveTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_above_case() {
    let t = Triangle::new(v(0.0, 1.0, 0.0), v(1.0, 2.0, 0.0), v(2.0, 1.0, 0.0));
    assert_eq!(t.closest_point(v(1.0, 1.5, 1.0)), v(1.0, 1.5, 0.0));
}

// original-test: tests/TestTriangle.cpp::PointBelowTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_below_case() {
    let t = Triangle::new(v(0.0, 1.0, 0.0), v(1.0, 2.0, 0.0), v(2.0, 1.0, 0.0));
    assert_eq!(t.closest_point(v(1.0, 1.5, -1.0)), v(1.0, 1.5, 0.0));
}

// original-test: tests/TestTriangle.cpp::PointLeftOfTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_left_case() {
    let t = Triangle::new(v(10.0, 10.0, 0.0), v(0.0, 10.0, 0.0), Vec3::ZERO);
    assert_eq!(t.closest_point(v(-1.0, 5.0, 0.0)), v(0.0, 5.0, 0.0));
}

// original-test: tests/TestTriangle.cpp::PointBottomOfTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_bottom_case() {
    let t = Triangle::new(Vec3::ZERO, v(10.0, 0.0, 0.0), v(10.0, 10.0, 0.0));
    assert_eq!(t.closest_point(v(5.0, -1.0, 0.0)), v(5.0, 0.0, 0.0));
}

// original-test: tests/TestTriangle.cpp::PointRightOfTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_right_case() {
    let t = Triangle::new(Vec3::ZERO, v(10.0, 0.0, 0.0), v(10.0, 10.0, 0.0));
    assert_eq!(t.closest_point(v(11.0, 5.0, 0.0)), v(10.0, 5.0, 0.0));
}

// original-test: tests/TestTriangle.cpp::PointTopOfTriangle
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_top_case() {
    let t = Triangle::new(Vec3::ZERO, v(0.0, 10.0, 0.0), v(10.0, 10.0, 0.0));
    assert_eq!(t.closest_point(v(5.0, 11.0, 0.0)), v(5.0, 10.0, 0.0));
}

// original-test: tests/TestTriangle.cpp::PointOffTheLongSide
#[cfg(kani)]
#[kani::proof]
fn triangle_closest_point_long_side_case() {
    let t = Triangle::new(Vec3::ZERO, v(10.0, 0.0, 0.0), v(10.0, 10.0, 0.0));
    assert_eq!(t.closest_point(v(0.0, 10.0, 0.0)), v(5.0, 5.0, 0.0));
}
