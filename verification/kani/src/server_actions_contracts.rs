// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
// Generated from the pinned original by generate_server_action_harnesses.py.
// Model bounds: eight actions, eight parameters/action, four ball projections;
// these concrete empty-park scenarios need at most three actions. Primitive
// values retain their original integer/float/bool types. No kani::assume.

use destiny_original_spec::{ActionName, ActionValue as V, BoundedList, NetworkAction,
    NetworkActionError, NetworkActionHistory, NetworkActionRecord, NetworkActions,
    OriginalParkProjection, OriginalBallInit, NetworkBallProjection, Vec3};

// Input adapter for the reviewed native-fixture state projection. Every
// created ID is listed by the caller. Native constructor/Blue/spatial and
// Python representation correspondence remain required, never assumed.
fn refresh_fixture_balls(actions: &mut NetworkActions, park: &OriginalParkProjection, ids: &[i64]) {
    actions.balls=BoundedList::new();
    let mut i=0_usize;
    while i<ids.len() {
        if let Some(id)=ids.get(i) {
            if park.contains(*id) {
                let ball=park.retained(*id);
                assert!(actions.balls.push(NetworkBallProjection {
                    id:V::Integer(ball.id),old_bubble_id:ball.old_bubble,
                    new_bubble_id:ball.new_bubble,cloak_mode:i64::from(ball.is_cloaked),
                    is_interactive:ball.is_interactive,speed_fraction:f64::from(ball.speed_fraction),
                }));
            }
        }
        i=i.saturating_add(1);
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
        if let Some(value) = arguments.get(i) { assert!(parameters.push(*value)); }
        i = i.saturating_add(1);
    }
    NetworkAction { name, parameters, local_argument, stream:None }
}

fn records(entries: &[(V, i64, NetworkAction)]) -> NetworkActionHistory {
    let mut result = NetworkActionHistory::new();
    let mut i: usize = 0;
    while i < entries.len() {
        if let Some((ball_id, timestamp, action)) = entries.get(i) {
            assert!(result.push(NetworkActionRecord { ball_id: *ball_id,
                timestamp: *timestamp, action: *action }));
        }
        i = i.saturating_add(1);
    }
    result
}

