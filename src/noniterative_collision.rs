// Migration source: 114e89fa584f0e1a81eb0a679aabf41af8870072; preserved Rust base: 8bf4fadf7ad457d933f7bf48d63ba5a43b5a0268.
// Generator SHA-256: fab6b6ba81c834a4a6224bd137e0ffe899a9932f120b3ff854f4daaf182ab791.
// Semantic bundle SHA-256: dbe1c2b4c433d1c53c9f3d1df8c78e05f02d5ab2989d8b79cd838e13704c96c5.
// Source/rule/configuration mappings: verification/migration/rust_transformations.json.
// Regenerate: python tools/migrate_destiny.py generate
// Copyright © 2015, 2023 CCP ehf. Original Destiny MIT license retained.
//! Native old-orientation Gradient/Potential pass, Ballpark.cpp:2745..3005.
//! Shared source-backed math accepts the actual friction/timestep. Native
//! collections are dynamic; each fresh per-pair projection has at most three
//! collision records (source recursionDepth < 2), and each static call at most
//! one. The bounded scratch list is not a park/neighbor-count limit.
//! Partition/harmonic/dead-bubble filters, mini-ball registration/identity,
//! missile dispatch, platform math, callers and host scheduling remain separate.

use crate::iterative_collision::{Body, StaticGeometry};
use destiny_original_spec::{
    BoundedList, OriginalMode, OriginalParkBall, OriginalParkError, OriginalStaticCollisionShape,
    Vec3, box_collision_with_environment, capsule_collision_with_environment,
    original_integrate_duration, sphere_potential_with_environment,
};

fn projection(body: Body) -> OriginalParkBall {
    OriginalParkBall {
        id: body.id,
        in_park: true,
        position: body.position,
        velocity: body.velocity,
        mass: body.mass,
        agility: body.agility,
        radius: body.radius,
        max_velocity: body.max_velocity,
        time_factor: body.time_factor,
        last_g: body.acceleration,
        last_c: body.collision_acceleration,
        is_free: body.is_free,
        is_massive: body.is_massive,
        is_cloaked: i8::from(body.is_cloaked),
        is_global: body.is_global,
        is_space_junk: body.is_space_junk,
        is_moribund: body.is_moribund,
        new_bubble: body.bubble,
        mode: body.mode,
        owner_id: body.owner_id,
        follow_id: body.follow_id,
        last_collision: -1.0,
        ..OriginalParkBall::default()
    }
}

fn candidate(body: Body, other: Body, is_master: bool) -> bool {
    if body.id == other.id || other.is_moribund {
        return false;
    }
    if body.mode == OriginalMode::Missile && body.owner_id >= 0 {
        return body.follow_id == other.id && body.bubble == other.bubble;
    }
    other.is_massive
        && !other.is_cloaked
        && (body.mode == OriginalMode::Missile || other.mode != OriginalMode::Missile)
        && (!is_master || other.bubble == body.bubble || other.is_global)
        && !(body.mode == OriginalMode::Missile
            && (other.id == body.owner_id
                || other.mode == OriginalMode::Miniball && other.owner_id == body.owner_id))
        && !(matches!(other.mode, OriginalMode::Missile | OriginalMode::Mushroom)
            && body.id == other.owner_id)
        && !(other.mode == OriginalMode::Missile && body.id == other.follow_id)
        && (!other.is_space_junk || body.is_space_junk)
}

/// Compute every body's output before publication; other bodies observe the
/// input positions/velocities throughout the ID-ordered gradient pass.
pub(crate) fn resolve(
    bodies: &mut [Body],
    statics: &[StaticGeometry],
    friction: f64,
    dt: f64,
    is_master: bool,
    source_contact_effects: &mut crate::iterative_collision::ContactTrace,
) -> Result<Vec<Option<(Vec3, Vec3)>>, OriginalParkError> {
    assert!(bodies.windows(2).all(|pair| pair[0].id < pair[1].id));
    let mut publication = Vec::with_capacity(bodies.len());
    for body_index in 0..bodies.len() {
        let body = bodies[body_index];
        if (!body.is_free || body.is_moribund) || body.cached_dead {
            publication.push(None);
            continue;
        }
        let mut me = projection(body);
        let mut source_contacts = Vec::new();
        if body.is_massive {
            for &other in bodies.iter() {
                if !candidate(body, other, is_master) {
                    continue;
                }
                // Source response does not inspect the existing record list;
                // per-pair scratch can therefore be reset without constraining
                // the number of actual neighbors. Missile event correspondence
                // still requires a native allocation/dispatch relation.
                me.collisions = BoundedList::new();
                let mut neighbor = projection(other);
                sphere_potential_with_environment(&mut me, &mut neighbor, 0, friction, dt)?;
                append_source_contacts(&mut source_contacts, &me);
            }
            if !matches!(body.mode, OriginalMode::Missile | OriginalMode::Mushroom) {
                for geometry in statics {
                    if geometry.owner == body.id {
                        continue;
                    }
                    me.collisions = BoundedList::new();
                    match geometry.shape {
                        OriginalStaticCollisionShape::Capsule(c) => {
                            capsule_collision_with_environment(&mut me, &c, friction, dt)?
                        }
                        OriginalStaticCollisionShape::Box(b) => {
                            box_collision_with_environment(&mut me, &b, friction, dt)?
                        }
                    }
                    append_source_contacts(&mut source_contacts, &me);
                }
            }
        }
        source_contact_effects.push((body.id, source_contacts));
        bodies[body_index].collision_acceleration = me.last_c;
        publication.push(Some(original_integrate_duration(
            me.position,
            me.velocity,
            me.last_g.add(me.last_c),
            me.mass * f64::from(me.agility),
            friction,
            me.time_factor,
            dt,
            dt,
        )));
    }
    #[cfg(kani)]
    kani::cover!(
        true,
        "actual source non-iterative gradient path completed before publication"
    );
    Ok(publication)
}

#[doc = "src/Ballpark.cpp:3005; preserve each retained kernel append, including recursive duplicates"]
fn append_source_contacts(target: &mut Vec<i64>, ball: &OriginalParkBall) {
    for index in 0_usize..(ball.collisions).len() {
        (target)
            .push(((ball.collisions).get(index)).expect("collision list representation invariant"));
    }
}
