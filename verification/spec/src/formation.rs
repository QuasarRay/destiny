//! Observable formation-slot transitions from `src/Ball.cpp` and
//! `src/Ballpark.cpp` in original Destiny. This independent scan is compared
//! with the production bit-selection kernel; neither depends on Bevy.

use crate::MAX_ORIGINAL_FORMATION_SLOTS;

pub const NO_FORMATION: u8 = 255;

fn original_bit(index: usize) -> Option<u16> {
    match index {
        0 => Some(0x0001),
        1 => Some(0x0002),
        2 => Some(0x0004),
        3 => Some(0x0008),
        4 => Some(0x0010),
        5 => Some(0x0020),
        6 => Some(0x0040),
        7 => Some(0x0080),
        8 => Some(0x0100),
        9 => Some(0x0200),
        10 => Some(0x0400),
        11 => Some(0x0800),
        12 => Some(0x1000),
        13 => Some(0x2000),
        14 => Some(0x4000),
        15 => Some(0x8000),
        _ => None,
    }
}

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
    let mut slot = 0_usize;
    while slot < count {
        // Every bit position is explicit; there is no wrapping or unchecked
        // arithmetic. The scan still follows the original operation order.
        let Some(bit) = original_bit(slot) else {
            return (reserved, -1);
        };
        if reserved & bit == 0 {
            return (reserved | bit, slot as i8);
        }
        // slot is below sixteen here. The saturating operation is total and
        // agrees with the original increment throughout the guarded domain.
        slot = slot.saturating_add(1);
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
    match original_bit(slot as usize) {
        Some(bit) => reserved & !bit,
        None => reserved,
    }
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
