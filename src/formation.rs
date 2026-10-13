//! Engine-independent production formation reservation kernels.
//!
//! The specification retains the original first-free scan. Production uses a
//! bit mask, so a refinement comparison exercises independent implementations.
//! The callers resolve attachment and formation identity to `slot_count`.

// A total, literal representation of the original std::bitset<16> positions.
// Avoid importing panic-only shift paths from the verifier's standard library.
// Out-of-capacity inputs retain the explicit unchanged-state result below.
fn slot_bit(index: u32) -> Option<u16> {
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

fn slot_mask(count: usize) -> Option<u16> {
    match count {
        0 => Some(0x0000),
        1 => Some(0x0001),
        2 => Some(0x0003),
        3 => Some(0x0007),
        4 => Some(0x000f),
        5 => Some(0x001f),
        6 => Some(0x003f),
        7 => Some(0x007f),
        8 => Some(0x00ff),
        9 => Some(0x01ff),
        10 => Some(0x03ff),
        11 => Some(0x07ff),
        12 => Some(0x0fff),
        13 => Some(0x1fff),
        14 => Some(0x3fff),
        15 => Some(0x7fff),
        16 => Some(0xffff),
        _ => None,
    }
}

#[must_use]
pub(crate) fn reserve_formation_slot(reserved: u16, slot_count: Option<usize>) -> (u16, i8) {
    let result = match slot_count {
        Some(count @ 0..=16) => {
            let allowed = match slot_mask(count) {
                Some(mask) => mask,
                None => return (reserved, -1),
            };
            let available = !reserved & allowed;
            if available == 0 {
                (reserved, -1)
            } else {
                let slot = available.trailing_zeros();
                match slot_bit(slot) {
                    Some(bit) => (reserved | bit, slot as i8),
                    None => (reserved, -1),
                }
            }
        }
        _ => (reserved, -1),
    };
    #[cfg(kani)]
    kani::cover!(true, "production formation reservation executed");
    result
}

#[must_use]
pub(crate) fn free_formation_slot(reserved: u16, slot_count: Option<usize>, slot: i64) -> u16 {
    let result = match (slot_count, u32::try_from(slot)) {
        (Some(count), Ok(index)) if index < 16 && (index as usize) < count => {
            match slot_bit(index) {
                Some(bit) => reserved & !bit,
                None => reserved,
            }
        }
        _ => reserved,
    };
    #[cfg(kani)]
    kani::cover!(true, "production formation release executed");
    result
}
