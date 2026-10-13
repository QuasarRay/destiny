use destiny_original_spec::{
    Plane, Triangle, Vec3, first_plane_contact, intersect_segment_plane, intersect_segment_sphere,
    intersect_sphere_triangle,
};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}

// original-case: tests/TestCollision.cpp::IntersectSegmentPlane::IntersectionExists
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn segment_plane_intersection() {
    let plane = Plane::new(v(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let exists = intersect_segment_plane(
        v(0.0, 1.0, 0.0),
        v(0.0, -1.0, 0.0),
        plane,
        &mut point,
        &mut time,
    );
    assert!(exists);
    assert!(point == v(0.0, 0.0, 0.0));
    assert!(time == 0.5);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSegmentPlane::NoIntersection
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn segment_plane_miss() {
    let plane = Plane::new(v(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let exists = intersect_segment_plane(
        v(0.0, 3.0, 0.0),
        v(0.0, 2.0, 0.0),
        plane,
        &mut point,
        &mut time,
    );
    assert!(!exists);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSegmentPlane::ParallelLineSegment
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn segment_plane_parallel() {
    let plane = Plane::new(v(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let exists = intersect_segment_plane(
        v(0.0, 1.0, 0.0),
        v(0.0, -1.0, 0.0),
        plane,
        &mut point,
        &mut time,
    );
    assert!(!exists);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSegmentSphere::HeadOn
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn segment_sphere_head_on() {
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let exists = intersect_segment_sphere(
        v(10.0, 0.0, 0.0),
        v(0.0, 0.0, 0.0),
        v(0.0, 0.0, 0.0),
        1.0,
        &mut point,
        &mut time,
    );
    assert!(exists);
    assert!(point == v(1.0, 0.0, 0.0));
    assert!(time == 0.9);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSegmentSphere::FromInside
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn segment_sphere_from_inside() {
    let a = v(0.0, 0.0, 0.0);
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let exists = intersect_segment_sphere(
        a,
        v(10.0, 0.0, 0.0),
        v(0.0, 0.0, 0.0),
        1.0,
        &mut point,
        &mut time,
    );
    assert!(exists);
    assert!(point == a);
    assert!(time == 0.0);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSegmentSphere::CompleteMiss
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn segment_sphere_complete_miss() {
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let exists = intersect_segment_sphere(
        v(0.0, 0.0, 7.0),
        v(0.0, 0.0, 8.0),
        v(0.0, 0.0, 0.0),
        1.0,
        &mut point,
        &mut time,
    );
    assert!(!exists);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSphereTriangle::CenterOfSphereIntersectsTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn sphere_triangle_center() {
    let triangle = Triangle::new(v(0.0, 0.0, 0.0), v(2.0, 0.0, 0.0), v(1.0, 2.0, 0.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = intersect_sphere_triangle(
        v(1.0, 1.0, 2.0),
        1.0,
        v(0.0, 0.0, -2.0),
        triangle,
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(point == v(1.0, 1.0, 0.0));
    assert!(time == 0.5);
    assert!(normal == v(0.0, 0.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSphereTriangle::SphereGrazesTriangleEdge
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn sphere_triangle_grazing_edge() {
    let triangle = Triangle::new(v(0.0, 0.0, 0.0), v(2.0, 0.0, 0.0), v(1.0, 2.0, 0.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = intersect_sphere_triangle(
        v(0.0, -1.0, 2.0),
        1.0,
        v(0.0, 0.0, -2.0),
        triangle,
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(point == v(0.0, 0.0, 0.0));
    assert!(time == 1.0);
    assert!(normal == v(0.0, -1.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSphereTriangle::CenterOfSphereIntersectsTriangleFromTheWrongSide
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn sphere_triangle_reversed_winding() {
    let a = v(0.0, 0.0, 0.0);
    let b = v(2.0, 0.0, 0.0);
    let c = v(1.0, 2.0, 0.0);
    let triangle = Triangle::new(c, b, a);
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = intersect_sphere_triangle(
        v(1.0, 1.0, 2.0),
        1.0,
        v(0.0, 0.0, -2.0),
        triangle,
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(!exists);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSphereTriangle::SphereDoesNotIntersectPlane
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn sphere_triangle_plane_miss() {
    let triangle = Triangle::new(v(0.0, 0.0, 0.0), v(2.0, 0.0, 0.0), v(1.0, 2.0, 0.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = intersect_sphere_triangle(
        v(1.0, 1.0, 2.0),
        1.0,
        v(0.0, 100.0, 0.0),
        triangle,
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(!exists);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::IntersectSphereTriangle::SphereDoesNotIntersectTriangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn sphere_triangle_triangle_miss() {
    let triangle = Triangle::new(v(0.0, 0.0, 0.0), v(2.0, 0.0, 0.0), v(1.0, 2.0, 0.0));
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = intersect_sphere_triangle(
        v(100.0, 1.0, 1.0),
        1.0,
        v(0.0, 0.0, -2.0),
        triangle,
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(!exists);
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}

// original-case: tests/TestCollision.cpp::GetPointOnBallThatFirstIntersectsPlane::ArbitraryPoint
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn first_contact_is_radius_below_center() {
    let actual = first_plane_contact(v(0.0, 3.0, 0.0), 1.0, v(0.0, 1.0, 0.0));
    assert!(actual == v(0.0, 2.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original collision scenario completed");
}
