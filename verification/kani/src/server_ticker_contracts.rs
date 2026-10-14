// Copyright © 2023 CCP ehf. Original tests' MIT license retained.
// Generated from pinned Git blobs by generate_server_ticker_harnesses.py.
// Complete fixture/action ordering, real evolution, bubbles, bytes and casts.
// Native parent lookup, Blue/Python aliases and signal lifetime remain open.
#![allow(non_snake_case)]
use destiny_original_spec::{
    ActionName, ActionValue as V, BatcherEvent, BatcherHistory, BatcherPayload, BoundedList,
    OriginalBallInit, OriginalMode, OriginalNetworkCast, OriginalParkProjection,
    OriginalServerTicker, ServerTickerSignal, Vec3,
};

const BALL_ID_1: i64 = 1001;
const BALL_ID_2: i64 = 1002;
const BALL_ID_3: i64 = 1003;
const CHAR_1: i64 = 90000128;
const CHAR_2: i64 = 90000256;
const CHAR_3: i64 = 90000512;
const CLIENT_1: i64 = 13579;
const CLIENT_2: i64 = 24680;
const CLIENT_3: i64 = 80808;
fn ids(values: &[i64]) -> BoundedList<i64, 8> {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}
fn args(values: &[V]) -> BoundedList<V, 8> {
    let mut result = BoundedList::new();
    let mut i = 0;
    while i < values.len() {
        if let Some(value) = values.get(i) {
            assert!(result.push(*value));
        }
        i += 1;
    }
    result
}
fn ids_equal(left: &BoundedList<i64, 8>, right: &BoundedList<i64, 8>) -> bool {
    let mut i = 0;
    while i < left.len() {
        let mut found = false;
        let mut j = 0;
        while j < right.len() {
            found |= left.get(i) == right.get(j);
            j += 1;
        }
        if !found {
            return false;
        }
        i += 1;
    }
    i = 0;
    while i < right.len() {
        let mut found = false;
        let mut j = 0;
        while j < left.len() {
            found |= right.get(i) == left.get(j);
            j += 1;
        }
        if !found {
            return false;
        }
        i += 1;
    }
    true
}
fn add_ball(park: &mut OriginalParkProjection, id: i64) -> i64 {
    assert!(!park.dynamical_orientation); // actual helper selects old-style AddBall
    let replacing = park.contains(id);
    let ball = park.add_ball(OriginalBallInit {
        id,
        mass: 1.0,
        radius: 1.0,
        max_velocity: 1.0,
        is_free: true,
        is_global: false,
        is_massive: true,
        is_interactive: true,
        is_space_junk: false,
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        agility: 1.0,
        speed_fraction: 1.0,
    });
    assert!(park.contains(ball) && !park.scope_error);
    if !replacing {
        assert!(park.insert_new_query_ball(ball) == Ok(()));
    }
    ball
}
fn set_up_ball(
    park: &mut OriginalParkProjection,
    ticker: &mut OriginalServerTicker,
    id: i64,
    character: i64,
    client: i64,
) -> i64 {
    let ball = add_ball(park, id);
    assert!(ticker.interests.add_client_interest(id, client).is_ok());
    assert!(
        ticker
            .interests
            .add_character_interest(id, character)
            .is_ok()
    );
    assert!(
        ticker
            .interests
            .map_to_client(character, Some(client))
            .is_ok()
    );
    assert!(
        ticker
            .interests
            .set_ball_info_characters(id, &[character])
            .is_ok()
    );
    ball
}
fn perform_tick(
    park: &mut OriginalParkProjection,
    ticker: &mut OriginalServerTicker,
    skip_evolve: bool,
) {
    assert!(ticker.tick(park) == Ok(()));
    if !skip_evolve {
        assert!(park.evolve_two_spherical_bodies() == Ok(()));
    }
    assert!(ticker.send_batch(park) == Ok(()));
}
fn assert_update(
    ticker: &OriginalServerTicker,
    clients: BoundedList<i64, 8>,
    event: BatcherEvent,
    single: bool,
) {
    let casts = if single {
        &ticker.network.singlecasts
    } else {
        &ticker.network.narrowcasts
    };
    let mut i = 0;
    while i < casts.len() {
        if let Some(cast) = casts.get(i) {
            // Every actual source batcher dispatch uses DoDestinyUpdate.
            let selected = if single {
                cast.clients.get(0) == clients.get(0)
            } else {
                ids_equal(&cast.clients, &clients)
            };
            if selected {
                let mut j = 0;
                while j < cast.history.len() {
                    if cast.history.get(j) == Some(event) {
                        return;
                    }
                    j += 1;
                }
            }
        }
        i += 1;
    }
    assert!(false); // original full helper assertTrue(False,...)
}
fn assert_balls_equal_without_bubbles(
    first: destiny_original_spec::OriginalParkBall,
    second: destiny_original_spec::OriginalParkBall,
) {
    // original-assertion: python/destiny/test/net/server/helpers.py:142
    assert!(first.id == second.id);
    // original-assertion: python/destiny/test/net/server/helpers.py:143
    assert!(first.mass == second.mass);
    // original-assertion: python/destiny/test/net/server/helpers.py:144
    assert!(f64::from(first.radius) == f64::from(second.radius));
    // original-assertion: python/destiny/test/net/server/helpers.py:145
    assert!(f64::from(first.max_velocity) == f64::from(second.max_velocity));
    // original-assertion: python/destiny/test/net/server/helpers.py:146
    assert!(first.is_free == second.is_free);
    // original-assertion: python/destiny/test/net/server/helpers.py:147
    assert!(first.is_global == second.is_global);
    // original-assertion: python/destiny/test/net/server/helpers.py:148
    assert!(first.is_massive == second.is_massive);
    // original-assertion: python/destiny/test/net/server/helpers.py:149
    assert!(first.is_interactive == second.is_interactive);
    // original-assertion: python/destiny/test/net/server/helpers.py:150
    assert!(first.is_cloaked == second.is_cloaked);
    // original-assertion: python/destiny/test/net/server/helpers.py:151
    assert!(first.is_moribund == second.is_moribund);
    // original-assertion: python/destiny/test/net/server/helpers.py:152
    assert!(first.harmonic == second.harmonic);
    // original-assertion: python/destiny/test/net/server/helpers.py:153
    assert!(first.corporation_id == second.corporation_id);
    // original-assertion: python/destiny/test/net/server/helpers.py:154
    assert!(first.alliance_id == second.alliance_id);
    // original-assertion: python/destiny/test/net/server/helpers.py:155
    assert!(first.position.x == second.position.x);
    // original-assertion: python/destiny/test/net/server/helpers.py:156
    assert!(first.position.y == second.position.y);
    // original-assertion: python/destiny/test/net/server/helpers.py:157
    assert!(first.position.z == second.position.z);
    // original-assertion: python/destiny/test/net/server/helpers.py:158
    assert!(first.velocity.x == second.velocity.x);
    // original-assertion: python/destiny/test/net/server/helpers.py:159
    assert!(first.velocity.y == second.velocity.y);
    // original-assertion: python/destiny/test/net/server/helpers.py:160
    assert!(first.velocity.z == second.velocity.z);
    // original-assertion: python/destiny/test/net/server/helpers.py:161
    assert!(f64::from(first.agility) == f64::from(second.agility));
    // original-assertion: python/destiny/test/net/server/helpers.py:162
    assert!(f64::from(first.speed_fraction) == f64::from(second.speed_fraction));
    // original-assertion: python/destiny/test/net/server/helpers.py:163
    assert!(first.mode == second.mode);
    // original-assertion: python/destiny/test/net/server/helpers.py:164
    assert!(first.goto.x == second.goto.x);
    // original-assertion: python/destiny/test/net/server/helpers.py:165
    assert!(first.goto.y == second.goto.y);
    // original-assertion: python/destiny/test/net/server/helpers.py:166
    assert!(first.goto.z == second.goto.z);
    // original-assertion: python/destiny/test/net/server/helpers.py:167
    assert!(first.follow_id == second.follow_id);
    // original-assertion: python/destiny/test/net/server/helpers.py:168
    assert!(f64::from(first.follow_range) == f64::from(second.follow_range));
    // original-assertion: python/destiny/test/net/server/helpers.py:169
    assert!(first.owner_id == second.owner_id);
    // original-assertion: python/destiny/test/net/server/helpers.py:170
    assert!(first.effect_stamp == second.effect_stamp);
    // original-assertion: python/destiny/test/net/server/helpers.py:174
    assert!(first.formation_id == second.formation_id);
    // original-assertion: python/destiny/test/net/server/helpers.py:175
    assert!(first.mini_balls == second.mini_balls);
    // original-assertion: python/destiny/test/net/server/helpers.py:176
    assert!(first.mini_capsules == second.mini_capsules);
    // original-assertion: python/destiny/test/net/server/helpers.py:177
    assert!(first.mini_boxes == second.mini_boxes);
}

