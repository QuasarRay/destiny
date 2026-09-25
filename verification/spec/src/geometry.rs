//! Pure translation of the geometry primitives exercised by the original
//! Destiny C++ unit tests. This module intentionally has no Bevy/Avian types.

use crate::Vec3;

impl Vec3 {
    #[must_use]
    pub fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z }
    }

    #[must_use]
    pub fn scale(self, scalar: f64) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }

    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[must_use]
    pub fn length(self) -> f64 {
        self.norm_squared().sqrt()
    }

    #[must_use]
    pub fn normalize(self) -> Self {
        let norm = self.length();
        if norm == 0.0 { self } else { self.scale(1.0 / norm) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Aabb {
    pub low: Vec3,
    pub high: Vec3,
}

impl Aabb {
    #[must_use]
    pub const fn new(low: Vec3, high: Vec3) -> Self {
        Self { low, high }
    }

    pub fn update(&mut self, vertex: Vec3) {
        if vertex.x > self.high.x {
            self.high.x = vertex.x;
        } else if vertex.x < self.low.x {
            self.low.x = vertex.x;
        }
        if vertex.y > self.high.y {
            self.high.y = vertex.y;
        } else if vertex.y < self.low.y {
            self.low.y = vertex.y;
        }
        if vertex.z > self.high.z {
            self.high.z = vertex.z;
        } else if vertex.z < self.low.z {
            self.low.z = vertex.z;
        }
    }

    #[must_use]
    pub fn can_exclude_collision(self, p0: Vec3, p1: Vec3, radius: f64) -> bool {
        (p0.x - radius > self.high.x && p1.x - radius > self.high.x)
            || (p0.x + radius < self.low.x && p1.x + radius < self.low.x)
            || (p0.y - radius > self.high.y && p1.y - radius > self.high.y)
            || (p0.y + radius < self.low.y && p1.y + radius < self.low.y)
            || (p0.z - radius > self.high.z && p1.z - radius > self.high.z)
            || (p0.z + radius < self.low.z && p1.z + radius < self.low.z)
    }

    #[must_use]
    pub fn longest_width(self) -> f64 {
        let dx = self.high.x - self.low.x;
        let dy = self.high.y - self.low.y;
        let dz = self.high.z - self.low.z;
        dx.max(dy.max(dz))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub d: f64,
    pub normal: Vec3,
}

impl Plane {
    #[must_use]
    pub fn new(point: Vec3, normal: Vec3) -> Self {
        Self { d: point.dot(normal), normal }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    #[must_use]
    pub const fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    #[must_use]
    pub fn normal(self) -> Vec3 {
        self.b.sub(self.a).cross(self.c.sub(self.a)).normalize()
    }

    /// Direct translation of original Destiny Triangle::GetClosestPoint.
    #[must_use]
    pub fn closest_point(self, p: Vec3) -> Vec3 {
        let ab = self.b.sub(self.a);
        let ac = self.c.sub(self.a);

        let ap = p.sub(self.a);
        let d1 = ab.dot(ap);
        let d2 = ac.dot(ap);
        if d1 <= 0.0 && d2 <= 0.0 {
            return self.a;
        }

        let bp = p.sub(self.b);
        let d3 = ab.dot(bp);
        let d4 = ac.dot(bp);
        if d3 >= 0.0 && d4 <= d3 {
            return self.b;
        }

        let vc = d1 * d4 - d3 * d2;
        if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            let v = d1 / (d1 - d3);
            return self.a.add(ab.scale(v));
        }

        let cp = p.sub(self.c);
        let d5 = ab.dot(cp);
        let d6 = ac.dot(cp);
        if d6 > 0.0 && d5 <= d6 {
            return self.c;
        }

        let vb = d5 * d2 - d1 * d6;
        if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            let w = d2 / (d2 - d6);
            return self.a.add(ac.scale(w));
        }

        let va = d3 * d6 - d5 * d4;
        if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
            let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
            return self.b.add(self.c.sub(self.b).scale(w));
        }

        let denom = 1.0 / (va + vb + vc);
        let v = vb * denom;
        let w = vc * denom;
        self.a.add(ab.scale(v)).add(ac.scale(w))
    }

    /// Direct translation of original Destiny Triangle::ContainsPoint.
    #[must_use]
    pub fn contains_point(self, p: Vec3) -> bool {
        let v0 = self.c.sub(self.a);
        let v1 = self.b.sub(self.a);
        let v2 = p.sub(self.a);

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
        u >= 0.0 && v >= 0.0 && u + v < 1.0
    }
}
