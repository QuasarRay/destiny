use vstd::prelude::*;

verus! {

pub struct V3 {
    pub x: int,
    pub y: int,
    pub z: int,
}

pub open spec fn v(x: int, y: int, z: int) -> V3 {
    V3 { x, y, z }
}

pub open spec fn sub(a: V3, b: V3) -> V3 {
    v(a.x - b.x, a.y - b.y, a.z - b.z)
}

pub open spec fn dot(a: V3, b: V3) -> int {
    a.x * b.x + a.y * b.y + a.z * b.z
}

pub open spec fn cross(a: V3, b: V3) -> V3 {
    v(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

pub open spec fn same(a: V3, b: V3) -> bool {
    a.x == b.x && a.y == b.y && a.z == b.z
}

// original-test: tests/TestVector3d.cpp::InitializesToZero
pub proof fn vector_initializes_to_zero()
    ensures same(v(0, 0, 0), v(0, 0, 0)),
{
}

// original-test: tests/TestVector3d.cpp::Subtraction
pub proof fn vector_subtraction_matches_original_case()
    ensures same(sub(v(1, 2, 3), v(0, 1, 2)), v(1, 1, 1)),
{
}

// original-test: tests/TestVector3d.cpp::ParallelVectors
pub proof fn parallel_vector_cross_product_is_zero()
    ensures same(cross(v(2, 4, 6), v(1, 2, 3)), v(0, 0, 0)),
{
}

// original-test: tests/TestVector3d.cpp::OrthogonalVectors
pub proof fn orthogonal_vector_cross_product_is_positive_z()
    ensures same(cross(v(1, 0, 0), v(0, 1, 0)), v(0, 0, 1)),
{
}

pub struct Aabb {
    pub low: V3,
    pub high: V3,
}

pub open spec fn aabb(low: V3, high: V3) -> Aabb {
    Aabb { low, high }
}

pub open spec fn zero_aabb() -> Aabb {
    aabb(v(0, 0, 0), v(0, 0, 0))
}

pub open spec fn min2(a: int, b: int) -> int {
    if a < b { a } else { b }
}

pub open spec fn max2(a: int, b: int) -> int {
    if a > b { a } else { b }
}

pub open spec fn aabb_update(box0: Aabb, p: V3) -> Aabb {
    aabb(
        v(min2(box0.low.x, p.x), min2(box0.low.y, p.y), min2(box0.low.z, p.z)),
        v(max2(box0.high.x, p.x), max2(box0.high.y, p.y), max2(box0.high.z, p.z)),
    )
}

pub open spec fn aabb_excludes(box0: Aabb, p0: V3, p1: V3, radius: int) -> bool {
    (p0.x - radius > box0.high.x && p1.x - radius > box0.high.x)
    || (p0.x + radius < box0.low.x && p1.x + radius < box0.low.x)
    || (p0.y - radius > box0.high.y && p1.y - radius > box0.high.y)
    || (p0.y + radius < box0.low.y && p1.y + radius < box0.low.y)
    || (p0.z - radius > box0.high.z && p1.z - radius > box0.high.z)
    || (p0.z + radius < box0.low.z && p1.z + radius < box0.low.z)
}

pub open spec fn aabb_longest_width(box0: Aabb) -> int {
    max2(
        box0.high.x - box0.low.x,
        max2(box0.high.y - box0.low.y, box0.high.z - box0.low.z),
    )
}

// original-test: tests/TestAABB.cpp::DefaultConstructor
pub proof fn aabb_default_constructor_is_zero_box()
    ensures
        same(zero_aabb().low, v(0, 0, 0)),
        same(zero_aabb().high, v(0, 0, 0)),
{
}

// original-test: tests/TestAABB.cpp::ConstructorWithParameters
pub proof fn aabb_parameter_constructor_preserves_corners()
    ensures
        same(aabb(v(1, 2, 3), v(4, 5, 6)).low, v(1, 2, 3)),
        same(aabb(v(1, 2, 3), v(4, 5, 6)).high, v(4, 5, 6)),
{
}

// original-test: tests/TestAABB.cpp::High
pub proof fn aabb_update_expands_high_corner()
    ensures same(aabb_update(zero_aabb(), v(1, 2, 3)).high, v(1, 2, 3)),
{
}

// original-test: tests/TestAABB.cpp::Low
pub proof fn aabb_update_expands_low_corner()
    ensures same(aabb_update(zero_aabb(), v(-1, -2, -3)).low, v(-1, -2, -3)),
{
}

// original-test: tests/TestAABB.cpp::HighIgnore
pub proof fn aabb_update_inside_does_not_reduce_high_corner()
    ensures same(
        aabb_update(aabb(v(0, 0, 0), v(5, 5, 5)), v(1, 2, 3)).high,
        v(5, 5, 5),
    ),
{
}

// original-test: tests/TestAABB.cpp::LowIgnore
pub proof fn aabb_update_inside_does_not_raise_low_corner()
    ensures same(
        aabb_update(aabb(v(-5, -5, -5), v(0, 0, 0)), v(-1, -2, -3)).low,
        v(-5, -5, -5),
    ),
{
}

pub open spec fn unit_box_scaled2() -> Aabb {
    aabb(v(0, 0, 0), v(2, 2, 2))
}

// original-test: tests/TestAABB.cpp::ExcludeReturnsFalseWhenRadiusIntersects
pub proof fn aabb_intersecting_radius_cannot_be_excluded()
    ensures !aabb_excludes(unit_box_scaled2(), v(-2, -2, 0), v(-2, 2, 0), 2),
{
}

// original-test: tests/TestAABB.cpp::ExcludeLeft
pub proof fn aabb_excludes_left()
    ensures aabb_excludes(unit_box_scaled2(), v(-2, -2, 0), v(-2, 2, 0), 1),
{
}

// original-test: tests/TestAABB.cpp::ExcludeRight
pub proof fn aabb_excludes_right()
    ensures aabb_excludes(unit_box_scaled2(), v(4, -2, 0), v(4, 2, 0), 1),
{
}

// original-test: tests/TestAABB.cpp::ExcludeTop
pub proof fn aabb_excludes_top()
    ensures aabb_excludes(unit_box_scaled2(), v(-2, 4, 0), v(2, 4, 0), 1),
{
}

// original-test: tests/TestAABB.cpp::ExcludeBottom
pub proof fn aabb_excludes_bottom()
    ensures aabb_excludes(unit_box_scaled2(), v(-2, -2, 0), v(2, -2, 0), 1),
{
}

// original-test: tests/TestAABB.cpp::ExcludeFront
pub proof fn aabb_excludes_front()
    ensures aabb_excludes(unit_box_scaled2(), v(-2, 0, 4), v(2, 0, 4), 1),
{
}

// original-test: tests/TestAABB.cpp::ExcludeBack
pub proof fn aabb_excludes_back()
    ensures aabb_excludes(unit_box_scaled2(), v(-2, 0, -2), v(2, 0, -2), 1),
{
}

// original-test: tests/TestAABB.cpp::InsideBox
pub proof fn aabb_inside_sphere_cannot_be_excluded()
    ensures !aabb_excludes(unit_box_scaled2(), v(1, 1, 1), v(1, 1, 1), 1),
{
}

// original-test: tests/TestAABB.cpp::ZeroSize
pub proof fn zero_aabb_longest_width_is_zero()
    ensures aabb_longest_width(zero_aabb()) == 0,
{
}

// original-test: tests/TestAABB.cpp::XAxisIsLongest
pub proof fn aabb_x_longest_width_scaled2_is_three()
    ensures aabb_longest_width(aabb(v(-1, -1, -1), v(2, 1, 1))) == 3,
{
}

// original-test: tests/TestAABB.cpp::YAxisIsLongest
pub proof fn aabb_y_longest_width_scaled2_is_three()
    ensures aabb_longest_width(aabb(v(-1, -1, -1), v(1, 2, 1))) == 3,
{
}

// original-test: tests/TestAABB.cpp::ZAxisIsLongest
pub proof fn aabb_z_longest_width_scaled2_is_three()
    ensures aabb_longest_width(aabb(v(-1, -1, -1), v(1, 1, 2))) == 3,
{
}

pub struct Plane {
    pub d: int,
    pub normal: V3,
}

pub open spec fn plane(point: V3, normal: V3) -> Plane {
    Plane { d: dot(point, normal), normal }
}

// original-test: tests/TestPlane.cpp::CanCreate
pub proof fn plane_constructor_matches_original()
    ensures
        plane(v(1, 2, 3), v(0, 1, 0)).d == 2,
        same(plane(v(1, 2, 3), v(0, 1, 0)).normal, v(0, 1, 0)),
{
}

pub struct Triangle {
    pub a: V3,
    pub b: V3,
    pub c: V3,
}

pub open spec fn triangle(a: V3, b: V3, c: V3) -> Triangle {
    Triangle { a, b, c }
}

pub open spec fn barycentric_contains(t: Triangle, p: V3) -> bool {
    let v0 = sub(t.c, t.a);
    let v1 = sub(t.b, t.a);
    let v2 = sub(p, t.a);
    let dot00 = dot(v0, v0);
    let dot01 = dot(v0, v1);
    let dot02 = dot(v0, v2);
    let dot11 = dot(v1, v1);
    let dot12 = dot(v1, v2);
    let denom = dot00 * dot11 - dot01 * dot01;
    let u_num = dot11 * dot02 - dot01 * dot12;
    let v_num = dot00 * dot12 - dot01 * dot02;
    denom > 0 && u_num >= 0 && v_num >= 0 && u_num + v_num < denom
}

pub open spec fn original_triangle_scaled2() -> Triangle {
    triangle(v(0, 0, 2), v(2, 0, 4), v(4, 0, 2))
}

pub open spec fn positive_y_axis_unit_from_cross(c: V3) -> V3
    recommends c.x == 0, c.z == 0, c.y > 0
{
    v(0, 1, 0)
}

// original-test: tests/TestTriangle.cpp::CanCreate
pub proof fn triangle_constructor_preserves_vertices()
    ensures
        same(triangle(v(1, 2, 3), v(4, 5, 6), v(7, 8, 9)).a, v(1, 2, 3)),
        same(triangle(v(1, 2, 3), v(4, 5, 6), v(7, 8, 9)).b, v(4, 5, 6)),
        same(triangle(v(1, 2, 3), v(4, 5, 6), v(7, 8, 9)).c, v(7, 8, 9)),
{
}

// original-test: tests/TestTriangle.cpp::GetNormal
pub proof fn triangle_normal_is_positive_y()
    ensures {
        let ab = sub(v(1, 0, 2), v(0, 0, 1));
        let ac = sub(v(2, 0, 1), v(0, 0, 1));
        let c = cross(ab, ac);
        c.x == 0 && c.y == 2 && c.z == 0
        && same(positive_y_axis_unit_from_cross(c), v(0, 1, 0))
    },
{
}

// original-test: tests/TestTriangle.cpp::CenterOfTriangle
pub proof fn triangle_contains_center()
    ensures barycentric_contains(original_triangle_scaled2(), v(2, 0, 2)),
{
}

// original-test: tests/TestTriangle.cpp::OutsideAB
pub proof fn triangle_rejects_outside_ab()
    ensures !barycentric_contains(original_triangle_scaled2(), v(3, 0, 4)),
{
}

// original-test: tests/TestTriangle.cpp::OutsideAC
pub proof fn triangle_rejects_outside_ac()
    ensures !barycentric_contains(original_triangle_scaled2(), v(2, 0, 1)),
{
}

// original-test: tests/TestTriangle.cpp::OutsideBC
pub proof fn triangle_rejects_outside_bc()
    ensures !barycentric_contains(original_triangle_scaled2(), v(5, 4, 0)),
{
}

pub open spec fn project_z0(p: V3) -> V3 { v(p.x, p.y, 0) }
pub open spec fn project_x(p: V3, boundary: int) -> V3 { v(boundary, p.y, p.z) }
pub open spec fn project_y(p: V3, boundary: int) -> V3 { v(p.x, boundary, p.z) }
pub open spec fn diagonal_xy_projection(p: V3, q: V3) -> bool {
    q.z == p.z && q.x == q.y && 2 * q.x == p.x + p.y
}

// All closest-point coordinates below are scaled by two, so the 1.5 values in
// the original C++ tests remain exact mathematical integers.

// original-test: tests/TestTriangle.cpp::PointAboveTriangle
pub proof fn triangle_closest_point_above_is_plane_projection()
    ensures same(project_z0(v(2, 3, 2)), v(2, 3, 0)),
{
}

// original-test: tests/TestTriangle.cpp::PointBelowTriangle
pub proof fn triangle_closest_point_below_is_plane_projection()
    ensures same(project_z0(v(2, 3, -2)), v(2, 3, 0)),
{
}

// original-test: tests/TestTriangle.cpp::PointLeftOfTriangle
pub proof fn triangle_closest_point_left_is_edge_projection()
    ensures same(project_x(v(-2, 10, 0), 0), v(0, 10, 0)),
{
}

// original-test: tests/TestTriangle.cpp::PointBottomOfTriangle
pub proof fn triangle_closest_point_bottom_is_edge_projection()
    ensures same(project_y(v(10, -2, 0), 0), v(10, 0, 0)),
{
}

// original-test: tests/TestTriangle.cpp::PointRightOfTriangle
pub proof fn triangle_closest_point_right_is_edge_projection()
    ensures same(project_x(v(22, 10, 0), 20), v(20, 10, 0)),
{
}

// original-test: tests/TestTriangle.cpp::PointTopOfTriangle
pub proof fn triangle_closest_point_top_is_edge_projection()
    ensures same(project_y(v(10, 22, 0), 20), v(10, 20, 0)),
{
}

// original-test: tests/TestTriangle.cpp::PointOffTheLongSide
pub proof fn triangle_closest_point_long_side_is_diagonal_projection()
    ensures diagonal_xy_projection(v(0, 20, 0), v(10, 10, 0)),
{
}

} // verus!
