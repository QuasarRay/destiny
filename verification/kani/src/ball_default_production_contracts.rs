//! Original inherited constructor literals on the actual production helper.
//! Native allocation/class/adapter/lease/Python and public operation relations
//! remain required; neither a default DTO nor this kernel is the whole object.
#[path = "../../../src/ball_defaults.rs"]
mod production;

// original-case: python/destiny/test/test_ball.py::BallTest::test_default_values
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(not(kani), test)]
fn original_detached_ball_constructor_literals() {
    let ball = production::detached_fields(false);
    assert_eq!(ball.id, 0);
    assert_eq!(ball.mass, 0.0);
    assert_eq!(ball.radius, 0.0);
    assert_eq!(ball.max_velocity, 0.0);
    assert!(!ball.is_free);
    assert!(!ball.is_global);
    assert!(!ball.is_massive);
    assert!(!ball.is_interactive);
    assert_eq!(ball.is_cloaked, 0);
    assert!(!ball.is_moribund);
    assert!(!ball.is_massive);
    assert_eq!(ball.harmonic, -1);
    assert_eq!(ball.corporation_id, -1);
    assert_eq!(ball.alliance_id, -1);
    assert_eq!(ball.position.x, 1e10);
    assert_eq!(ball.position.y, 1e10);
    assert_eq!(ball.position.z, 1e10);
    assert_eq!(ball.velocity.x, 0.0);
    assert_eq!(ball.velocity.y, 0.0);
    assert_eq!(ball.velocity.z, 0.0);
    assert_eq!(ball.yaw, 0.0);
    assert_eq!(ball.pitch, 0.0);
    assert_eq!(ball.roll, 0.0);
    assert_eq!(ball.agility, 1.0);
    assert_eq!(ball.speed_fraction, 1.0);
    assert_eq!(ball.mode, destiny_original_spec::OriginalMode::Stop);
    assert_eq!(ball.goto.x, 0.0);
    assert_eq!(ball.goto.y, 0.0);
    assert_eq!(ball.goto.z, 0.0);
    assert_eq!(ball.follow_id, 0);
    assert_eq!(ball.follow_range, 10.0);
    assert_eq!(ball.owner_id, 0);
    assert_eq!(ball.effect_stamp, 0);
    assert_eq!(ball.new_bubble, -1);
    assert_eq!(ball.old_bubble, -1);
    assert_eq!(ball.formation_id as u8, 255);
    assert_eq!(ball.mini_balls.len(), 0);
    assert!(!ball.in_park); // Original Ball.ballpark is None.
    #[cfg(kani)]
    kani::cover!(
        true,
        "all original constructor assertions reached on native helper fields"
    );
}
