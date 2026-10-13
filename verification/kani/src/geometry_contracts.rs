use destiny_original_spec::{Aabb, Plane, Triangle, Vec3};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

// original-test: tests/TestVector3d.cpp::InitializesToZero
// original-case: tests/TestVector3d.cpp::Vector3d::InitializesToZero
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn vector_initializes_to_zero() {
    assert!(Vec3::default() == v(0.0, 0.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestVector3d.cpp::Subtraction
// original-case: tests/TestVector3d.cpp::Vector3d::Subtraction
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn vector_subtraction_matches_original_case() {
    assert!(v(1.0, 2.0, 3.0).sub(v(0.0, 1.0, 2.0)) == v(1.0, 1.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestVector3d.cpp::ParallelVectors
// original-case: tests/TestVector3d.cpp::Vector3dCrossProduct::ParallelVectors
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn parallel_vector_cross_product_is_zero() {
    assert!(v(2.0, 4.0, 6.0).cross(v(1.0, 2.0, 3.0)) == Vec3::ZERO);
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestVector3d.cpp::OrthogonalVectors
// original-case: tests/TestVector3d.cpp::Vector3dCrossProduct::OrthogonalVectors
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn orthogonal_vector_cross_product_is_positive_z() {
    assert!(v(1.0, 0.0, 0.0).cross(v(0.0, 1.0, 0.0)) == v(0.0, 0.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::DefaultConstructor
// original-case: tests/TestAABB.cpp::Constructor::DefaultConstructor
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_default_constructor_is_zero_box() {
    let b = Aabb::default();
    assert!(b.low == Vec3::ZERO);
    assert!(b.high == Vec3::ZERO);
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ConstructorWithParameters
// original-case: tests/TestAABB.cpp::Constructor::ConstructorWithParameters
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_parameter_constructor_preserves_corners() {
    let b = Aabb::new(v(1.0, 2.0, 3.0), v(4.0, 5.0, 6.0));
    assert!(b.low == v(1.0, 2.0, 3.0));
    assert!(b.high == v(4.0, 5.0, 6.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::High
// original-case: tests/TestAABB.cpp::Update::High
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_update_expands_high_corner() {
    let mut b = Aabb::default();
    b.update(v(1.0, 2.0, 3.0));
    assert!(b.high == v(1.0, 2.0, 3.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::Low
// original-case: tests/TestAABB.cpp::Update::Low
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_update_expands_low_corner() {
    let mut b = Aabb::default();
    b.update(v(-1.0, -2.0, -3.0));
    assert!(b.low == v(-1.0, -2.0, -3.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::HighIgnore
// original-case: tests/TestAABB.cpp::Update::HighIgnore
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_update_inside_does_not_reduce_high_corner() {
    let mut b = Aabb::new(Vec3::ZERO, v(5.0, 5.0, 5.0));
    b.update(v(1.0, 2.0, 3.0));
    assert!(b.high == v(5.0, 5.0, 5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::LowIgnore
// original-case: tests/TestAABB.cpp::Update::LowIgnore
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_update_inside_does_not_raise_low_corner() {
    let mut b = Aabb::new(v(-5.0, -5.0, -5.0), Vec3::ZERO);
    b.update(v(-1.0, -2.0, -3.0));
    assert!(b.low == v(-5.0, -5.0, -5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

fn unit_aabb() -> Aabb {
    Aabb::new(Vec3::ZERO, v(1.0, 1.0, 1.0))
}

// original-test: tests/TestAABB.cpp::ExcludeReturnsFalseWhenRadiusIntersects
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeReturnsFalseWhenRadiusIntersects
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_does_not_exclude_when_swept_radius_intersects() {
    assert!(!unit_aabb().can_exclude_collision(v(-1.0, -1.0, 0.0), v(-1.0, 1.0, 0.0), 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ExcludeLeft
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeLeft
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_excludes_sweep_left() {
    assert!(unit_aabb().can_exclude_collision(v(-1.0, -1.0, 0.0), v(-1.0, 1.0, 0.0), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ExcludeRight
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeRight
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_excludes_sweep_right() {
    assert!(unit_aabb().can_exclude_collision(v(2.0, -1.0, 0.0), v(2.0, 1.0, 0.0), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ExcludeTop
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeTop
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_excludes_sweep_top() {
    assert!(unit_aabb().can_exclude_collision(v(-1.0, 2.0, 0.0), v(1.0, 2.0, 0.0), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ExcludeBottom
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeBottom
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_excludes_sweep_bottom() {
    assert!(unit_aabb().can_exclude_collision(v(-1.0, -1.0, 0.0), v(1.0, -1.0, 0.0), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ExcludeFront
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeFront
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_excludes_sweep_front() {
    assert!(unit_aabb().can_exclude_collision(v(-1.0, 0.0, 2.0), v(1.0, 0.0, 2.0), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ExcludeBack
// original-case: tests/TestAABB.cpp::CanExcludeCollision::ExcludeBack
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_excludes_sweep_back() {
    assert!(unit_aabb().can_exclude_collision(v(-1.0, 0.0, -1.0), v(1.0, 0.0, -1.0), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::InsideBox
// original-case: tests/TestAABB.cpp::CanExcludeCollision::InsideBox
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_does_not_exclude_sphere_inside_box() {
    assert!(!unit_aabb().can_exclude_collision(v(0.5, 0.5, 0.5), v(0.5, 0.5, 0.5), 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ZeroSize
// original-case: tests/TestAABB.cpp::GetLongestWidth::ZeroSize
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn zero_aabb_longest_width_is_zero() {
    assert!(Aabb::default().longest_width().to_bits() == 0.0_f64.to_bits());
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::XAxisIsLongest
// original-case: tests/TestAABB.cpp::GetLongestWidth::XAxisIsLongest
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_reports_x_as_longest_width_case() {
    let b = Aabb::new(v(-0.5, -0.5, -0.5), v(1.0, 0.5, 0.5));
    assert!(b.longest_width().to_bits() == 1.5_f64.to_bits());
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::YAxisIsLongest
// original-case: tests/TestAABB.cpp::GetLongestWidth::YAxisIsLongest
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_reports_y_as_longest_width_case() {
    let b = Aabb::new(v(-0.5, -0.5, -0.5), v(0.5, 1.0, 0.5));
    assert!(b.longest_width().to_bits() == 1.5_f64.to_bits());
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestAABB.cpp::ZAxisIsLongest
// original-case: tests/TestAABB.cpp::GetLongestWidth::ZAxisIsLongest
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn aabb_reports_z_as_longest_width_case() {
    let b = Aabb::new(v(-0.5, -0.5, -0.5), v(0.5, 0.5, 1.0));
    assert!(b.longest_width().to_bits() == 1.5_f64.to_bits());
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestPlane.cpp::CanCreate
// original-case: tests/TestPlane.cpp::Plane::CanCreate
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn plane_constructor_matches_original() {
    let normal = v(0.0, 1.0, 0.0);
    let plane = Plane::new(v(1.0, 2.0, 3.0), normal);
    assert!(plane.d.to_bits() == 2.0_f64.to_bits());
    assert!(plane.normal == normal);
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

fn original_triangle() -> Triangle {
    Triangle::new(v(0.0, 0.0, 1.0), v(1.0, 0.0, 2.0), v(2.0, 0.0, 1.0))
}

// original-test: tests/TestTriangle.cpp::CanCreate
// original-case: tests/TestTriangle.cpp::Triangle::CanCreate
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_constructor_preserves_vertices() {
    let a = v(1.0, 2.0, 3.0);
    let b = v(4.0, 5.0, 6.0);
    let c = v(7.0, 8.0, 9.0);
    let t = Triangle::new(a, b, c);
    assert!(t.a == a);
    assert!(t.b == b);
    assert!(t.c == c);
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::GetNormal
// original-case: tests/TestTriangle.cpp::Triangle::GetNormal
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_normal_matches_original_case() {
    assert!(original_triangle().normal() == v(0.0, 1.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::CenterOfTriangle
// original-case: tests/TestTriangle.cpp::PointInTriangle::CenterOfTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_contains_center_case() {
    assert!(original_triangle().contains_point(v(1.0, 0.0, 1.0)));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::OutsideAB
// original-case: tests/TestTriangle.cpp::PointInTriangle::OutsideAB
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_rejects_outside_ab_case() {
    assert!(!original_triangle().contains_point(v(1.5, 0.0, 2.0)));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::OutsideAC
// original-case: tests/TestTriangle.cpp::PointInTriangle::OutsideAC
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_rejects_outside_ac_case() {
    assert!(!original_triangle().contains_point(v(1.0, 0.0, 0.5)));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::OutsideBC
// original-case: tests/TestTriangle.cpp::PointInTriangle::OutsideBC
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_rejects_outside_bc_case() {
    assert!(!original_triangle().contains_point(v(2.5, 2.0, 0.0)));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointAboveTriangle
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointAboveTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_above_case() {
    let t = Triangle::new(v(0.0, 1.0, 0.0), v(1.0, 2.0, 0.0), v(2.0, 1.0, 0.0));
    assert!(t.closest_point(v(1.0, 1.5, 1.0)) == v(1.0, 1.5, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointBelowTriangle
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointBelowTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_below_case() {
    let t = Triangle::new(v(0.0, 1.0, 0.0), v(1.0, 2.0, 0.0), v(2.0, 1.0, 0.0));
    assert!(t.closest_point(v(1.0, 1.5, -1.0)) == v(1.0, 1.5, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointLeftOfTriangle
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointLeftOfTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_left_case() {
    let t = Triangle::new(v(10.0, 10.0, 0.0), v(0.0, 10.0, 0.0), Vec3::ZERO);
    assert!(t.closest_point(v(-1.0, 5.0, 0.0)) == v(0.0, 5.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointBottomOfTriangle
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointBottomOfTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_bottom_case() {
    let t = Triangle::new(Vec3::ZERO, v(10.0, 0.0, 0.0), v(10.0, 10.0, 0.0));
    assert!(t.closest_point(v(5.0, -1.0, 0.0)) == v(5.0, 0.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointRightOfTriangle
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointRightOfTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_right_case() {
    let t = Triangle::new(Vec3::ZERO, v(10.0, 0.0, 0.0), v(10.0, 10.0, 0.0));
    assert!(t.closest_point(v(11.0, 5.0, 0.0)) == v(10.0, 5.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointTopOfTriangle
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointTopOfTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_top_case() {
    let t = Triangle::new(Vec3::ZERO, v(0.0, 10.0, 0.0), v(10.0, 10.0, 0.0));
    assert!(t.closest_point(v(5.0, 11.0, 0.0)) == v(5.0, 10.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}

// original-test: tests/TestTriangle.cpp::PointOffTheLongSide
// original-case: tests/TestTriangle.cpp::GetClosestPoint::PointOffTheLongSide
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn triangle_closest_point_long_side_case() {
    let t = Triangle::new(Vec3::ZERO, v(10.0, 0.0, 0.0), v(10.0, 10.0, 0.0));
    assert!(t.closest_point(v(0.0, 10.0, 0.0)) == v(5.0, 5.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original geometry scenario completed");
}
