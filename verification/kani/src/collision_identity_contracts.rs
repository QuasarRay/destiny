//! Full i64/boolean domain of the actual source ID policy, including checked
//! exhaustion. This is a counter kernel, not allocation-map or FFI refinement.
#[path = "../../../src/collision_identity.rs"]
mod production;

#[kani::proof]
fn source_collision_identity_policy_is_total_for_both_counter_partitions() {
    let master: bool = kani::any();
    let requested: i64 = kani::any();
    let local: i64 = kani::any();
    let high: i64 = kani::any();
    let result = production::allocate_collision_id(master, requested, local, high);
    let high_partition = if master {
        requested < 0
    } else {
        requested == -1
    };
    let local_partition = master && requested == 0;
    if high_partition {
        if high == i64::MIN {
            assert_eq!(result, None);
        } else {
            assert_eq!(result, Some((high - 1, local, high - 1)));
        }
    } else if local_partition {
        if local == i64::MIN {
            assert_eq!(result, None);
        } else {
            assert_eq!(result, Some((local - 1, local - 1, high)));
        }
    } else {
        assert_eq!(result, Some((requested, local, high)));
    }
    kani::cover!(
        master && requested < 0 && high > i64::MIN && result.is_some(),
        "server high counter domain"
    );
    kani::cover!(
        !master && requested == -1 && high > i64::MIN && result.is_some(),
        "client high counter domain"
    );
    kani::cover!(
        local_partition && local > i64::MIN && result.is_some(),
        "server zero local counter domain"
    );
    kani::cover!(
        high_partition && high == i64::MIN && result.is_none(),
        "high counter exhaustion rejected"
    );
    kani::cover!(
        local_partition && local == i64::MIN && result.is_none(),
        "local counter exhaustion rejected"
    );
    kani::cover!(
        !high_partition && !local_partition && result.is_some(),
        "unchanged explicit identity domain"
    );
}
