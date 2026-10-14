//! Actual source spherical initialization and insertion-ordered update kernel.
//! Component/default/input/output adapters, callbacks and every omitted spatial
//! configuration remain separate callers; no whole-operation credit is claimed.
use destiny_original_spec::Vec3;
#[path = "../../../src/bubbles.rs"]
mod production;

// original-composed-regression: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_addition
// Complete composition timed out at120s, receipt a029b1be44264247afb41d0454b46671.
// The two following proof obligations preserve its initialization/update union;
// the actual component-to-dictionary adapter is still separately required.
#[test]
fn source_single_interactive_addition_all_four_dictionaries() {
    let (mut tracking, labels) = production::initialize(&[(
        1,
        Vec3 {
            x: 1e10,
            y: 1e10,
            z: 1e10,
        },
        0.0,
        0.0,
    )])
    .unwrap();
    assert_eq!(labels.len(), 1);
    let mut balls = [production::Ball {
        id: 1,
        old: -1,
        new: labels[0].1,
        interactive: true,
        global: false,
        cloaked: false,
        troll: false,
    }];
    tracking.transition(balls[0]);
    let result = tracking.updates(&mut balls, &[Some(1)]);
    use production::Delta::Balls;
    assert_eq!(result.dictionaries[0], vec![(1, Balls(vec![1]))]);
    assert_eq!(result.dictionaries[1], vec![(1, Balls(vec![]))]);
    assert_eq!(result.dictionaries[2], vec![(balls[0].new, Balls(vec![1]))]);
    assert_eq!(result.dictionaries[3], vec![(balls[0].new, Balls(vec![]))]);
    assert_eq!(result.invalid_user_index, None);
    assert_eq!(result.transitions, vec![(1, -1, balls[0].new)]);
    assert_eq!(balls[0].old, balls[0].new);
    #[cfg(kani)]
    kani::cover!(
        true,
        "original source addition dictionaries and ordered transition reached"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_addition
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(not(kani), test)]
fn source_single_addition_partition_initialization() {
    let (tracking, labels) = production::initialize(&[(
        1,
        Vec3 {
            x: 1e10,
            y: 1e10,
            z: 1e10,
        },
        0.0,
        0.0,
    )])
    .unwrap();
    assert_eq!(labels, vec![(1, 0)]);
    assert_eq!(tracking.cells.len(), 1);
    assert_eq!(
        tracking.bubble_at(Vec3 {
            x: 1e10,
            y: 1e10,
            z: 1e10
        }),
        Ok(Some(0))
    );
    #[cfg(kani)]
    kani::cover!(
        true,
        "original one-body partition constructor and label relation reached"
    );
}

// original-case: python/destiny/test/ballpark/test_lifecycle_management.py::TestAdditionsAndDeletions::test_addition
#[cfg_attr(kani, kani::proof)]
#[cfg_attr(kani, kani::unwind(65))]
#[cfg_attr(not(kani), test)]
fn source_single_addition_update_postconditions() {
    // Initializer obligation establishes label0. These are the exact original
    // fields transferred by the production initialization component adapter.
    let mut tracking = production::Tracking::default();
    let mut balls = [production::Ball {
        id: 1,
        old: -1,
        new: 0,
        interactive: true,
        global: false,
        cloaked: false,
        troll: false,
    }];
    tracking.transition(balls[0]);
    let result = tracking.updates(&mut balls, &[Some(1)]);
    use production::Delta::Balls;
    assert_eq!(result.dictionaries[0], vec![(1, Balls(vec![1]))]);
    assert_eq!(result.dictionaries[1], vec![(1, Balls(vec![]))]);
    assert_eq!(result.dictionaries[2], vec![(0, Balls(vec![1]))]);
    assert_eq!(result.dictionaries[3], vec![(0, Balls(vec![]))]);
    assert_eq!(result.invalid_user_index, None);
    assert_eq!(result.transitions, vec![(1, -1, 0)]);
    assert_eq!(balls[0].old, 0);
    #[cfg(kani)]
    kani::cover!(
        true,
        "all original dictionary and source update frame postconditions reached"
    );
}

#[test]
fn aliased_dictionaries_and_repeated_users_observe_the_same_mutable_map() {
    let (mut tracking, labels) = production::initialize(&[(0, Vec3::ZERO, 0.0, 0.0)]).unwrap();
    let mut balls = [production::Ball {
        id: 0,
        old: -1,
        new: labels[0].1,
        interactive: false,
        global: false,
        cloaked: false,
        troll: false,
    }];
    tracking.transition(balls[0]);
    let result = tracking
        .updates_with_aliases(&mut balls, &[Some(0), Some(0)], [0, 0, 0, 0])
        .unwrap();
    for dictionary in result.dictionaries {
        assert_eq!(dictionary, vec![(0, production::Delta::Bubble(0))]);
    }
    assert_eq!(result.transitions, vec![(0, -1, 0)]);
    assert_eq!(result.invalid_user_index, None);
}

#[test]
fn source_partition_native_collections_have_no_projection_capacity_limit() {
    let spheres = (1..=100)
        .map(|id| {
            (
                id,
                Vec3 {
                    x: 1e10,
                    y: 1e10,
                    z: 1e10,
                },
                0.0,
                0.0,
            )
        })
        .collect::<Vec<_>>();
    let (tracking, labels) = production::initialize(&spheres).unwrap();
    assert_eq!(labels.len(), 100);
    assert!(labels.iter().all(|entry| entry.1 == 0));
    assert_eq!(tracking.cells.len(), 1);
}
