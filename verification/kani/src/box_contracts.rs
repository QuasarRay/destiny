use destiny_original_spec::{BoxShape, Rectangle, Vec3};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3 { x, y, z }
}
fn unit_cube() -> BoxShape {
    BoxShape::new(
        Vec3::ZERO,
        v(1.0, 0.0, 0.0),
        v(0.0, 1.0, 0.0),
        v(0.0, 0.0, 1.0),
    )
}

fn cube_ten() -> BoxShape {
    BoxShape::new(
        Vec3::ZERO,
        v(10.0, 0.0, 0.0),
        v(0.0, 10.0, 0.0),
        v(0.0, 0.0, 10.0),
    )
}

fn rectangle() -> Rectangle {
    Rectangle::new(
        Vec3::ZERO,
        v(10.0, 0.0, 0.0),
        v(10.0, 10.0, 0.0),
        v(0.0, 10.0, 0.0),
    )
}

// original-case: tests/TestBoxShape.cpp::UnitCubeTest::GetClosestCornerToPointTopRightBack
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn unit_cube_closest_corner_back() {
    assert!(unit_cube().closest_corner(v(2.0, 2.0, 0.0)) == 4);
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::UnitCubeTest::GetClosestCornerToPointTopRightFront
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn unit_cube_closest_corner_front() {
    assert!(unit_cube().closest_corner(v(2.0, 2.0, 2.0)) == 7);
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::UnitCubeTest::GetCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn unit_cube_center() {
    assert!(unit_cube().center() == v(0.5, 0.5, 0.5));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::UnitCubeTest::GetBoundingRadius
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn unit_cube_bounding_radius() {
    assert!(unit_cube().bounding_radius() == 3.0_f64.sqrt() / 2.0);
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::RectangleSphereCollisionTest::CanCreate
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn rectangle_constructor() {
    let r = rectangle();
    assert!(r.a == v(0.0, 0.0, 0.0));
    assert!(r.b == v(10.0, 0.0, 0.0));
    assert!(r.c == v(10.0, 10.0, 0.0));
    assert!(r.d == v(0.0, 10.0, 0.0));
    assert!(r.normal == v(0.0, 0.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::RectangleSphereCollisionTest::CenterOfRectangle
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn rectangle_collision_center() {
    let side = rectangle();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = side.collide_with_sphere(
        v(5.0, 5.0, 6.0),
        1.0,
        v(0.0, 0.0, -10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 5.0, 0.0));
    assert!(normal == v(0.0, 0.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::RectangleSphereCollisionTest::GrazeLeftSide
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn rectangle_collision_left() {
    let side = rectangle();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = side.collide_with_sphere(
        v(-1.0, 5.0, 5.0),
        1.0,
        v(0.0, 0.0, -10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(0.0, 5.0, 0.0));
    assert!(normal == v(-1.0, 0.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::RectangleSphereCollisionTest::GrazeRightSide
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9), kani::solver(kissat))]
#[cfg_attr(not(kani), test)]
fn rectangle_collision_right() {
    let side = rectangle();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = side.collide_with_sphere(
        v(11.0, 5.0, 5.0),
        1.0,
        v(0.0, 0.0, -10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(10.0, 5.0, 0.0));
    assert!(normal == v(1.0, 0.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::RectangleSphereCollisionTest::GrazeTopSide
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn rectangle_collision_top() {
    let side = rectangle();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = side.collide_with_sphere(
        v(5.0, 11.0, 5.0),
        1.0,
        v(0.0, 0.0, -10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 10.0, 0.0));
    assert!(normal == v(0.0, 1.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::RectangleSphereCollisionTest::GrazeBottomSide
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn rectangle_collision_bottom() {
    let side = rectangle();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = side.collide_with_sphere(
        v(5.0, -1.0, 5.0),
        1.0,
        v(0.0, 0.0, -10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 0.0, 0.0));
    assert!(normal == v(0.0, -1.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::CubeCollisionTest::FrontCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_collision_front() {
    let b = cube_ten();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = b.collide_with_sphere(
        v(5.0, 5.0, 16.0),
        1.0,
        v(0.0, 0.0, -10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 5.0, 10.0));
    assert!(normal == v(0.0, 0.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::CubeCollisionTest::BackCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_collision_back() {
    let b = cube_ten();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = b.collide_with_sphere(
        v(5.0, 5.0, -6.0),
        1.0,
        v(0.0, 0.0, 10.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 5.0, 0.0));
    assert!(normal == v(0.0, 0.0, -1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::CubeCollisionTest::LeftCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_collision_left() {
    let b = cube_ten();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = b.collide_with_sphere(
        v(-6.0, 5.0, 5.0),
        1.0,
        v(10.0, 0.0, 0.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(0.0, 5.0, 5.0));
    assert!(normal == v(-1.0, 0.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::CubeCollisionTest::RightCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_collision_right() {
    let b = cube_ten();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = b.collide_with_sphere(
        v(16.0, 5.0, 5.0),
        1.0,
        v(-10.0, 0.0, 0.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(10.0, 5.0, 5.0));
    assert!(normal == v(1.0, 0.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::CubeCollisionTest::TopCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_collision_top() {
    let b = cube_ten();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = b.collide_with_sphere(
        v(5.0, 16.0, 5.0),
        1.0,
        v(0.0, -10.0, 0.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 10.0, 5.0));
    assert!(normal == v(0.0, 1.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::CubeCollisionTest::BottomCenter
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_collision_bottom() {
    let b = cube_ten();
    let mut point = Vec3::ZERO;
    let mut time = f64::NAN;
    let mut normal = Vec3::ZERO;
    let exists = b.collide_with_sphere(
        v(5.0, -6.0, 5.0),
        1.0,
        v(0.0, 10.0, 0.0),
        &mut point,
        &mut time,
        &mut normal,
    );
    assert!(exists);
    assert!(time == 0.5);
    assert!(point == v(5.0, 0.0, 5.0));
    assert!(normal == v(0.0, -1.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::CenterOfBox
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_center() {
    let actual = cube_ten().closest_point(v(5.0, 5.0, 5.0));
    assert!(actual == v(5.0, 5.0, 5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::TopFrontRightCorner
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_corner() {
    let actual = cube_ten().closest_point(v(11.0, 11.0, 11.0));
    assert!(actual == v(10.0, 10.0, 10.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::Left
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_left() {
    let actual = cube_ten().closest_point(v(-1.0, 5.0, 5.0));
    assert!(actual == v(0.0, 5.0, 5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::Right
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_right() {
    let actual = cube_ten().closest_point(v(11.0, 5.0, 5.0));
    assert!(actual == v(10.0, 5.0, 5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::Top
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_top() {
    let actual = cube_ten().closest_point(v(5.0, 11.0, 5.0));
    assert!(actual == v(5.0, 10.0, 5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::Bottom
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_bottom() {
    let actual = cube_ten().closest_point(v(5.0, -1.0, 5.0));
    assert!(actual == v(5.0, 0.0, 5.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::Back
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_back() {
    let actual = cube_ten().closest_point(v(5.0, 5.0, -1.0));
    assert!(actual == v(5.0, 5.0, 0.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::GetClosestPointOnBoxTest::Front
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn cube_closest_point_front() {
    let actual = cube_ten().closest_point(v(5.0, 5.0, 11.0));
    assert!(actual == v(5.0, 5.0, 10.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}

// original-case: tests/TestBoxShape.cpp::UnitCubeTest::CanCreate
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(9))]
#[cfg_attr(not(kani), test)]
fn unit_cube_constructor() {
    let b = unit_cube();
    assert!(b.local_x == v(1.0, 0.0, 0.0));
    assert!(b.local_y == v(0.0, 1.0, 0.0));
    assert!(b.local_z == v(0.0, 0.0, 1.0));
    assert!(b.corners[0] == v(0.0, 0.0, 0.0));
    assert!(b.corners[1] == v(1.0, 0.0, 0.0));
    assert!(b.corners[2] == v(0.0, 1.0, 0.0));
    assert!(b.corners[3] == v(0.0, 0.0, 1.0));
    assert!(b.corners[4] == v(1.0, 1.0, 0.0));
    assert!(b.corners[5] == v(0.0, 1.0, 1.0));
    assert!(b.corners[6] == v(1.0, 0.0, 1.0));
    assert!(b.corners[7] == v(1.0, 1.0, 1.0));
    #[cfg(kani)]
    kani::cover!(true, "exact original box scenario completed");
}
