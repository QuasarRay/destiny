//! Source MissileFollow admission and field update, separate from callback,
//! registry, engine integration and Python signed-byte boundaries.
#[cfg(not(verus_keep_ghost))]
use destiny_original_spec::Vec3;

macro_rules! missile_admission_body {
    ($declare:ident) => {
        $declare! {
            admitted(src: i64, target: i64, owner: i64, src_exists: bool, target_exists: bool, moribund: bool) -> bool {
                #[cfg(kani)]
                kani::cover!(true, "actual MissileFollow admission executed");
                src != target && src != owner && target != owner && src_exists && target_exists && !moribund
            }
        }
    };
}
#[cfg(verus_keep_ghost)]
pub(crate) use missile_admission_body;
#[cfg(not(verus_keep_ghost))]
macro_rules! rust_missile_admission {
    ($name:ident($src:ident:i64,$target:ident:i64,$owner:ident:i64,$src_exists:ident:bool,$target_exists:ident:bool,$moribund:ident:bool)->bool $body:block) => {
        pub(crate) fn $name($src:i64,$target:i64,$owner:i64,$src_exists:bool,$target_exists:bool,$moribund:bool)->bool $body
    };
}
#[cfg(not(verus_keep_ghost))]
missile_admission_body!(rust_missile_admission);

#[cfg(not(verus_keep_ghost))]
#[derive(Clone, Copy)]
pub(crate) struct Fields {
    pub follow_id: i64,
    pub owner_id: i64,
    pub effect_stamp: i64,
    pub follow_range: f32,
    pub destination: Vec3,
    pub speed_fraction: f32,
}

#[cfg(not(verus_keep_ghost))]
pub(crate) fn follow_fields(
    target: i64,
    owner: i64,
    current_time: i64,
    src_radius: f32,
    target_radius: f32,
    velocity: Vec3,
) -> Fields {
    #[cfg(kani)]
    kani::cover!(true, "actual MissileFollow fields executed");
    Fields {
        follow_id: target,
        owner_id: owner,
        effect_stamp: current_time,
        follow_range: -(src_radius + target_radius),
        destination: velocity.normalize().scale(1e16),
        speed_fraction: 1.0,
    }
}

#[cfg(not(verus_keep_ghost))]
pub(crate) fn launch_velocity(
    velocity: Vec3,
    max_velocity: f32,
    stationary_direction: Vec3,
    target_offset: Option<Vec3>,
    aimed: u8,
    target_cloaked: bool,
) -> (Vec3, f64) {
    #[cfg(kani)]
    kani::cover!(true, "actual source launcher velocity executed");
    if let Some(offset) = target_offset.filter(|_| aimed == 1 && !target_cloaked) {
        (offset.normalize(), 0.0)
    } else {
        let mut direction = velocity.normalize();
        if direction.norm_squared() == 0.0 {
            direction = stationary_direction;
        }
        let maximum = f64::from(max_velocity);
        (
            velocity.add(direction.scale(150.0f64.max(maximum))),
            maximum,
        )
    }
}