fn count_add_ball_actions(
    park: &OriginalParkProjection,
    ticker: &OriginalServerTicker,
    history: BatcherHistory,
    id: i64,
) -> usize {
    let mut count = 0;
    let mut i = 0;
    while i < history.len() {
        if let Some(event) = history.get(i) {
            if event.name == ActionName::AddBallsToPark {
                let index = match event.payload {
                    BatcherPayload::Stream { stream_index, .. } => Some(stream_index),
                    _ => None,
                };
                assert!(index.is_some()); // original action_parameters[0] must be bytes
                let bytes = ticker.batcher.stream_bytes(index.unwrap_or(usize::MAX));
                assert!(bytes.is_some());
                let mut reader = OriginalParkProjection::default();
                assert!(reader.read_original_state(&bytes.unwrap_or_default(), 0) == Ok(()));
                assert!(reader.contains(id)); // original helper assertIn
                assert!(park.contains(id)); // original dictionary index must exist
                assert_balls_equal_without_bubbles(park.retained(id), reader.retained(id));
                count += 1;
            }
        }
        i += 1;
    }
    count
}
fn assert_add_ball_cast(
    park: &OriginalParkProjection,
    ticker: &OriginalServerTicker,
    id: i64,
    clients: BoundedList<i64, 8>,
    single: bool,
) {
    let casts = if single {
        &ticker.network.singlecasts
    } else {
        &ticker.network.narrowcasts
    };
    let mut found = 0;
    let mut i = 0;
    while i < casts.len() {
        if let Some(cast) = casts.get(i) {
            let selected = if single {
                cast.clients.get(0) == clients.get(0)
            } else {
                ids_equal(&cast.clients, &clients)
            };
            if selected {
                found += count_add_ball_actions(park, ticker, cast.history, id);
            }
        }
        i += 1;
    }
    assert!(found == 1); // original assertEqual(1, found)
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_applies_action
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_applies_action() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let ball = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .go_to_point(
                    V::Integer(BALL_ID_1),
                    V::Float(10000.0_f64),
                    V::Float(0.0_f64),
                    V::Float(0.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        // original-assertion: python/destiny/test/net/server/test_ticker.py:60
        assert!(OriginalMode::Goto == park.retained(ball).mode);

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_add_ball_event_sent_to_balls_in_same_bubble_as_uncloaking_ball
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_add_ball_event_sent_to_balls_in_same_bubble_as_uncloaking_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .uncloak_ball(V::Integer(BALL_ID_1))
                .is_ok()
        );
        ticker.clear_network();
        perform_tick(&mut park, &mut ticker, true);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:237
        assert_add_ball_cast(&park, &ticker, BALL_ID_1, ids(&[CLIENT_2]), true);

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_add_balls_to_park
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_add_balls_to_park() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_2),
                    V::Float(1000000000.0_f64),
                    V::Float(0.0_f64),
                    V::Float(0.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        ticker.clear_network();
        assert!(
            ticker
                .actions
                .add_ball_to_client_parks(&mut park, BALL_ID_2)
                == Ok(())
        );
        perform_tick(&mut park, &mut ticker, true);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:299
        assert_add_ball_cast(&park, &ticker, BALL_ID_2, ids(&[CLIENT_1]), false);

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_ball_not_global_sends_remove_ball_update_to_other_bubbles
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_ball_not_global_sends_remove_ball_update_to_other_bubbles() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_3, CHAR_3, CLIENT_3);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_2),
                    V::Float(1000000000.0_f64),
                    V::Float(0.0_f64),
                    V::Float(0.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_3),
                    V::Float(1000000000.0_f64),
                    V::Float(0.0_f64),
                    V::Float(1000.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        assert!(ticker.actions.make_ball_global(&mut park, BALL_ID_2) == Ok(()));
        perform_tick(&mut park, &mut ticker, false);
        ticker.clear_network();
        assert!(ticker.actions.make_ball_local(&mut park, BALL_ID_2) == Ok(()));
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:311
        assert_update(
            &ticker,
            ids(&[CLIENT_1]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::RemoveBall,
                payload: BatcherPayload::Tuple(args(&[V::Integer(BALL_ID_2)])),
            },
            false,
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_cloak_ball_cloaks_ball
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_cloak_ball_cloaks_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let ball = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        // original-assertion: python/destiny/test/net/server/test_ticker.py:66
        assert!(1_i64 == i64::from(park.retained(ball).is_cloaked));

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_cloak_ball_emits_signal
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_cloak_ball_emits_signal() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        // original mock callback constructed with no invocations
        ticker.cloaking_connected = true;
        let _ = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        // original-assertion: python/destiny/test/net/server/test_ticker.py:74
        assert!(
            ticker.signal_calls.len() == 1
                && ticker.signal_calls.get(0) == Some(ServerTickerSignal::Cloaking(BALL_ID_1))
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_cloak_ball_event_narrowcast_to_clients_in_bubble
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_cloak_ball_event_narrowcast_to_clients_in_bubble() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        perform_tick(&mut park, &mut ticker, false);
        let uncloak_range_meters = 1000.0_f64;
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(uncloak_range_meters)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:129
        assert_update(
            &ticker,
            ids(&[CLIENT_1, CLIENT_2]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::CloakBall,
                payload: BatcherPayload::Tuple(args(&[
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(uncloak_range_meters),
                ])),
            },
            false,
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_cloaked_ball_updates_only_get_sent_to_cloaked_balls_client
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_cloaked_ball_updates_only_get_sent_to_cloaked_balls_client() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        let uncloak_range_meters = 1000.0_f64;
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(uncloak_range_meters)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        ticker.clear_network();
        let x = 1.5_f64;
        let y = 2.9_f64;
        let z = 3.0_f64;
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .go_to_point(V::Integer(BALL_ID_1), V::Float(x), V::Float(y), V::Float(z))
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion: python/destiny/test/net/server/test_ticker.py:337
        assert!(ticker.network.narrowcasts.is_empty());
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:338
        assert_update(
            &ticker,
            ids(&[CLIENT_1]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::GotoPoint,
                payload: BatcherPayload::Tuple(args(&[
                    V::Integer(BALL_ID_1),
                    V::Float(x),
                    V::Float(y),
                    V::Float(z),
                ])),
            },
            true,
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_cloaking_and_uncloaking_in_same_tick_does_not_uncloak_ball
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_cloaking_and_uncloaking_in_same_tick_does_not_uncloak_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let ball = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .uncloak_ball(V::Integer(BALL_ID_1))
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        // original-assertion: python/destiny/test/net/server/test_ticker.py:146
        assert!(i64::from(park.retained(ball).is_cloaked) != 0);

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_handle_remove_global_ball_sends_remove_ball_event_to_each_bubble_containing_interactives
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_handle_remove_global_ball_sends__0103b72d109740d37735f3cd() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_3, CHAR_3, CLIENT_3);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_2),
                    V::Float(1000000000.0_f64),
                    V::Float(0.0_f64),
                    V::Float(0.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_3),
                    V::Float(1000000000.0_f64),
                    V::Float(0.0_f64),
                    V::Float(1000.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        assert!(ticker.actions.make_ball_global(&mut park, BALL_ID_2) == Ok(()));
        perform_tick(&mut park, &mut ticker, false);
        ticker.clear_network();
        assert!(ticker.actions.remove_ball(&mut park, BALL_ID_2) == Ok(()));
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:322
        assert_update(
            &ticker,
            ids(&[CLIENT_1]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::RemoveBall,
                payload: BatcherPayload::Tuple(args(&[V::Integer(BALL_ID_2)])),
            },
            false,
        );
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:323
        assert_update(
            &ticker,
            ids(&[CLIENT_3]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::RemoveBall,
                payload: BatcherPayload::Tuple(args(&[V::Integer(BALL_ID_2)])),
            },
            false,
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_no_updates_are_sent_out_for_bubbles_not_containing_interactive_balls
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_no_updates_are_sent_out_for_bub__cef9c0947d634a94ae58fa01() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_interactive(V::Integer(BALL_ID_1), V::Boolean(false))
                .is_ok()
        );
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_interactive(V::Integer(BALL_ID_2), V::Boolean(false))
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        ticker.clear_network();
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .go_to_point(
                    V::Integer(BALL_ID_1),
                    V::Float(1000.0_f64),
                    V::Float(2000.0_f64),
                    V::Float(3000.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_speed_fraction(V::Integer(BALL_ID_2), V::Float(0.5_f64))
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion: python/destiny/test/net/server/test_ticker.py:349
        assert!(ticker.network.narrowcasts.is_empty());
        // original-assertion: python/destiny/test/net/server/test_ticker.py:350
        assert!(ticker.network.singlecasts.is_empty());

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_set_ball_position_emits_signal
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_set_ball_position_emits_signal() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        // original mock callback constructed with no invocations
        ticker.position_connected = true;
        let is_local_ball = false;
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_1),
                    V::Float(1234.5_f64),
                    V::Float(678.9_f64),
                    V::Float(987654321.0_f64),
                    V::Boolean(is_local_ball)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion: python/destiny/test/net/server/test_ticker.py:250
        assert!(
            ticker.signal_calls.len() == 1
                && ticker.signal_calls.get(0)
                    == Some(ServerTickerSignal::SetPosition(BALL_ID_1, is_local_ball))
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_set_ball_position_event_not_sent_to_bubble_for_local_balls
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_set_ball_position_event_not_sent_to_bubble_for_local_balls() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        perform_tick(&mut park, &mut ticker, false);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_1),
                    V::Float(1.5_f64),
                    V::Float(2.9_f64),
                    V::Float(3.0_f64),
                    V::Boolean(true)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion: python/destiny/test/net/server/test_ticker.py:290
        assert!(ticker.network.narrowcasts.is_empty());

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_set_ball_position_event_sent_to_bubble
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_set_ball_position_event_sent_to_bubble() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_2, CHAR_2, CLIENT_2);
        perform_tick(&mut park, &mut ticker, false);
        let x = 1.5_f64;
        let y = 2.9_f64;
        let z = 3.0_f64;
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_1),
                    V::Float(1.5_f64),
                    V::Float(2.9_f64),
                    V::Float(3.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:276
        assert_update(
            &ticker,
            ids(&[CLIENT_1, CLIENT_2]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::SetBallPosition,
                payload: BatcherPayload::Tuple(args(&[
                    V::Integer(BALL_ID_1),
                    V::Float(x),
                    V::Float(y),
                    V::Float(z),
                ])),
            },
            false,
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_set_ball_position_sets_ball_position
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_set_ball_position_sets_ball_position() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let ball = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let x = 1234.5_f64;
        let y = 678.9_f64;
        let z = 987654321.0_f64;
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .set_ball_position(
                    V::Integer(BALL_ID_1),
                    V::Float(1234.5_f64),
                    V::Float(678.9_f64),
                    V::Float(987654321.0_f64),
                    V::Boolean(false)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion: python/destiny/test/net/server/test_ticker.py:261
        assert!(
            (x, y, z)
                == (
                    park.retained(ball).position.x,
                    park.retained(ball).position.y,
                    park.retained(ball).position.z
                )
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_uncloak_ball_emits_signal
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_uncloak_ball_emits_signal() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        // original mock callback constructed with no invocations
        ticker.uncloaking_connected = true;
        let _ = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .uncloak_ball(V::Integer(BALL_ID_1))
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        // original-assertion: python/destiny/test/net/server/test_ticker.py:170
        assert!(
            ticker.signal_calls.len() == 1
                && ticker.signal_calls.get(0) == Some(ServerTickerSignal::Uncloaking(BALL_ID_1))
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_uncloak_ball_uncloaks_ball
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_uncloak_ball_uncloaks_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let ball = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .uncloak_ball(V::Integer(BALL_ID_1))
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));
        // original-assertion: python/destiny/test/net/server/test_ticker.py:155
        assert!(!(i64::from(park.retained(ball).is_cloaked) != 0));

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_uncloak_does_not_error_when_ball_missing
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_uncloak_does_not_error_when_ball_missing() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .uncloak_ball(V::Integer(BALL_ID_1))
                .is_ok()
        );
        assert!(ticker.tick(&mut park) == Ok(()));

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}

// original-case: python/destiny/test/net/server/test_ticker.py::TestTicker::test_uncloak_event_sent_to_uncloaking_ball
// original-source-sha256: e3ca9496603f83467a5fd808551dbaf959e6c10d4206a332d2e2f3ca612ad790
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(1025))]
#[cfg_attr(test, test)]
fn TestTicker__test_uncloak_event_sent_to_uncloaking_ball() {
    let mut park = OriginalParkProjection::default();
    park.is_master = true;
    {
        let mut ticker = OriginalServerTicker::default();
        let _ = set_up_ball(&mut park, &mut ticker, BALL_ID_1, CHAR_1, CLIENT_1);
        let _ = add_ball(&mut park, BALL_ID_1);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .cloak_ball(
                    V::Integer(BALL_ID_1),
                    V::Integer(1_i64),
                    V::Float(1000.0_f64)
                )
                .is_ok()
        );
        perform_tick(&mut park, &mut ticker, false);
        assert!(ticker.refresh_action_balls(&park) == Ok(()));
        assert!(
            ticker
                .actions
                .queue
                .uncloak_ball(V::Integer(BALL_ID_1))
                .is_ok()
        );
        ticker.clear_network();
        perform_tick(&mut park, &mut ticker, false);
        // original-assertion helper call: python/destiny/test/net/server/test_ticker.py:192
        assert_update(
            &ticker,
            ids(&[CLIENT_1]),
            BatcherEvent {
                timestamp: ticker.stamp_for_system - 1,
                name: ActionName::UncloakBall,
                payload: BatcherPayload::Tuple(args(&[V::Integer(BALL_ID_1)])),
            },
            true,
        );

        assert!(!park.scope_error);
    } // original ticker/batcher/updater/actions reference deletion order
    // Native park destruction and weak-reference release remain pending.
    #[cfg(kani)]
    kani::cover!(
        true,
        "complete original server ticker fixture and assertions reached"
    );
}
