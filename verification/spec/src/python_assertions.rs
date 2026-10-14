//! The `unittest.TestCase.assertAlmostEqual` decimal-places predicate.
//!
//! The original tests first accept equal values, otherwise require
//! `round(abs(first - second), places) == 0`. This is a decimal rounding cell,
//! not `abs(first-second) <= 10**(-places)`. For nonnegative binary64 d and
//! 1..=18 places, its exact boundary is the non-binary rational
//! 1/(2*10**places); equality with that boundary is impossible. At zero places
//! the exactly representable half is accepted by ties-to-even rounding.
//!
//! IEEE subtraction is still performed before the exact integer comparison.
//! None represents an unsupported decimal-place count, never assertion success.

/// `unittest.assertAlmostEqual(..., delta=...)`: exact equality is accepted
/// first; otherwise compare the actual binary64 absolute difference to delta.
/// This retains NaN/infinity behavior and does not replace delta by places.
#[must_use]
pub fn python_almost_equal_delta(first: f64, second: f64, delta: f64) -> bool {
    first == second || (first - second).abs() <= delta
}

/// Three exact float objects summed by CPython 3.12.9's builtin sum.
/// The initial integer zero is added to the first float before the compensated
/// float fast path processes the next two items. The compensation is only
/// added when nonzero and finite, as in Python/bltinmodule.c:2393..2477.
/// https://github.com/python/cpython/blob/v3.12.9/Python/bltinmodule.c
/// Interpreter/build correspondence remains a boundary requirement.
#[must_use]
pub fn python_sum_three_floats(values: [f64; 3]) -> f64 {
    let mut total = 0.0 + values[0];
    let mut correction = 0.0;
    let mut index = 1;
    while index < 3 {
        let value = values[index];
        let next = total + value;
        correction += if total.abs() >= value.abs() {
            (total - next) + value
        } else {
            (value - next) + total
        };
        total = next;
        index += 1;
    }
    if correction != 0.0 && correction.is_finite() {
        total += correction;
    }
    total
}

#[cfg(test)]
mod reduction_regressions {
    use super::{python_almost_equal_delta, python_sum_three_floats};
    #[test]
    fn compensation_keeps_a_small_term_after_cancellation() {
        assert_eq!(python_sum_three_floats([1e16, 1.0, -1e16]), 1.0);
        assert_eq!(
            python_sum_three_floats([f64::INFINITY, 1.0, 2.0]),
            f64::INFINITY
        );
        assert!(python_sum_three_floats([f64::INFINITY, f64::NEG_INFINITY, 1.0]).is_nan());
    }
    #[test]
    fn delta_preserves_equality_first_and_its_inclusive_boundary() {
        assert!(python_almost_equal_delta(f64::INFINITY, f64::INFINITY, 0.0));
        assert!(python_almost_equal_delta(0.0, 20.0, 20.0));
        assert!(!python_almost_equal_delta(
            0.0,
            f64::from_bits(20.0_f64.to_bits() + 1),
            20.0
        ));
        assert!(!python_almost_equal_delta(f64::NAN, f64::NAN, 20.0));
    }
}

#[must_use]
pub fn python_almost_equal(first: f64, second: f64, places: u32) -> Option<bool> {
    let scale: u128 = match places {
        0 => 1,
        1 => 10,
        2 => 100,
        3 => 1_000,
        4 => 10_000,
        5 => 100_000,
        6 => 1_000_000,
        7 => 10_000_000,
        8 => 100_000_000,
        9 => 1_000_000_000,
        10 => 10_000_000_000,
        11 => 100_000_000_000,
        12 => 1_000_000_000_000,
        13 => 10_000_000_000_000,
        14 => 100_000_000_000_000,
        15 => 1_000_000_000_000_000,
        16 => 10_000_000_000_000_000,
        17 => 100_000_000_000_000_000,
        18 => 1_000_000_000_000_000_000,
        _ => return None,
    };
    if first == second {
        return Some(true);
    }
    if !first.is_finite() || !second.is_finite() {
        return Some(false);
    }
    let difference = (first - second).abs();
    if !difference.is_finite() {
        return Some(false);
    }
    if difference == 0.0 {
        return Some(true);
    }
    let bits = difference.to_bits();
    let raw_exponent = (bits >> 52) & 0x7ff;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let (mantissa, denominator_power) = if raw_exponent == 0 {
        (u128::from(fraction), 1_074_u64)
    } else if raw_exponent >= 1_075 {
        // d >= 1, beyond every supported cell around zero.
        return Some(false);
    } else {
        (
            u128::from(fraction | 0x0010_0000_0000_0000),
            1_075_u64.saturating_sub(raw_exponent),
        )
    };
    // The numerator is at most (2^53-1)*10^18*2 < 2^114.
    let numerator = mantissa.saturating_mul(scale).saturating_mul(2);
    if denominator_power >= 128 {
        return Some(true);
    }
    let denominator = 1_u128.wrapping_shl(denominator_power as u32);
    Some(if places == 0 {
        numerator <= denominator
    } else {
        numerator < denominator
    })
}

