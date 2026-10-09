//! Observable formation-slot transitions from `src/Ball.cpp` and
//! `src/Ballpark.cpp` in original Destiny. The runtime delegates to these
//! functions; neither the state nor the invariants depend on Bevy.

use crate::MAX_ORIGINAL_FORMATION_SLOTS;

pub const NO_FORMATION: u8 = 255;

/// Original first-free scan over the ball's 16-bit reservation set. `None`
/// represents an unassigned, detached, or invalid formation.
#[must_use]
pub fn reserve_formation_slot(reserved: u16, slot_count: Option<usize>) -> (u16, i8) {
    let Some(count) = slot_count else {
        return (reserved, -1);
    };
    if count > usize::from(MAX_ORIGINAL_FORMATION_SLOTS) {
        return (reserved, -1);
    }
    for slot in 0..count {
        let bit = 1_u16 << slot;
        if reserved & bit == 0 {
            return (reserved | bit, slot as i8);
        }
    }
    (reserved, -1)
}

/// Free only the requested valid slot. Negative/out-of-range requests are
/// no-ops. Indices beyond the original bitset capacity are also no-ops rather
/// than reproducing the C++ `bitset::reset` exception.
#[must_use]
pub fn free_formation_slot(reserved: u16, slot_count: Option<usize>, slot: i64) -> u16 {
    let Some(count) = slot_count else {
        return reserved;
    };
    if slot < 0 || slot >= i64::from(MAX_ORIGINAL_FORMATION_SLOTS) || slot as usize >= count {
        return reserved;
    }
    reserved & !(1_u16 << slot)
}

/// Assignment changes the ID while retaining occupied bits, as the original
/// implementation does. Clearing an assigned formation resets its bitset.
/// Movement/follower transitions are outside this slot-allocation contract.
#[must_use]
pub fn assign_formation(
    current: u8,
    reserved: u16,
    requested: i64,
    formation_count: usize,
) -> (u8, u16) {
    if requested == -1 || requested == i64::from(NO_FORMATION) {
        return (
            NO_FORMATION,
            if current == NO_FORMATION { reserved } else { 0 },
        );
    }
    if !(0..=127).contains(&requested) || requested as usize >= formation_count {
        return (current, reserved);
    }
    (requested as u8, reserved)
}
