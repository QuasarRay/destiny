//! Translation of original Destiny src/Collision.cpp at the catalog revision.
//! Original algorithm: Copyright © 2015 CCP ehf., MIT (repository LICENSE).
//! Output references preserve the original write order, including writes before
//! a false return. No Avian contact or native-wrapper correspondence is claimed.

use crate::{Plane, Triangle, Vec3};

#[must_use]
pub fn first_plane_contact(ball_position: Vec3, radius: f64, normal: Vec3) -> Vec3 {
    ball_position.sub(normal.scale(radius))
}

pub fn intersect_segment_plane(
    p0: Vec3,
    p1: Vec3,
    plane: Plane,
    point: &mut Vec3,
    time: &mut f64,
) -> bool {
    let direction = p1.sub(p0);
    let normal_dot_direction = plane.normal.dot(direction);
    if normal_dot_direction == 0.0 {
        return false;
    }
    *time = (plane.d - plane.normal.dot(p0)) / normal_dot_direction;
    if *time >= 0.0 && *time <= 1.0 {
        *point = p0.add(direction.scale(*time));
        return true;
    }
    false
}

pub fn intersect_segment_sphere(
    p0: Vec3,
    p1: Vec3,
    sphere: Vec3,
    radius: f64,
    point: &mut Vec3,
    time: &mut f64,
) -> bool {
    let direction = p1.sub(p0);
    let normalized_direction = direction.normalize();
    let m = p0.sub(sphere);
    let b = m.dot(normalized_direction);
    let c = m.dot(m) - radius * radius;
    if c > 0.0 && b > 0.0 {
        return false;
    }
    let discriminant = b * b - c;
    if discriminant < 0.0 {
        return false;
    }
    *time = -b - discriminant.sqrt();
    if *time < 0.0 {
        *time = 0.0;
    } else {
        *time /= direction.length();
    }
    if *time > 1.0 {
        return false;
    }
    *point = p0.add(direction.scale(*time));
    true
}

pub fn intersect_sphere_triangle(
    sphere: Vec3,
    radius: f64,
    velocity: Vec3,
    triangle: Triangle,
    point: &mut Vec3,
    time: &mut f64,
    normal: &mut Vec3,
) -> bool {
    let triangle_normal = triangle.normal();
    let p0 = first_plane_contact(sphere, radius, triangle_normal);
    let p1 = p0.add(velocity);
    let plane = Plane::new(triangle.a, triangle_normal);
    if !intersect_segment_plane(p0, p1, plane, point, time) {
        return false;
    }
    if triangle.contains_point(*point) {
        *normal = triangle_normal;
        return true;
    }
    *point = triangle.closest_point(*point);
    let p0 = *point;
    let p1 = point.sub(velocity);
    *normal = Vec3 {
        x: -velocity.x,
        y: -velocity.y,
        z: -velocity.z,
    };
    let mut impact_point = Vec3::ZERO;
    if !intersect_segment_sphere(p0, p1, sphere, radius, &mut impact_point, time) {
        return false;
    }
    *normal = sphere.sub(impact_point).normalize();
    true
}
