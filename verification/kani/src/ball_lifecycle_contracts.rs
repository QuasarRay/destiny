//! Actual retained-object state kernel. Construction, allocation/map identity,
//! Python leases, snapshot adapters, Mutex and callback cleanup remain callers.
#[path = "../../../src/ball_lifecycle.rs"]
mod production;

#[cfg(kani)]
#[kani::proof]
fn total_remote_follower_selection() {
    let free: bool = kani::any();
    let mode: u8 = kani::any();
    let exists: bool = kani::any();
    let same: bool = kani::any();
    let admitted = production::remote_follower(free, mode, exists, same);
    assert_eq!(
        admitted,
        free && (mode == 1 || mode == 4 || mode == 5 || mode == 12) && (!exists || !same)
    );
    kani::cover!(admitted && mode == 1 && exists, "remote Follow admitted");
    kani::cover!(admitted && mode == 4, "remote Orbit admitted");
    kani::cover!(admitted && mode == 5, "remote Missile admitted");
    kani::cover!(admitted && mode == 12, "remote Formation admitted");
    kani::cover!(
        admitted && !exists,
        "inconsistent missing target included by source"
    );
    kani::cover!(
        free && exists && same && !admitted,
        "same-bubble follower excluded"
    );
    kani::cover!(!free && !admitted, "nonfree ball excluded");
}

// Derived StopAllFollowers decision contract, including the original removal
// case's Follow branch. Collection/callback correspondence remains separate.
// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestFollow::test_removing_ball_stops_followers_from_following_that_ball
// original-case: python/destiny/test/ballpark/test_movement_controls.py::TestMissileFollow::test_removing_ball_stops_followers_from_following_that_ball
#[cfg(kani)]
#[kani::proof]
fn total_follower_removal_action() {
    let live: bool = kani::any();
    let follows: bool = kani::any();
    let mode: u8 = kani::any();
    let interactive: bool = kani::any();
    let action = production::follower_action(live, follows, mode, interactive);
    assert_eq!(action == 0, !live || !follows);
    assert_eq!(
        action == 1,
        live && follows && mode != 5 && (mode != 4 || !interactive)
    );
    assert_eq!(action == 2, live && follows && mode == 4 && interactive);
    assert_eq!(action == 3, live && follows && mode == 5);
    kani::cover!(!live && follows, "removed follower rejected");
    kani::cover!(live && !follows, "reentrant retargeted follower preserved");
    kani::cover!(
        live && follows && mode == 1,
        "original Follow removal branch"
    );
    kani::cover!(action == 1 && mode == 4, "noninteractive Orbit stopped");
    kani::cover!(action == 2, "interactive Orbit switches to Goto");
    kani::cover!(action == 3, "Missile loses mass and switches to Goto");
}

#[cfg(kani)]
#[kani::proof]
fn total_retained_object_transition() {
    let attached: bool = kani::any();
    let moribund: bool = kani::any();
    let references: usize = kani::any();
    let event: u8 = kani::any();
    let result = production::transition(attached, moribund, references, event);
    let allowed = match event {
        0 => (attached || references > 0) && references < usize::MAX,
        1 => references > 0,
        2..=5 => attached,
        _ => false,
    };
    assert_eq!(result.is_some(), allowed);
    if let Some((next_attached, next_moribund, next_references)) = result {
        assert_eq!(next_attached, attached && event != 2 && event != 3);
        assert_eq!(
            next_moribund,
            event == 2 || event == 5 || (event != 4 && moribund)
        );
        match event {
            0 => assert_eq!(next_references as u128, references as u128 + 1),
            1 => assert_eq!(next_references as u128 + 1, references as u128),
            _ => assert_eq!(next_references, references),
        }
    }
    kani::cover!(
        event == 0 && result.is_some() && !attached,
        "retained detached object copied"
    );
    kani::cover!(
        event == 0 && result.is_none() && references == usize::MAX,
        "lease exhaustion rejected"
    );
    kani::cover!(
        event == 0 && result.is_none() && !attached && references == 0,
        "dead object cannot resurrect"
    );
    kani::cover!(
        event == 1 && references == 1 && !attached && result == Some((false, moribund, 0)),
        "last detached lease released"
    );
    kani::cover!(
        event == 1 && references == 0 && result.is_none(),
        "empty release rejected"
    );
    kani::cover!(
        event == 2 && result == Some((false, true, references)),
        "immediate removal retains moribund state"
    );
    kani::cover!(
        event == 3 && !moribund && result == Some((false, false, references)),
        "clear preserves nonmoribund state"
    );
    kani::cover!(
        event == 4 && moribund && result == Some((true, false, references)),
        "attached repair clears moribund state"
    );
    kani::cover!(
        event == 5 && result == Some((true, true, references)),
        "delayed removal retains membership"
    );
    kani::cover!(event > 5 && result.is_none(), "unknown transition rejected");
}

#[test]
fn lease_survives_removal_and_clear_does_not_imply_moribund() {
    let first = production::transition(true, false, 0, 0).unwrap();
    let alias = production::transition(first.0, first.1, first.2, 0).unwrap();
    let removed = production::transition(alias.0, alias.1, alias.2, 2).unwrap();
    assert_eq!(removed, (false, true, 2));
    let copied = production::transition(removed.0, removed.1, removed.2, 0).unwrap();
    assert_eq!(copied, (false, true, 3));
    assert_eq!(
        production::transition(true, false, 1, 3),
        Some((false, false, 1))
    );
    assert_eq!(production::transition(false, true, 0, 0), None);
}
