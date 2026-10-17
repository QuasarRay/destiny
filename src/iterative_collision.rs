// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// Copyright © 2015, 2023 CCP ehf. Original Destiny MIT license retained.
//! Ordered, old-orientation sphere impacts from Ballpark.cpp:1988..2713 and
//! CollisionBallProperties.cpp. This is the native iterative collision path.
//!
//! Collections grow with the actual park and configured iteration count. The
//! eight-body/22-location proof projection is not a production capacity limit.
//! Capsule/box checks consume the reviewed source geometry translation. Actual
//! source allocation/candidate identities, mini-ball registration, rotational
//! impulses, host forces and platform math remain required obligations.

use std::collections::BTreeSet;

use destiny_original_spec::{
    OriginalMode, OriginalStaticCollisionShape, Vec3, original_integrate_duration,
    original_two_sphere_time,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct StaticGeometry {
    // Internal per-tick key, distinct from original allocated collider IDs.
    // Source allocation/enumeration and cross-tick identity are not proved by
    // this borrowed geometry projection; no public surrogate ID is supplied.
    pub owner: i64,
    pub shape: OriginalStaticCollisionShape,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ColliderKey {
    None,
    Ball(i64),
    Static(usize),
}

#[derive(Clone, Copy)]
enum Target {
    Ball(usize),
    Static(usize),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Body {
    pub id: i64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub collision_acceleration: Vec3,
    pub mass: f64,
    pub agility: f32,
    pub radius: f32,
    pub max_velocity: f32,
    pub time_factor: f64,
    pub is_free: bool,
    pub is_massive: bool,
    pub is_cloaked: bool,
    pub is_global: bool,
    pub is_space_junk: bool,
    pub is_moribund: bool,
    pub bubble: i64,
    pub mode: OriginalMode,
    pub owner_id: i64,
    pub follow_id: i64,
    pub cached_dead: bool,
}

#[derive(Clone, Copy, Debug)]
struct Location {
    time: f64,
    collider: ColliderKey,
    position: Vec3,
    velocity: Vec3,
}

#[derive(Debug)]
struct Properties {
    locations: Vec<Location>,
    start: f64,
    end: f64,
}

#[derive(Clone, Copy)]
struct Impact {
    time: f64,
    collider: ColliderKey,
    velocity_change: Vec3,
}

#[derive(Clone, Copy)]
struct Collision {
    time: f64,
    collider: Target,
    normal: Vec3,
}

fn integrate(
    body: Body,
    position: Vec3,
    velocity: Vec3,
    friction: f64,
    dt: f64,
    duration: f64,
) -> (Vec3, Vec3) {
    original_integrate_duration(
        position,
        velocity,
        body.acceleration.add(body.collision_acceleration),
        body.mass * f64::from(body.agility),
        friction,
        body.time_factor,
        dt,
        duration,
    )
}

impl Properties {
    fn new(body: Body, friction: f64, dt: f64) -> Self {
        let (position, velocity) = integrate(body, body.position, body.velocity, friction, dt, dt);
        Self {
            locations: vec![
                Location {
                    time: 0.0,
                    collider: ColliderKey::None,
                    position: body.position,
                    velocity: body.velocity,
                },
                Location {
                    time: dt,
                    collider: ColliderKey::None,
                    position,
                    velocity,
                },
            ],
            start: 0.0,
            end: dt,
        }
    }

    fn at(&self, body: Body, friction: f64, dt: f64, time: f64) -> (Vec3, Vec3) {
        let index = self
            .locations
            .iter()
            .take_while(|p| p.time + 1e-4 < time)
            .count()
            .saturating_sub(1);
        let location = self.locations[index];
        if (location.time - time).abs() >= 1e-4 {
            integrate(
                body,
                location.position,
                location.velocity,
                friction,
                dt,
                time - location.time,
            )
        } else {
            (location.position, location.velocity)
        }
    }

    fn ignore(&self, collider: ColliderKey, time: f64) -> bool {
        if self.locations.len() < 2 || time > 1e-4 {
            return false;
        }
        let index = self.locations.len() - 2;
        let lower = self.locations[index].time - 1e-4;
        self.locations[..=index]
            .iter()
            .rev()
            .take_while(|p| p.time >= lower)
            .any(|p| p.collider == collider)
    }

    fn finish(&mut self, body: Body, friction: f64, dt: f64) {
        let previous = *self
            .locations
            .last()
            .expect("initialized collision history");
        if dt > previous.time {
            let (position, velocity) = integrate(
                body,
                previous.position,
                previous.velocity,
                friction,
                dt,
                dt - previous.time,
            );
            self.locations.push(Location {
                time: dt,
                collider: ColliderKey::None,
                position,
                velocity,
            });
            self.start = previous.time;
            self.end = dt;
        }
    }

    fn add(
        &mut self,
        body: Body,
        friction: f64,
        dt: f64,
        impact: Impact,
    ) -> Vec<(f64, ColliderKey)> {
        if self.end <= impact.time {
            return Vec::new();
        }
        let index = self
            .locations
            .iter()
            .rposition(|p| impact.time >= p.time)
            .map_or(0, |i| i + 1);
        // AddImpact reports discarded impacts from newest to oldest.
        let invalidated = self.locations[index..]
            .iter()
            .rev()
            .filter(|p| p.collider != ColliderKey::None)
            .map(|p| (p.time, p.collider))
            .collect();
        let (position, velocity) = if index == 0 {
            (body.position, body.velocity)
        } else {
            let previous = self.locations[index - 1];
            integrate(
                body,
                previous.position,
                previous.velocity,
                friction,
                dt,
                impact.time - previous.time,
            )
        };
        self.locations.truncate(index);
        self.locations.push(Location {
            time: impact.time,
            collider: impact.collider,
            position,
            velocity: velocity.add(impact.velocity_change),
        });
        self.end = impact.time;
        invalidated
    }

    fn remove(
        &mut self,
        body: Body,
        friction: f64,
        dt: f64,
        time: f64,
        collider: ColliderKey,
    ) -> Vec<(f64, ColliderKey)> {
        // Preserve the original found=true/index-zero fallthrough.
        let index = self
            .locations
            .iter()
            .position(|p| p.collider == collider && (p.time - time).abs() < 1e-4)
            .unwrap_or(0);
        if index == 0 {
            self.locations[0].collider = ColliderKey::None;
            self.locations[0].velocity = body.velocity;
        }
        // RemoveImpact, unlike AddImpact, reports oldest to newest.
        let invalidated = self.locations[index + 1..]
            .iter()
            .filter(|p| p.collider != ColliderKey::None)
            .map(|p| (p.time, p.collider))
            .collect();
        self.locations.truncate(index.max(1));
        self.finish(body, friction, dt);
        invalidated
    }
}

struct Solver<'a> {
    bodies: &'a mut [Body],
    properties: Vec<Option<Properties>>,
    friction: f64,
    dt: f64,
    statics: &'a [StaticGeometry],
}

impl Solver<'_> {
    fn ensure(&mut self, index: usize) {
        if self.properties[index].is_none() {
            self.properties[index] =
                Some(Properties::new(self.bodies[index], self.friction, self.dt));
        }
    }

    fn finish(&mut self, index: usize) {
        self.properties[index]
            .as_mut()
            .expect("initialized collision history")
            .finish(self.bodies[index], self.friction, self.dt);
    }

    fn invalidate(&mut self, source: i64, invalidated: Vec<(f64, ColliderKey)>) {
        let mut pending: Vec<_> = invalidated
            .into_iter()
            .rev()
            .map(|(time, id)| (time, id, source))
            .collect();
        // Explicit DFS frames retain recursive source order without imposing
        // the bounded model's 176-depth limit on the native collection.
        while let Some((time, target, collider)) = pending.pop() {
            let ColliderKey::Ball(id) = target else {
                continue;
            };
            let Ok(index) = self.bodies.binary_search_by_key(&id, |body| body.id) else {
                continue;
            };
            if !self.bodies[index].is_free {
                continue;
            }
            let Some(properties) = self.properties[index].as_mut() else {
                continue;
            };
            let removed = properties.remove(
                self.bodies[index],
                self.friction,
                self.dt,
                time,
                ColliderKey::Ball(collider),
            );
            pending.extend(
                removed
                    .into_iter()
                    .rev()
                    .map(|(time, id)| (time, id, self.bodies[index].id)),
            );
        }
    }

    fn add(&mut self, index: usize, mut impact: Impact) {
        let body = self.bodies[index];
        let properties = self.properties[index]
            .as_mut()
            .expect("initialized collision history");
        if properties.end <= impact.time {
            return;
        }
        // The multiplication is f32 before conversion, exactly as in C++.
        let fraction = f64::from(25.0_f32 * body.max_velocity * body.max_velocity)
            / impact.velocity_change.norm_squared();
        if fraction < 1.0 {
            impact.velocity_change = impact.velocity_change.scale(fraction);
        }
        let invalidated = properties.add(body, self.friction, self.dt, impact);
        self.invalidate(body.id, invalidated);
    }

    fn next(&mut self, index: usize, neighbors: &[usize]) -> Option<Collision> {
        let body = self.bodies[index];
        let properties = self.properties[index]
            .as_ref()
            .expect("initialized collision history");
        let start = properties.start;
        let end = properties.end;
        let (p0, _) = properties.at(body, self.friction, self.dt, start);
        let (p1, _) = properties.at(body, self.friction, self.dt, end);
        let mut selected: Option<Collision> = None;
        if !matches!(body.mode, OriginalMode::Missile | OriginalMode::Mushroom) {
            for (index, geometry) in self.statics.iter().enumerate() {
                if geometry.owner == body.id {
                    continue;
                }
                if let Some((time, normal)) = geometry.shape.check(p0, p1, body.radius) {
                    if properties.ignore(ColliderKey::Static(index), time) {
                        continue;
                    }
                    let actual = start + (end - start) * time;
                    if selected.is_none_or(|old| actual < old.time) {
                        selected = Some(Collision {
                            time: actual,
                            collider: Target::Static(index),
                            normal,
                        });
                    }
                }
                if selected.is_some_and(|c| c.time == start) {
                    return selected;
                }
            }
        }
        // Static spheres first, followed by free spheres; ties retain ID order.
        for &collider in neighbors {
            let other = self.bodies[collider];
            if other.is_free || neighbor_ignore(body, other) {
                continue;
            }
            let time = original_two_sphere_time(
                p0,
                p1,
                other.position,
                other.position,
                f64::from(body.radius) + f64::from(other.radius),
            );
            if time >= 0.0
                && !properties.ignore(ColliderKey::Ball(other.id), time)
                && selected.is_none_or(|old| time < old.time)
            {
                selected = Some(Collision {
                    time: start + time * (end - start),
                    collider: Target::Ball(collider),
                    normal: p0
                        .add(p1.sub(p0).scale(time))
                        .sub(other.position)
                        .normalize(),
                });
            }
            if selected.is_some_and(|c| c.time == start) {
                return selected;
            }
        }
        for &collider in neighbors {
            let other = self.bodies[collider];
            if !other.is_free
                || !(other.id > body.id || body.mode == OriginalMode::Missile)
                || neighbor_ignore(body, other)
            {
                continue;
            }
            self.ensure(collider);
            let properties = self.properties[index]
                .as_ref()
                .expect("initialized collision history");
            let other_properties = self.properties[collider]
                .as_ref()
                .expect("initialized collision history");
            let stop = end.min(other_properties.end);
            if start >= stop {
                continue;
            }
            let mut times = vec![start];
            times.extend(
                other_properties
                    .locations
                    .iter()
                    .filter(|p| p.time > start && p.time < stop)
                    .map(|p| p.time),
            );
            times.push(stop);
            let (mut bp0, _) = properties.at(body, self.friction, self.dt, times[0]);
            let (mut op0, _) = other_properties.at(other, self.friction, self.dt, times[0]);
            for pair in times.windows(2) {
                let (bp1, _) = properties.at(body, self.friction, self.dt, pair[1]);
                let (op1, _) = other_properties.at(other, self.friction, self.dt, pair[1]);
                let time = original_two_sphere_time(
                    bp0,
                    bp1,
                    op0,
                    op1,
                    f64::from(body.radius) + f64::from(other.radius),
                );
                if time >= 0.0 {
                    let actual = pair[0] + (pair[1] - pair[0]) * time;
                    if selected.is_none_or(|old| actual < old.time) {
                        if properties.ignore(ColliderKey::Ball(other.id), time) {
                            continue;
                        }
                        selected = Some(Collision {
                            time: actual,
                            collider: Target::Ball(collider),
                            normal: Vec3::ZERO,
                        });
                        break;
                    }
                }
                if times.len() > 2 {
                    bp0 = bp1;
                    op0 = op1;
                }
            }
            if selected.is_some_and(|c| c.time == start) {
                break;
            }
        }
        selected
    }

    fn response(&mut self, index: usize, neighbors: &[usize]) -> bool {
        self.ensure(index);
        let properties = self.properties[index]
            .as_ref()
            .expect("initialized collision history");
        if properties.start >= properties.end {
            self.finish(index);
            return false;
        }
        if properties.locations[0].collider != ColliderKey::None
            || neighbors.is_empty() && self.statics.is_empty()
        {
            return false;
        }
        let Some(collision) = self.next(index, neighbors) else {
            self.finish(index);
            return false;
        };
        let body = self.bodies[index];
        let other = match collision.collider {
            Target::Ball(index) => Some((index, self.bodies[index])),
            Target::Static(_) => None,
        };
        if other.is_none_or(|(_, body)| !body.is_free) {
            let properties = self.properties[index]
                .as_ref()
                .expect("initialized collision history");
            if properties.start == collision.time {
                self.bodies[index].collision_acceleration = collision.normal.scale(
                    f64::from(body.max_velocity) * self.friction
                        / (body.mass * f64::from(body.agility))
                        - body.acceleration.dot(collision.normal),
                );
            }
            let (_, velocity) =
                properties.at(self.bodies[index], self.friction, self.dt, collision.time);
            let impulse = velocity.scale(-2.0).dot(collision.normal);
            let key = match collision.collider {
                Target::Ball(index) => ColliderKey::Ball(self.bodies[index].id),
                Target::Static(index) => ColliderKey::Static(index),
            };
            self.add(
                index,
                Impact {
                    time: collision.time,
                    collider: if impulse < 0.0 {
                        ColliderKey::Ball(0)
                    } else {
                        key
                    },
                    velocity_change: if impulse < 0.0 {
                        Vec3::ZERO
                    } else {
                        collision.normal.scale(impulse)
                    },
                },
            );
        } else {
            let (other_index, other) = other.expect("free sphere target");
            let (position, velocity) = self.properties[index]
                .as_ref()
                .expect("initialized collision history")
                .at(body, self.friction, self.dt, collision.time);
            let (other_position, other_velocity) = self.properties[other_index]
                .as_ref()
                .expect("initialized collision history")
                .at(other, self.friction, self.dt, collision.time);
            let mut normal = position.sub(other_position);
            let length = normal.length();
            if length > 0.0 {
                normal = normal.scale(1.0 / length);
            } else {
                normal.z = 1.0;
            }
            let other_normal = normal.scale(-1.0);
            if collision.time == 0.0 {
                self.bodies[index].collision_acceleration = normal.scale(
                    f64::from(body.max_velocity) * self.friction
                        / (body.mass * f64::from(body.agility))
                        - body.acceleration.dot(normal),
                );
                self.bodies[other_index].collision_acceleration = other_normal.scale(
                    f64::from(other.max_velocity) * self.friction
                        / (other.mass * f64::from(other.agility))
                        - other.acceleration.dot(other_normal),
                );
            }
            let impulse = other_velocity.sub(velocity).dot(other_normal) * -2.0;
            let negative = impulse < 0.0;
            let impulse = if negative {
                0.0
            } else {
                body.mass * other.mass * impulse / (body.mass + other.mass)
            };
            self.add(
                other_index,
                Impact {
                    time: collision.time,
                    collider: ColliderKey::Ball(if negative { 0 } else { body.id }),
                    velocity_change: if negative {
                        Vec3::ZERO
                    } else {
                        other_normal.scale(impulse / other.mass)
                    },
                },
            );
            self.add(
                index,
                Impact {
                    time: collision.time,
                    collider: ColliderKey::Ball(if negative { 0 } else { other.id }),
                    velocity_change: if negative {
                        Vec3::ZERO
                    } else {
                        normal.scale(impulse / body.mass)
                    },
                },
            );
        }
        self.finish(index);
        true
    }
}

fn neighbor_ignore(body: Body, other: Body) -> bool {
    (body.mode == OriginalMode::Missile
        && (other.id == body.owner_id
            || other.mode == OriginalMode::Miniball && other.owner_id == body.owner_id))
        || (matches!(other.mode, OriginalMode::Missile | OriginalMode::Mushroom)
            && body.id == other.owner_id)
        || (other.mode == OriginalMode::Missile && body.id == other.follow_id)
        || (other.is_space_junk && !body.is_space_junk)
        || !other.is_massive
}

fn candidates(bodies: &[Body], index: usize, is_master: bool) -> Vec<usize> {
    let body = bodies[index];
    bodies
        .iter()
        .enumerate()
        .filter(|(other_index, other)| {
            *other_index != index
                && !other.is_moribund
                && if body.mode == OriginalMode::Missile && body.owner_id >= 0 {
                    other.id == body.follow_id && other.bubble == body.bubble
                } else {
                    other.is_massive
                        && !other.is_cloaked
                        && (body.mode == OriginalMode::Missile
                            || other.mode != OriginalMode::Missile)
                        && (!is_master || other.bubble == body.bubble || other.is_global)
                }
        })
        .map(|(index, _)| index)
        .collect()
}

/// Inputs are in strictly ascending ID order, with the actual configured
/// timestep/iteration count. The native caller validates its API envelope and
/// rejects non-finite results before publication; positive finite arithmetic
/// and every f32 conversion remain caller/domain proof obligations. In
/// particular, no universal positive-mass assumption is claimed for that API.
/// Returns a simultaneous position/velocity publication plan; the caller must
/// publish it under exclusive operation ownership and retain identity checks.
pub(crate) fn resolve(
    bodies: &mut [Body],
    statics: &[StaticGeometry],
    friction: f64,
    dt: f64,
    iterations: usize,
    is_master: bool,
    source_contact_effects: &mut crate::iterative_collision::ContactTrace,
) -> Vec<Option<(Vec3, Vec3)>> {
    assert!(bodies.windows(2).all(|pair| pair[0].id < pair[1].id));
    let free: BTreeSet<_> = bodies
        .iter()
        .enumerate()
        .filter(|(_, body)| body.is_free)
        .map(|(index, _)| index)
        .collect();
    let mut solver = Solver {
        properties: (0..bodies.len()).map(|_| None).collect(),
        bodies,
        statics,
        friction,
        dt,
    };
    let mut current = free.clone();
    let mut neighbors: Vec<Option<Vec<usize>>> = (0..solver.bodies.len()).map(|_| None).collect();
    for _ in 0..iterations {
        let mut next = BTreeSet::new();
        for index in current {
            let body = solver.bodies[index];
            if (body.is_moribund || !body.is_massive) || body.cached_dead {
                continue;
            }
            let nearby =
                neighbors[index].get_or_insert_with(|| candidates(solver.bodies, index, is_master));
            if solver.response(index, nearby) {
                next.extend(
                    nearby
                        .iter()
                        .copied()
                        .filter(|&index| solver.bodies[index].is_free),
                );
                next.insert(index);
            }
        }
        if next.is_empty() {
            break;
        }
        current = next;
    }
    for index in 0..solver.bodies.len() {
        source_contact_effects.push((
            solver.bodies[index].id,
            source_location_contacts(solver.properties[index].as_ref(), solver.statics),
        ));
    }
    #[cfg(kani)]
    kani::cover!(
        true,
        "production ordered sphere solver completed the actual iteration path"
    );
    (0..solver.bodies.len())
        .map(|index| {
            let body = solver.bodies[index];
            if (!body.is_free || body.is_moribund) || body.cached_dead {
                return None;
            }
            Some(match &solver.properties[index] {
                Some(properties) => properties.at(body, friction, dt, dt),
                None => integrate(body, body.position, body.velocity, friction, dt, dt),
            })
        })
        .collect()
}

pub(crate) type ContactTrace = Vec<(i64, Vec<i64>)>;
#[doc = "src/Ballpark.cpp:617-626; loc.collider != -1, preserve order/duplicates/zero and actual collider IDs"]
fn source_location_contacts(
    properties: Option<&Properties>,
    statics: &[StaticGeometry],
) -> Vec<i64> {
    let mut ids = Vec::new();
    if let Some(properties) = properties {
        for location in &properties.locations {
            let id = match location.collider {
                ColliderKey::None => -1_i64,
                ColliderKey::Ball(id) => id,
                ColliderKey::Static(index) => (statics[index].shape).id(),
            };
            if (id) != (-1_i64) {
                (ids).push(id);
            }
        }
    }
    ids
}
