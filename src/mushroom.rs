//! Source Evolve mushroom radius calculation. The float power is the real
//! platform operation; no real-number or portable-libm stand-in is substituted.
use destiny_original_spec::{OriginalBallInit, Vec3};

pub(crate) fn initial_ball(position: Vec3) -> OriginalBallInit {
    #[cfg(kani)]
    kani::cover!(true, "actual mushroom creation parameters constructed");
    OriginalBallInit {
        id: 0,
        mass: f64::from(1e20_f32),
        radius: 0.0,
        max_velocity: 0.0,
        is_free: false,
        is_global: false,
        is_massive: true,
        is_interactive: false,
        is_space_junk: false,
        position,
        velocity: Vec3::ZERO,
        agility: 1.0,
        speed_fraction: 1.0,
    }
}

pub(crate) fn radius_at_tick(
    current: i64,
    started: i64,
    tick_interval_ms: i32,
    range: f32,
    duration: f64,
) -> Result<Option<f32>, &'static str> {
    let elapsed = current
        .checked_sub(started)
        .and_then(|ticks| ticks.checked_mul(i64::from(tick_interval_ms)))
        .ok_or("mushroom elapsed time overflow")?;
    let fraction = (elapsed as f32) * 0.001_f32 / duration as f32;
    #[cfg(kani)]
    kani::cover!(true, "actual source mushroom time fraction evaluated");
    if fraction < 1.0 {
        Ok(Some(range * fraction.powf(0.25_f32)))
    } else {
        Ok(None)
    }
}