fn contains_record(history: NetworkActionHistory, ball_id: V, timestamp: i64,
    action: NetworkAction) -> bool
{
    let mut i: usize = 0;
    while i < history.len() {
        if history.get(i) == Some(NetworkActionRecord { ball_id, timestamp, action }) { return true; }
        i = i.saturating_add(1);
    }
    false
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_go_to_point
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn go_to_point() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Integer(100);
    let y = V::Integer(200);
    let z = V::Integer(300);
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    let expected_action = action(
        ActionName::GotoPoint,
        &[BALL_ID_1, V::Integer(100), V::Integer(200), V::Integer(300)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:90
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}
// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_duplicate_subsequent_actions_are_ignored
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn duplicate_subsequent_actions_are_ignored() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Integer(100);
    let y = V::Integer(200);
    let z = V::Integer(300);
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    let expected_action = action(
        ActionName::GotoPoint,
        &[BALL_ID_1, V::Integer(100), V::Integer(200), V::Integer(300)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:97
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_duplicate_subsequent_actions_with_different_timestamps_are_not_ignored
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn duplicate_subsequent_actions_with_different_timestamps_are_not_ignored() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Integer(100);
    let y = V::Integer(200);
    let z = V::Integer(300);
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    actions.set_stamp_for_system(INITIAL_TIMESTAMP.saturating_add(1));
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    let expected_action = action(
        ActionName::GotoPoint,
        &[BALL_ID_1, V::Integer(100), V::Integer(200), V::Integer(300)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:105
    assert!(
        actions.flush_history()
            == records(&[
                (BALL_ID_1, INITIAL_TIMESTAMP, expected_action),
                (
                    BALL_ID_1,
                    INITIAL_TIMESTAMP.saturating_add(1),
                    expected_action
                )
            ])
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_duplicate_non_subsequent_actions_are_not_ignored
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn duplicate_non_subsequent_actions_are_not_ignored() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Integer(100);
    let y = V::Integer(200);
    let z = V::Integer(300);
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    assert!(actions.stop(BALL_ID_1) == Ok(()));
    assert!(actions.go_to_point(BALL_ID_1, x, y, z) == Ok(()));
    let goto_action = action(ActionName::GotoPoint, &[BALL_ID_1, x, y, z], None);
    let stop_action = action(ActionName::Stop, &[BALL_ID_1], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:119
    assert!(
        actions.flush_history()
            == records(&[
                (BALL_ID_1, INITIAL_TIMESTAMP, goto_action),
                (BALL_ID_1, INITIAL_TIMESTAMP, stop_action),
                (BALL_ID_1, INITIAL_TIMESTAMP, goto_action)
            ])
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_go_to_direction
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn go_to_direction() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Integer(100);
    let y = V::Integer(200);
    let z = V::Integer(300);
    assert!(actions.go_to_direction(BALL_ID_1, x, y, z) == Ok(()));
    let expected_action = action(ActionName::GotoDirection, &[BALL_ID_1, x, y, z], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:131
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_orbit
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn orbit() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let range_meters = V::Integer(123);
    assert!(actions.orbit(BALL_ID_1, BALL_ID_2, range_meters) == Ok(()));
    let expected_action = action(
        ActionName::Orbit,
        &[BALL_ID_1, BALL_ID_2, range_meters],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:137
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_troll
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_troll() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let tick_delay = V::Integer(42);
    assert!(actions.set_ball_troll(BALL_ID_1, tick_delay) == Ok(()));
    let expected_action = action(ActionName::SetBallTroll, &[BALL_ID_1, tick_delay], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:143
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_velocity
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_velocity() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let vx = V::Float(333.3);
    let vy = V::Float(666.6);
    let vz = V::Float(999.9);
    assert!(actions.set_ball_velocity(BALL_ID_1, vx, vy, vz) == Ok(()));
    let expected_action = action(ActionName::SetBallVelocity, &[BALL_ID_1, vx, vy, vz], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:149
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_angular_velocity
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_angular_velocity() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = true;
    let wx = V::Float(1.0);
    let wy = V::Float(2.0);
    let wz = V::Float(3.0);
    assert!(actions.set_ball_angular_velocity(BALL_ID_1, wx, wy, wz) == Ok(()));
    let expected_action = action(
        ActionName::SetBallAngularVelocity,
        &[BALL_ID_1, wx, wy, wz],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:156
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_angular_velocity_raises_when_dynamical_orientation_disabled
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_angular_velocity_raises_when_dynamical_orientation_disabled() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = false;
    let wx = V::Float(1.0);
    let wy = V::Float(2.0);
    let wz = V::Float(3.0);
    // original-assertion: python/destiny/test/net/server/test_actions.py:161
    assert!(
        actions.set_ball_angular_velocity(BALL_ID_1, wx, wy, wz)
            == Err(NetworkActionError::DynamicalOrientationDisabled)
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_max_angular_velocity
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_max_angular_velocity() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = true;
    let wx = V::Float(1.0);
    let wy = V::Float(2.0);
    let wz = V::Float(3.0);
    assert!(actions.set_max_angular_velocity(BALL_ID_1, wx, wy, wz) == Ok(()));
    let expected_action = action(
        ActionName::SetMaxAngularVelocity,
        &[BALL_ID_1, wx, wy, wz],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:168
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_max_angular_velocity_raises_when_dynamical_orientation_disabled
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_max_angular_velocity_raises_when_dynamical_orientation_disabled() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = false;
    let wx = V::Float(1.0);
    let wy = V::Float(2.0);
    let wz = V::Float(3.0);
    // original-assertion: python/destiny/test/net/server/test_actions.py:173
    assert!(
        actions.set_max_angular_velocity(BALL_ID_1, wx, wy, wz)
            == Err(NetworkActionError::DynamicalOrientationDisabled)
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_rotation
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_rotation() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = true;
    let rx = V::Float(0.0);
    let ry = V::Float(0.1);
    let rz = V::Float(0.2);
    let rw = V::Float(0.3);
    assert!(actions.set_ball_rotation(BALL_ID_1, rx, ry, rz, rw) == Ok(()));
    let expected_action = action(
        ActionName::SetBallRotation,
        &[BALL_ID_1, rx, ry, rz, rw],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:180
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_rotation_raises_when_dynamical_orientation_disabled
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_rotation_raises_when_dynamical_orientation_disabled() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = false;
    let rx = V::Float(0.0);
    let ry = V::Float(0.1);
    let rz = V::Float(0.2);
    let rw = V::Float(0.3);
    // original-assertion: python/destiny/test/net/server/test_actions.py:185
    assert!(
        actions.set_ball_rotation(BALL_ID_1, rx, ry, rz, rw)
            == Err(NetworkActionError::DynamicalOrientationDisabled)
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_massive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_massive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_massive(BALL_ID_1, V::Boolean(false)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallMassive,
        &[BALL_ID_1, V::Boolean(false)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:190
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_stop
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn stop() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.stop(BALL_ID_1) == Ok(()));
    let expected_action = action(ActionName::Stop, &[BALL_ID_1], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:195
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_mass
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_mass() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mass_kg = V::Float(72.6);
    assert!(actions.set_ball_mass(BALL_ID_1, mass_kg) == Ok(()));
    let expected_action = action(ActionName::SetBallMass, &[BALL_ID_1, mass_kg], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:201
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_agility
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_agility() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let agility = V::Float(3.141592);
    assert!(actions.set_ball_agility(BALL_ID_1, agility) == Ok(()));
    let expected_action = action(ActionName::SetBallAgility, &[BALL_ID_1, agility], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:207
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_angular_agility
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_angular_agility() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = true;
    let angular_agility = V::Float(0.3);
    assert!(actions.set_ball_angular_agility(BALL_ID_1, angular_agility) == Ok(()));
    let expected_action = action(
        ActionName::SetBallAngularAgility,
        &[BALL_ID_1, angular_agility],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:214
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_angular_agility_raises_when_dynamical_orientation_disabled
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_angular_agility_raises_when_dynamical_orientation_disabled() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = false;
    let angular_agility = V::Float(0.3);
    // original-assertion: python/destiny/test/net/server/test_actions.py:219
    assert!(
        actions.set_ball_angular_agility(BALL_ID_1, angular_agility)
            == Err(NetworkActionError::DynamicalOrientationDisabled)
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_speed_fraction
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_speed_fraction() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let speed_fraction = V::Float(0.5);
    assert!(actions.set_speed_fraction(BALL_ID_1, speed_fraction) == Ok(()));
    let expected_action = action(
        ActionName::SetSpeedFraction,
        &[BALL_ID_1, speed_fraction],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:273
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_warp_to
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn warp_to() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Float(123.0);
    let y = V::Float(456.0);
    let z = V::Float(789.0);
    let minimum_range = V::Float(1000.0);
    let warp_speed = V::Integer(20);
    assert!(actions.warp_to(BALL_ID_1, x, y, z, minimum_range, warp_speed) == Ok(()));
    let expected_action = action(
        ActionName::WarpTo,
        &[BALL_ID_1, x, y, z, minimum_range, warp_speed],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:281
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_entity_warp_in
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn entity_warp_in() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Float(123.0);
    let y = V::Float(456.0);
    let z = V::Float(789.0);
    let warp_speed = V::Integer(20);
    assert!(actions.entity_warp_in(BALL_ID_1, x, y, z, warp_speed) == Ok(()));
    let expected_action = action(
        ActionName::EntityWarpIn,
        &[BALL_ID_1, x, y, z, warp_speed],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:288
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_position_local_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_position_local_ball() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Float(123.0);
    let y = V::Float(456.0);
    let z = V::Float(789.0);
    assert!(actions.set_ball_position(BALL_ID_1, x, y, z, V::Boolean(false)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallPosition,
        &[BALL_ID_1, x, y, z],
        Some(V::Boolean(false)),
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:294
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_position_not_local_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_position_not_local_ball() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let x = V::Float(123.0);
    let y = V::Float(456.0);
    let z = V::Float(789.0);
    assert!(actions.set_ball_position(BALL_ID_1, x, y, z, V::Boolean(true)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallPosition,
        &[BALL_ID_1, x, y, z],
        Some(V::Boolean(true)),
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:300
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_harmonic_on_forcefield_with_all_negative_one_values_makes_field_non_massive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_harmonic_on_forcefield_with_all_negative_one_values_makes_field_non_massive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let harmonic_value = V::Integer(-1);
    let corporation_id = V::Integer(-1);
    let alliance_id = V::Integer(-1);
    assert!(
        actions.set_ball_harmonic(
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(true)
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::SetBallMassive,
        &[BALL_ID_1, V::Boolean(false)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:308
    assert!(contains_record(
        actions.flush_history(),
        BALL_ID_1,
        INITIAL_TIMESTAMP,
        expected_action
    ));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_harmonic_on_forcefield_with_harmonic_value_makes_field_massive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_harmonic_on_forcefield_with_harmonic_value_makes_field_massive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let harmonic_value = V::Integer(0);
    let corporation_id = V::Integer(-1);
    let alliance_id = V::Integer(-1);
    assert!(
        actions.set_ball_harmonic(
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(true)
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::SetBallMassive,
        &[BALL_ID_1, V::Boolean(true)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:316
    assert!(contains_record(
        actions.flush_history(),
        BALL_ID_1,
        INITIAL_TIMESTAMP,
        expected_action
    ));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_harmonic_on_forcefield_with_corporation_id_makes_field_massive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_harmonic_on_forcefield_with_corporation_id_makes_field_massive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let harmonic_value = V::Integer(-1);
    let corporation_id = CORP_ID_1;
    let alliance_id = V::Integer(-1);
    assert!(
        actions.set_ball_harmonic(
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(true)
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::SetBallMassive,
        &[BALL_ID_1, V::Boolean(true)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:324
    assert!(contains_record(
        actions.flush_history(),
        BALL_ID_1,
        INITIAL_TIMESTAMP,
        expected_action
    ));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_harmonic_on_forcefield_with_alliance_id_makes_field_massive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_harmonic_on_forcefield_with_alliance_id_makes_field_massive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let harmonic_value = V::Integer(-1);
    let corporation_id = V::Integer(-1);
    let alliance_id = ALLIANCE_ID_1;
    assert!(
        actions.set_ball_harmonic(
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(true)
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::SetBallMassive,
        &[BALL_ID_1, V::Boolean(true)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:332
    assert!(contains_record(
        actions.flush_history(),
        BALL_ID_1,
        INITIAL_TIMESTAMP,
        expected_action
    ));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_harmonic_on_forcefield_sets_harmonic_values
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_harmonic_on_forcefield_sets_harmonic_values() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let harmonic_value = V::Integer(12345);
    let corporation_id = CORP_ID_1;
    let alliance_id = ALLIANCE_ID_1;
    assert!(
        actions.set_ball_harmonic(
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(true)
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::SetBallHarmonic,
        &[
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(true),
        ],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:340
    assert!(contains_record(
        actions.flush_history(),
        BALL_ID_1,
        INITIAL_TIMESTAMP,
        expected_action
    ));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_harmonic_on_non_forcefield_sets_harmonic_values
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_harmonic_on_non_forcefield_sets_harmonic_values() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let harmonic_value = V::Integer(12345);
    let corporation_id = CORP_ID_1;
    let alliance_id = ALLIANCE_ID_1;
    assert!(
        actions.set_ball_harmonic(
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(false)
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::SetBallHarmonic,
        &[
            BALL_ID_1,
            harmonic_value,
            corporation_id,
            alliance_id,
            V::Boolean(false),
        ],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:348
    assert!(contains_record(
        actions.flush_history(),
        BALL_ID_1,
        INITIAL_TIMESTAMP,
        expected_action
    ));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_radius
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_radius() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let radius_meters = V::Float(42.0);
    assert!(actions.set_ball_radius(BALL_ID_1, radius_meters) == Ok(()));
    let expected_action = action(ActionName::SetBallRadius, &[BALL_ID_1, radius_meters], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:354
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_free
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_free() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_free(BALL_ID_1, V::Boolean(true)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallFree,
        &[BALL_ID_1, V::Boolean(true)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:359
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_not_free
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_not_free() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_free(BALL_ID_1, V::Boolean(false)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallFree,
        &[BALL_ID_1, V::Boolean(false)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:364
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_interactive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_interactive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_interactive(BALL_ID_1, V::Boolean(true)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallInteractive,
        &[BALL_ID_1, V::Boolean(true)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:369
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_not_interactive
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_not_interactive() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_interactive(BALL_ID_1, V::Boolean(false)) == Ok(()));
    let expected_action = action(
        ActionName::SetBallInteractive,
        &[BALL_ID_1, V::Boolean(false)],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:374
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_follow_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn follow_ball() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let range_meters = V::Float(1000.0);
    assert!(actions.follow_ball(BALL_ID_1, BALL_ID_2, range_meters) == Ok(()));
    let expected_action = action(
        ActionName::FollowBall,
        &[BALL_ID_1, BALL_ID_2, range_meters],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:380
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_max_speed
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_max_speed() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let max_meters_per_second = V::Float(500.0);
    assert!(actions.set_max_speed(BALL_ID_1, max_meters_per_second) == Ok(()));
    let expected_action = action(
        ActionName::SetMaxSpeed,
        &[BALL_ID_1, max_meters_per_second],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:386
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_max_angular_speed
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_max_angular_speed() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = true;
    let max_angular_speed = V::Float(6.0);
    assert!(actions.set_max_angular_speed(BALL_ID_1, max_angular_speed) == Ok(()));
    let expected_action = action(
        ActionName::SetMaxAngularSpeed,
        &[BALL_ID_1, max_angular_speed],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:393
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_set_ball_angular_speed_raises_when_dynamical_orientation_disabled
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn set_ball_angular_speed_raises_when_dynamical_orientation_disabled() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    actions.dynamical_orientation = false;
    let wx = V::Float(1.0);
    let wy = V::Float(2.0);
    let wz = V::Float(3.0);
    // original-assertion: python/destiny/test/net/server/test_actions.py:398
    assert!(
        actions.set_ball_angular_velocity(BALL_ID_1, wx, wy, wz)
            == Err(NetworkActionError::DynamicalOrientationDisabled)
    );
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_cloak_nonexistent_ball_should_fail
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn cloak_nonexistent_ball_should_fail() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let uncloak_range_meters = V::Float(1000.0);
    let success = actions.cloak_ball(BALL_ID_1, V::Integer(1), uncloak_range_meters);
    // original-assertion: python/destiny/test/net/server/test_actions.py:403
    assert!(success == Ok(false));
    // original-assertion: python/destiny/test/net/server/test_actions.py:404
    assert!(actions.flush_history().is_empty());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_uncloak_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn uncloak_ball() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.uncloak_ball(BALL_ID_1) == Ok(()));
    let expected_action = action(ActionName::UncloakBall, &[BALL_ID_1], None);
    // original-assertion: python/destiny/test/net/server/test_actions.py:418
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_launch_missile
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn launch_missile() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let src_id = BALL_ID_1;
    let dst_id = BALL_ID_2;
    let owner_id = BALL_ID_3;
    let is_aimed_launch = V::Boolean(true);
    let is_missile_massive = V::Boolean(false);
    assert!(
        actions.launch_missile(
            src_id,
            dst_id,
            owner_id,
            is_aimed_launch,
            is_missile_massive
        ) == Ok(())
    );
    let expected_action = action(
        ActionName::LaunchMissile,
        &[
            src_id,
            dst_id,
            owner_id,
            is_aimed_launch,
            is_missile_massive,
        ],
        None,
    );
    // original-assertion: python/destiny/test/net/server/test_actions.py:428
    assert!(actions.flush_history() == records(&[(src_id, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_undo_pending_cloak_ignores_other_actions
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn undo_pending_cloak_ignores_other_actions() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let radius_meters = V::Float(42.0);
    assert!(actions.set_ball_radius(BALL_ID_1, radius_meters) == Ok(()));
    let expected_action = action(ActionName::SetBallRadius, &[BALL_ID_1, radius_meters], None);
    actions.undo_pending_cloak(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:461
    assert!(actions.flush_history() == records(&[(BALL_ID_1, INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_cloak_returns_false_when_no_pending_cloak
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_cloak_returns_false_when_no_pending_cloak() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let has_pending_cloak = actions.has_pending_cloak(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:465
    assert!(!has_pending_cloak);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_uncloak_returns_false_when_no_pending_uncloak
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_uncloak_returns_false_when_no_pending_uncloak() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let has_pending_uncloak = actions.has_pending_uncloak(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:475
    assert!(!has_pending_uncloak);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_uncloak_returns_true_when_pending_uncloak
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_uncloak_returns_true_when_pending_uncloak() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.uncloak_ball(BALL_ID_1) == Ok(()));
    let has_pending_uncloak = actions.has_pending_uncloak(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:480
    assert!(has_pending_uncloak);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_cloak_mode_returns_none_when_ball_does_not_exist
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_cloak_mode_returns_none_when_ball_does_not_exist() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let cloak_mode = actions.get_pending_cloak_mode(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:484
    assert!(cloak_mode.is_none());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_set_free_returns_false_when_freedom_not_pending
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_set_free_returns_false_when_freedom_not_pending() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let freedom_pending = actions.has_pending_set_free(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:499
    assert!(!freedom_pending.truthy());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_set_free_returns_true_when_freedom_pending
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_set_free_returns_true_when_freedom_pending() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_free(BALL_ID_1, V::Boolean(true)) == Ok(()));
    let freedom_pending = actions.has_pending_set_free(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:510
    assert!(freedom_pending.truthy());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_set_not_free_returns_false_when_freedom_removal_not_pending
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_set_not_free_returns_false_when_freedom_removal_not_pending() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let freedom_removal_pending = actions.has_pending_set_not_free(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:514
    assert!(!freedom_removal_pending.truthy());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_set_not_free_returns_true_when_freedom_removal_pending
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_set_not_free_returns_true_when_freedom_removal_pending() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    assert!(actions.set_ball_free(BALL_ID_1, V::Boolean(false)) == Ok(()));
    let freedom_removal_pending = actions.has_pending_set_not_free(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:525
    assert!(freedom_removal_pending.truthy());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_interactive_state_returns_none_when_ball_does_not_exist
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_interactive_state_returns_none_when_ball_does_not_exist() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let state = actions.get_pending_interactive_state(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:529
    assert!(state.is_none());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_speed_fraction_returns_none_when_ball_does_not_exist
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_speed_fraction_returns_none_when_ball_does_not_exist() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let speed_fraction = actions.get_pending_speed_fraction(BALL_ID_1);
    // original-assertion: python/destiny/test/net/server/test_actions.py:544
    assert!(speed_fraction.is_none());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    #[cfg(kani)]
    kani::cover!(true, "complete original empty-park action scenario");
}
// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_cloak_existing_ball
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn cloak_existing_ball() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    let uncloak_range_meters = V::Float(1000.0);
    let cloak_mode = V::Integer(1);
    let success = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.cloak_ball(V::Integer(ball), cloak_mode, uncloak_range_meters) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:411
    assert!(success == Ok(true));
    let expected_action = action(ActionName::CloakBall, &[V::Integer(ball), cloak_mode, uncloak_range_meters], Some(V::Integer(park.retained(ball).old_bubble)));
    // original-assertion: python/destiny/test/net/server/test_actions.py:413
    assert!(actions.flush_history() == records(&[(V::Integer(ball), INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_cloak_returns_true_when_pending_cloak
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_cloak_returns_true_when_pending_cloak() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(matches!({ refresh_fixture_balls(&mut actions, &park, &[ball]); actions.cloak_ball(V::Integer(ball), V::Integer(1), V::Float(1000.0)) }, Ok(_))); // ignored original bool; no unexpected error
    let has_pending_cloak = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.has_pending_cloak(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:471
    assert!(has_pending_cloak);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_cloak_mode_returns_existing_mode_when_no_cloak_pending
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_cloak_mode_returns_existing_mode_when_no_cloak_pending() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    let cloak_mode = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.get_pending_cloak_mode(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:489
    assert!(Some(V::Integer(0)) == cloak_mode);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_cloak_mode_returns_pending_cloak_mode_when_no_cloak_pending
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_cloak_mode_returns_pending_cloak_mode_when_no_cloak_pending() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(matches!({ refresh_fixture_balls(&mut actions, &park, &[ball]); actions.cloak_ball(V::Integer(ball), V::Integer(1), V::Float(1000.0)) }, Ok(_))); // ignored original bool; no unexpected error
    let cloak_mode = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.get_pending_cloak_mode(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:495
    assert!(Some(V::Integer(1)) == cloak_mode);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_set_free_returns_false_when_freedom_not_pending_and_ball_already_free
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_set_free_returns_false_when_freedom_not_pending_and_ball_already_free() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    park.set_free(ball, true);
    let freedom_pending = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.has_pending_set_free(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:505
    assert!(!freedom_pending.truthy());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_has_pending_set_not_free_returns_false_when_freedom_removal_not_pending_and_ball_already_not_free
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn has_pending_set_not_free_returns_false_when_freedom_removal_not_pending_and_ball_already_not_free() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    park.set_free(ball, false);
    let freedom_removal_pending = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.has_pending_set_not_free(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:520
    assert!(!freedom_removal_pending.truthy());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_interactive_state_returns_current_state_when_ball_exists
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_interactive_state_returns_current_state_when_ball_exists() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    let state = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.get_pending_interactive_state(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:534
    assert!(Some(V::Boolean(true)) == state);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_interactive_state_returns_pending_state_when_available
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_interactive_state_returns_pending_state_when_available() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!({ refresh_fixture_balls(&mut actions, &park, &[ball]); actions.set_ball_interactive(V::Integer(ball), V::Boolean(false)) } == Ok(()));
    let state = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.get_pending_interactive_state(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:540
    assert!(Some(V::Boolean(false)) == state);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_speed_fraction_returns_current_state_when_ball_exists
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_speed_fraction_returns_current_state_when_ball_exists() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 0.5_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    let speed_fraction = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.get_pending_speed_fraction(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:549
    assert!(Some(V::Float(0.5)) == speed_fraction);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_get_pending_speed_fraction_returns_pending_state_when_available
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn get_pending_speed_fraction_returns_pending_state_when_available() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 0.5_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!({ refresh_fixture_balls(&mut actions, &park, &[ball]); actions.set_speed_fraction(V::Integer(ball), V::Float(1.0)) } == Ok(()));
    let speed_fraction = { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.get_pending_speed_fraction(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:555
    assert!(Some(V::Float(1.0)) == speed_fraction);
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_undo_pending_cloak
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn undo_pending_cloak() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    assert!(matches!({ refresh_fixture_balls(&mut actions, &park, &[ball]); actions.cloak_ball(V::Integer(ball), V::Integer(1), V::Float(1000.0)) }, Ok(_))); // ignored original bool; no unexpected error
    { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.undo_pending_cloak(V::Integer(ball)) };
    // original-assertion: python/destiny/test/net/server/test_actions.py:445
    assert!(actions.flush_history().is_empty());
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

// original-case: python/destiny/test/net/server/test_actions.py::TestActions::test_undo_pending_cloak_ignores_other_balls
// original-source-sha256: f1b53d5a4a63fd73df4a0e0770c0bd8e713577278e97657d7c4151230bb2212b
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(17))]
#[cfg_attr(test, test)]
fn undo_pending_cloak_ignores_other_balls() {
    let mut actions = NetworkActions::new_for_empty_park_at_time_zero();
    let mut park=OriginalParkProjection::default();
    park.is_master=true;
    assert!(!actions.dynamical_orientation); // actual original helper selects old-style AddBall
    let ball=park.add_ball(OriginalBallInit { id: -(1_i64), mass: 1.0_f64, radius: 1.0_f32, max_velocity: 1.0_f32, is_free: true, is_global: false, is_massive: true, is_interactive: true, is_space_junk: false, agility: 1.0_f32, speed_fraction: 1.0_f32, position: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 }, velocity: Vec3 { x: 0.0_f64, y: 0.0_f64, z: 0.0_f64 } });
    assert!(park.contains(ball) && !park.scope_error); // original helper must create the object
    let uncloak_range_meters = V::Float(1000.0);
    let cloak_mode = V::Integer(1);
    assert!(matches!({ refresh_fixture_balls(&mut actions, &park, &[ball]); actions.cloak_ball(V::Integer(ball), cloak_mode, uncloak_range_meters) }, Ok(_))); // ignored original bool; no unexpected error
    { refresh_fixture_balls(&mut actions, &park, &[ball]); actions.undo_pending_cloak(V::Integer(ball + 1_i64)) };
    let expected_action = action(ActionName::CloakBall, &[V::Integer(ball), cloak_mode, uncloak_range_meters], Some(V::Integer(park.retained(ball).old_bubble)));
    // original-assertion: python/destiny/test/net/server/test_actions.py:454
    assert!(actions.flush_history() == records(&[(V::Integer(ball), INITIAL_TIMESTAMP, expected_action)]));
    // Original tearDown: ClearAll, release objects, reset settings.
    actions.clear_park();
    actions.reset_settings_to_default();
    assert!(!park.scope_error);
    park.clear_all();
    assert!(!park.scope_error);
    #[cfg(kani)]
    kani::cover!(true, "original action assertions and native-fixture projection teardown completed");
}

