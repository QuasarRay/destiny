//! Source EntityWarpIn fields following WarpTo, synchronous follower cleanup
//! and RealWarp. Native/rotation/sensor/event adapters remain separate callers.
use destiny_original_spec::Vec3;

pub(crate) fn command_action(
    distance_squared: f64,
    already_warp: bool,
    same_destination: bool,
) -> u8 {
    #[cfg(kani)]
    kani::cover!(true, "actual WarpTo command action selected");
    if distance_squared < 10_000_000_000.0 {
        1
    } else if already_warp && same_destination {
        0
    } else {
        3
    }
}
pub(crate) fn arrival(
    point: Vec3,
    position: Vec3,
    goto: Vec3,
    min_range: f64,
    current: i64,
) -> Option<(Vec3, Vec3, i64, f64)> {
    #[cfg(kani)]
    kani::cover!(true, "actual EntityWarpIn arrival fields executed");
    let destination = goto.add(position.sub(goto).normalize().scale(min_range));
    let velocity = point.normalize().scale(149_597_870_700.0);
    let effect = current.checked_sub(5)?.max(0);
    Some((
        destination,
        velocity,
        effect,
        position.sub(destination).length(),
    ))
}
