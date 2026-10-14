// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
// Generated from the pinned original by generate_server_action_harnesses.py.
// Model bounds: eight actions, eight parameters/action, four ball projections;
// these concrete empty-park scenarios need at most three actions. Primitive
// values retain their original integer/float/bool types. No kani::assume.

use destiny_original_spec::{
    ActionName, ActionValue as V, BoundedList, NetworkAction, NetworkActionError,
    NetworkActionHistory, NetworkActionRecord, NetworkActions, NetworkBallProjection,
    OriginalBallInit, OriginalParkProjection, Vec3,
};

// Input adapter for the reviewed native-fixture state projection. Every
// created ID is listed by the caller. Native constructor/Blue/spatial and
// Python representation correspondence remain required, never assumed.
fn refresh_fixture_balls(actions: &mut NetworkActions, park: &OriginalParkProjection, ids: &[i64]) {
    actions.balls = BoundedList::new();
    let mut i = 0_usize;
    while i < ids.len() {
        if let Some(id) = ids.get(i) {
            if park.contains(*id) {
                let ball = park.retained(*id);
                assert!(actions.balls.push(NetworkBallProjection {
                    id: V::Integer(ball.id),
                    old_bubble_id: ball.old_bubble,
                    new_bubble_id: ball.new_bubble,
                    cloak_mode: i64::from(ball.is_cloaked),
                    is_interactive: ball.is_interactive,
                    speed_fraction: f64::from(ball.speed_fraction),
                }));
            }
        }
        i = i.saturating_add(1);
    }
}

const BALL_ID_1: V = V::Integer(1_000_000_000_001);
const BALL_ID_2: V = V::Integer(1_000_000_000_002);
const BALL_ID_3: V = V::Integer(1_000_000_000_003);
const CORP_ID_1: V = V::Integer(98_000_001);
const ALLIANCE_ID_1: V = V::Integer(99_000_001);
const INITIAL_TIMESTAMP: i64 = 0;

fn action(name: ActionName, arguments: &[V], local_argument: Option<V>) -> NetworkAction {
    let mut parameters = BoundedList::new();
    let mut i: usize = 0;
    while i < arguments.len() {
        if let Some(value) = arguments.get(i) {
            assert!(parameters.push(*value));
        }
        i = i.saturating_add(1);
    }
    NetworkAction {
        name,
        parameters,
        local_argument,
        stream: None,
    }
}

fn records(entries: &[(V, i64, NetworkAction)]) -> NetworkActionHistory {
    let mut result = NetworkActionHistory::new();
    let mut i: usize = 0;
    while i < entries.len() {
        if let Some((ball_id, timestamp, action)) = entries.get(i) {
            assert!(result.push(NetworkActionRecord {
                ball_id: *ball_id,
                timestamp: *timestamp,
                action: *action
            }));
        }
        i = i.saturating_add(1);
    }
    result
}

fn contains_record(
    history: NetworkActionHistory,
    ball_id: V,
    timestamp: i64,
    action: NetworkAction,
) -> bool {
    let mut i: usize = 0;
    while i < history.len() {
        if history.get(i)
            == Some(NetworkActionRecord {
                ball_id,
                timestamp,
                action,
            })
        {
            return true;
        }
        i = i.saturating_add(1);
    }
    false
}

