//! Original BoxShape.cpp translation; Copyright © 2015 CCP ehf., MIT.
//! Corner/side ordering, strict comparisons, first-match collision selection
//! and the original slab handling of parallel rays are retained.

use crate::{Triangle, Vec3, first_plane_contact, intersect_sphere_triangle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub d: Vec3,
    pub normal: Vec3,
}

impl Rectangle {
    #[must_use]
    pub fn new(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Self {
        Self {
            a,
            b,
            c,
            d,
            normal: b.sub(a).cross(c.sub(a)).normalize(),
        }
    }

    pub fn collide_with_sphere(
        &self,
        sphere: Vec3,
        radius: f64,
        velocity: Vec3,
        point: &mut Vec3,
        time: &mut f64,
        normal: &mut Vec3,
    ) -> bool {
        let pa = self.a.sub(sphere);
        let pc = self.c.sub(sphere);
        let v = velocity.dot(pc.cross(pa));
        let triangle = if v <= 0.0 {
            Triangle::new(self.a, self.b, self.c)
        } else {
            Triangle::new(self.a, self.c, self.d)
        };
        intersect_sphere_triangle(sphere, radius, velocity, triangle, point, time, normal)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxShape {
    pub corners: [Vec3; 8],
    pub sides: [Rectangle; 6],
    pub local_x: Vec3,
    pub local_y: Vec3,
    pub local_z: Vec3,
}

impl BoxShape {
    #[must_use]
    pub fn new(corner: Vec3, x: Vec3, y: Vec3, z: Vec3) -> Self {
        let a = corner;
        let b = corner.add(x);
        let c = corner.add(y);
        let d = corner.add(z);
        let e = b.add(y);
        let f = c.add(z);
        let g = b.add(z);
        let h = e.add(z);
        Self {
            corners: [a, b, c, d, e, f, g, h],
            sides: [
                Rectangle::new(h, f, d, g),
                Rectangle::new(c, e, b, a),
                Rectangle::new(f, c, a, d),
                Rectangle::new(e, h, g, b),
                Rectangle::new(c, f, h, e),
                Rectangle::new(b, g, d, a),
            ],
            local_x: x,
            local_y: y,
            local_z: z,
        }
    }

    #[must_use]
    pub fn center(&self) -> Vec3 {
        self.corners[0].add(self.corners[7].sub(self.corners[0]).scale(0.5))
    }

    #[must_use]
    pub fn bounding_radius(&self) -> f64 {
        self.corners[7].sub(self.corners[0]).length() / 2.0
    }

    #[must_use]
    pub fn closest_corner(&self, p: Vec3) -> usize {
        let mut closest = 0;
        let mut minimum = self.corners[0].sub(p).norm_squared();
        let mut index = 1_u8;
        while index < 8 {
            let corner = match index {
                1 => self.corners[1],
                2 => self.corners[2],
                3 => self.corners[3],
                4 => self.corners[4],
                5 => self.corners[5],
                6 => self.corners[6],
                _ => self.corners[7],
            };
            let distance = corner.sub(p).norm_squared();
            if distance < minimum {
                minimum = distance;
                closest = usize::from(index);
            }
            index = index.saturating_add(1);
        }
        closest
    }

    #[must_use]
    pub fn closest_point(&self, p: Vec3) -> Vec3 {
        fn axis_offset(d: Vec3, axis: Vec3) -> Vec3 {
            let half = axis.length() / 2.0;
            let unit = axis.normalize();
            let mut distance = d.dot(unit);
            if distance > half {
                distance = half;
            } else if distance < -half {
                distance = -half;
            }
            unit.scale(distance)
        }
        let center = self.center();
        let d = p.sub(center);
        center
            .add(axis_offset(d, self.local_x))
            .add(axis_offset(d, self.local_y))
            .add(axis_offset(d, self.local_z))
    }

    fn side(&self, index: usize) -> Rectangle {
        match index {
            0 => self.sides[0],
            1 => self.sides[1],
            2 => self.sides[2],
            3 => self.sides[3],
            4 => self.sides[4],
            _ => self.sides[5],
        }
    }

    fn connected_sides(corner: usize) -> (usize, usize, usize) {
        match corner {
            0 => (1, 2, 5),
            1 => (1, 3, 5),
            2 => (1, 2, 4),
            3 => (0, 2, 5),
            4 => (1, 3, 4),
            5 => (0, 2, 4),
            6 => (0, 3, 5),
            _ => (0, 3, 4),
        }
    }

    fn ray_intersect(&self, origin: Vec3, ray: Vec3) -> Option<(usize, f64, usize, f64)> {
        fn slab(axis: Vec3, origin: Vec3, ray: Vec3) -> (f64, f64, bool) {
            let length = axis.norm_squared();
            let arm_origin = axis.dot(origin);
            let arm_ray = axis.dot(ray);
            if arm_ray < 0.0 {
                (
                    (0.5 * length - arm_origin) / arm_ray,
                    (-0.5 * length - arm_origin) / arm_ray,
                    true,
                )
            } else if arm_ray > 0.0 {
                (
                    (-0.5 * length - arm_origin) / arm_ray,
                    (0.5 * length - arm_origin) / arm_ray,
                    false,
                )
            } else {
                (f64::MIN, f64::MAX, false)
            }
        }
        let origin = origin.sub(self.center());
        let x = slab(self.local_x, origin, ray);
        let y = slab(self.local_y, origin, ray);
        let z = slab(self.local_z, origin, ray);
        let mut enter = (x.0, if x.2 { 3 } else { 2 });
        let mut exit = (x.1, if x.2 { 2 } else { 3 });
        if enter.0 < y.0 {
            enter = (y.0, if y.2 { 4 } else { 5 });
        }
        if z.0 > enter.0 {
            enter = (z.0, if z.2 { 0 } else { 1 });
        }
        if y.1 < exit.0 {
            exit = (y.1, if y.2 { 5 } else { 4 });
        }
        if z.1 < exit.0 {
            exit = (z.1, if z.2 { 1 } else { 0 });
        }
        if enter.0 < exit.0 {
            Some((enter.1, enter.0, exit.1, exit.0))
        } else {
            None
        }
    }

    pub fn collide_with_sphere(
        &self,
        sphere: Vec3,
        radius: f64,
        velocity: Vec3,
        point: &mut Vec3,
        time: &mut f64,
        normal: &mut Vec3,
    ) -> bool {
        let connected = Self::connected_sides(self.closest_corner(sphere));
        let mut could_overlap = true;
        let mut shortest = -1.0;
        // Original closestHalfspace is set before use on the overlap path.
        let mut closest_halfspace = connected.0;
        let mut index = 0_u8;
        while index < 3 {
            let side_id = match index {
                0 => connected.0,
                1 => connected.1,
                _ => connected.2,
            };
            let side = self.side(side_id);
            let contact = first_plane_contact(sphere, radius, side.normal);
            let distance = side.normal.dot(side.a.sub(contact));
            if distance >= 0.0 {
                if distance < shortest || shortest == -1.0 {
                    shortest = distance;
                    closest_halfspace = side_id;
                }
            } else {
                could_overlap = false;
                break;
            }
            index = index.saturating_add(1);
        }
        if could_overlap {
            let closest_point = self.closest_point(sphere);
            *normal = sphere.sub(closest_point);
            let normal_length = normal.length();
            if normal_length < radius {
                if normal_length < 1e-10 {
                    let speed = velocity.length();
                    if speed < 1e-10 {
                        let depth = if shortest < 1e-5 { 1e-5 } else { shortest };
                        *normal = self.side(closest_halfspace).normal.scale(depth);
                    } else if let Some((side1, d1, side2, d2)) =
                        self.ray_intersect(sphere, velocity)
                    {
                        let (side, distance) = if d1.abs() < d2.abs() {
                            (side1, d1)
                        } else {
                            (side2, d2)
                        };
                        let depth = distance.abs() * speed;
                        let depth = if depth < 1e-5 { 1e-5 } else { depth };
                        *normal = self.side(side).normal.scale(depth);
                    } else {
                        *normal = self.side(closest_halfspace).normal.scale(shortest);
                    }
                } else {
                    let depth = (radius - normal_length).abs();
                    *normal = normal.normalize().scale(depth);
                }
                *time = 0.0;
                return true;
            }
        }
        let mut index = 0_u8;
        while index < 3 {
            let side_id = match index {
                0 => connected.0,
                1 => connected.1,
                _ => connected.2,
            };
            if self
                .side(side_id)
                .collide_with_sphere(sphere, radius, velocity, point, time, normal)
            {
                return true;
            }
            index = index.saturating_add(1);
        }
        false
    }
}