/// An exact decimal rounding-cell key for CPython's correctly rounded
/// binary64-to-decimal `round(value, places)` path.
///
/// This partition supports finite |value| <= 2^26 and 0..=6 places. Within
/// this partition, adjacent decimal results remain distinct after conversion
/// back to binary64: their separation is at least 10^-6, whereas a binary64
/// rounding cell here is at most 2^-26 wide. Comparing the signed decimal
/// integers therefore compares the actual Python numeric results, including
/// equality of positive and negative zero. No floating multiplication by a
/// decimal scale is used; that would introduce a second rounding at ties.
///
/// The original point-list helper defaults to four places. Other magnitudes,
/// place counts and Python fallback conversion implementations require their
/// own correspondence obligation; None never satisfies an original assertion.
#[must_use]
pub fn python_decimal_round_key(value: f64, places: u32) -> Option<i128> {
    let five_power = match places {
        0 => 1_u128,
        1 => 5,
        2 => 25,
        3 => 125,
        4 => 625,
        5 => 3_125,
        6 => 15_625,
        _ => return None,
    };
    if !value.is_finite() || value.abs() > 67_108_864.0 {
        return None;
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as u32;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let (mantissa, denominator_power) = if exponent == 0 {
        (u128::from(fraction), 1_074_u32.checked_sub(places)?)
    } else {
        (
            u128::from(fraction | 0x0010_0000_0000_0000),
            1_075_u32.checked_sub(exponent)?.checked_sub(places)?,
        )
    };
    // At most (2^53-1)*5^6 < 2^67. A denominator >= 2^128
    // therefore puts this value strictly inside the zero rounding cell.
    let numerator = mantissa.checked_mul(five_power)?;
    if denominator_power >= 128 {
        return Some(0);
    }
    let denominator = 1_u128.checked_shl(denominator_power)?;
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let half = denominator >> 1;
    let increment = remainder > half || (remainder == half && quotient & 1 == 1);
    let magnitude = quotient.checked_add(u128::from(increment))?;
    let signed = i128::try_from(magnitude).ok()?;
    Some(if bits >> 63 == 0 { signed } else { -signed })
}

#[must_use]
pub fn python_rounded_equal(first: f64, second: f64, places: u32) -> Option<bool> {
    Some(python_decimal_round_key(first, places)? == python_decimal_round_key(second, places)?)
}

#[cfg(test)]
mod tests {
    use super::{python_almost_equal, python_decimal_round_key, python_rounded_equal};

    #[test]
    fn decimal_rounding_preserves_binary_ties_and_separate_cells() {
        assert_eq!(python_decimal_round_key(2.675, 2), Some(267));
        assert_eq!(python_decimal_round_key(2.625, 2), Some(262));
        assert_eq!(python_decimal_round_key(-2.625, 2), Some(-262));
        assert_eq!(python_decimal_round_key(0.5, 0), Some(0));
        assert_eq!(python_decimal_round_key(1.5, 0), Some(2));
        assert_eq!(python_decimal_round_key(-1.5, 0), Some(-2));
        assert_eq!(python_decimal_round_key(f64::from_bits(1), 6), Some(0));
        assert_eq!(python_rounded_equal(-0.0, 0.0, 4), Some(true));
        // A tiny difference can straddle a decimal boundary. Difference
        // tolerance must not replace the point helper's rounding comparison.
        assert_eq!(
            python_rounded_equal(1.000049999, 1.000050001, 4),
            Some(false)
        );
        assert_eq!(
            python_decimal_round_key(67_108_864.0, 6),
            Some(67_108_864_000_000)
        );
        assert_eq!(python_decimal_round_key(67_108_865.0, 4), None);
        assert_eq!(python_decimal_round_key(0.0, 7), None);
        assert_eq!(python_decimal_round_key(f64::NAN, 4), None);
    }

    #[test]
    fn original_decimal_cell_and_ieee_edges() {
        assert_eq!(python_almost_equal(0.5, 0.0, 0), Some(true));
        assert_eq!(python_almost_equal(0.5000000000000001, 0.0, 0), Some(false));
        assert_eq!(python_almost_equal(0.000000049999999, 0.0, 7), Some(true));
        assert_eq!(python_almost_equal(0.000000050000001, 0.0, 7), Some(false));
        assert_eq!(python_almost_equal(f64::from_bits(1), 0.0, 18), Some(true));
        assert_eq!(python_almost_equal(-0.0, 0.0, 7), Some(true));
        assert_eq!(
            python_almost_equal(f64::INFINITY, f64::INFINITY, 7),
            Some(true)
        );
        assert_eq!(python_almost_equal(f64::NAN, f64::NAN, 7), Some(false));
        assert_eq!(python_almost_equal(f64::INFINITY, 0.0, 7), Some(false));
        assert_eq!(python_almost_equal(f64::MAX, -f64::MAX, 7), Some(false));
        assert_eq!(python_almost_equal(0.0, 0.0, 19), None);
    }
}