use destiny_original_spec::OriginalServerParkActions;
fn balls_are_equal_ignoring_bubbles(
    first: destiny_original_spec::OriginalParkBall,
    second: destiny_original_spec::OriginalParkBall,
) -> bool {
    // Every original conjunct is preserved, including its maxVelocity/radius
    // and speedFraction/mass comparisons. Native object list equality remains
    // an external correspondence requirement; these scenarios have no children.
    (first.id == second.id
        && first.mass == second.mass
        && f64::from(first.radius) == f64::from(second.radius)
        && f64::from(first.max_velocity) == f64::from(second.radius)
        && first.is_free == second.is_free
        && first.is_global == second.is_global
        && first.is_massive == second.is_massive
        && first.is_interactive == second.is_interactive
        && first.is_cloaked == second.is_cloaked
        && first.is_moribund == second.is_moribund
        && first.harmonic == second.harmonic
        && first.corporation_id == second.corporation_id
        && first.alliance_id == second.alliance_id
        && first.position.x == second.position.x
        && first.position.y == second.position.y
        && first.position.z == second.position.z
        && first.velocity.x == second.velocity.x
        && first.velocity.y == second.velocity.y
        && first.velocity.z == second.velocity.z
        && first.yaw == second.yaw
        && first.pitch == second.pitch
        && first.roll == second.roll
        && f64::from(first.agility) == f64::from(second.agility)
        && f64::from(first.speed_fraction) == second.mass
        && f64::from(first.speed_fraction) == f64::from(second.speed_fraction)
        && first.mode == second.mode
        && first.goto.x == second.goto.x
        && first.goto.y == second.goto.y
        && first.goto.z == second.goto.z
        && first.follow_id == second.follow_id
        && f64::from(first.follow_range) == f64::from(second.follow_range)
        && first.owner_id == second.owner_id
        && first.effect_stamp == second.effect_stamp
        && first.formation_id == second.formation_id
        && first.mini_balls == second.mini_balls
        && first.mini_capsules == second.mini_capsules
        && first.mini_boxes == second.mini_boxes)
}
fn assert_ball_added_to_client_parks(
    actions: &mut OriginalServerParkActions,
    ball: destiny_original_spec::OriginalParkBall,
    timestamp: i64,
) {
    let history = actions.queue.flush_history();
    let mut i = 0;
    while i < history.len() {
        if let Some(record) = history.get(i) {
            if record.ball_id == V::Integer(ball.id)
                && record.timestamp == timestamp
                && record.action.name == ActionName::AddBallsToPark
            {
                assert!(
                    record.action.parameters.len() + usize::from(record.action.stream.is_some())
                        == 1
                );
                let stream = record.action.stream;
                assert!(stream.is_some());
                let mut reader = OriginalParkProjection::default();
                assert!(reader.read_original_state(&stream.unwrap_or_default(), 0) == Ok(()));
                assert!(reader.contains(ball.id)); // original dictionary index must not raise
                let loaded = reader.retained(ball.id);
                assert!(
                    ball.mini_balls.is_empty()
                        && ball.mini_capsules.is_empty()
                        && ball.mini_boxes.is_empty()
                );
                if balls_are_equal_ignoring_bubbles(ball, loaded) {
                    return;
                }
            }
        }
        i += 1;
    }
    assert!(!true); // original assertFalse(True, "Ball not added ...")
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_add_ball_to_client_parks
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_add_ball_to_client_parks() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.add_ball_to_client_parks(&park, ball) == Ok(()));
    // original-assertion: python/destiny/test/net/server/test_actions.py:224
    assert_ball_added_to_client_parks(&mut actions, park.retained(ball), INITIAL_TIMESTAMP);
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_make_ball_local
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_make_ball_local() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.make_ball_local(&mut park, ball) == Ok(()));
    let expected_action = action(
        ActionName::BallNotGlobal,
        &[V::Integer(park.retained(ball).new_bubble)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:230
    assert!(
        actions.queue.flush_history()
            == records(&[(V::Integer(ball), INITIAL_TIMESTAMP, expected_action)])
    );
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_make_ball_global_makes_ball_global
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_make_ball_global_makes_ball_global() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.make_ball_global(&mut park, ball) == Ok(()));
    // original-assertion: python/destiny/test/net/server/test_actions.py:235
    assert!(park.retained(ball).is_global);
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_make_ball_global_adds_ball_to_client_parks
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_make_ball_global_adds_ball_to_client_parks() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.make_ball_global(&mut park, ball) == Ok(()));
    // original-assertion: python/destiny/test/net/server/test_actions.py:240
    assert_ball_added_to_client_parks(&mut actions, park.retained(ball), INITIAL_TIMESTAMP);
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_make_ball_global_sends_out_signal_with_src_id
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_make_ball_global_sends_out_signal_with_src_id() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(actions.made_global_callback_parameters.is_empty()); // original callback list initialized empty
    actions.made_global_connected = true;
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.make_ball_global(&mut park, ball) == Ok(()));
    // original-assertion: python/destiny/test/net/server/test_actions.py:251
    assert!(
        actions.made_global_callback_parameters.len() == 1
            && actions.made_global_callback_parameters.get(0) == Some(ball)
    );
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_remove_ball_removes_ball_right_away
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_remove_ball_removes_ball_right_away() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.remove_ball(&mut park, ball) == Ok(()));
    // original-assertion: python/destiny/test/net/server/test_actions.py:256
    assert!(!park.contains(ball));
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_remove_ball_does_not_note_removal_of_non_global_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_remove_ball_does_not_note_removal_of_non_global_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.remove_ball(&mut park, ball) == Ok(()));
    // original-assertion: python/destiny/test/net/server/test_actions.py:261
    assert!(actions.queue.flush_history().is_empty());
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_remove_ball_notifies_removal_of_global_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_remove_ball_notifies_removal_of_global_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball = park.add_ball(OriginalBallInit {
        id: -(1_i64),
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: true,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(ball) == Ok(()));
    assert!(actions.remove_ball(&mut park, ball) == Ok(()));
    let expected_action = action(ActionName::RemoveGlobalBall, &[V::Integer(ball)], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:267
    assert!(
        actions.queue.flush_history()
            == records(&[(V::Integer(ball), INITIAL_TIMESTAMP, expected_action)])
    );
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_clean_up_followers
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(129))]
#[cfg_attr(test, test)]
fn test_clean_up_followers() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    let mut actions = OriginalServerParkActions::default();
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let follower = park.add_ball(OriginalBallInit {
        id: 1_i64,
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(follower) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(follower) == Ok(()));
    assert!(!actions.queue.dynamical_orientation); // actual original helper selects old-style AddBall
    let followee = park.add_ball(OriginalBallInit {
        id: 2_i64,
        mass: 1.0_f64,
        radius: 1.0_f32,
        max_velocity: 1.0_f32,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        agility: 1.0_f32,
        speed_fraction: 1.0_f32,
        position: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
        velocity: Vec3 {
            x: 0.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    });
    assert!(park.contains(followee) && !park.scope_error); // original helper must create the object
    assert!(park.insert_new_query_ball(followee) == Ok(()));
    park.follow(follower, followee, 1000.0_f32, false);
    park.set_position(
        follower,
        Vec3 {
            x: 500000000.0_f64,
            y: 0.0_f64,
            z: 0.0_f64,
        },
    );
    let remote = park.remote_followers();
    assert!(remote.is_ok());
    let remote = remote.unwrap_or_default();
    // original-assertion: python/destiny/test/net/server/test_actions.py:436
    assert!(remote.len() == 1);
    assert!(actions.clean_up_followers(&park) == Ok(()));
    let expected_action = action(ActionName::Stop, &[V::Integer(follower)], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:439
    assert!(
        actions.queue.flush_history()
            == records(&[(V::Integer(follower), INITIAL_TIMESTAMP, expected_action)])
    );
    park.clear_all();
    actions.queue.clear_park();
    actions.queue.reset_settings_to_default();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original native server action and helper assertions completed"
    );
}
