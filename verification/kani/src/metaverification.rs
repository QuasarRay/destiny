use destiny_metaverification::{delegate_unary, kani_regression};

fn handwritten_non_negative_setter(previous: f64, requested: f64) -> f64 {
    if requested >= 0.0 {
        requested
    } else {
        previous
    }
}

fn reusable_non_negative_setter(previous: f64, requested: f64) -> f64 {
    destiny_original_spec::apply_non_negative_setter(previous, requested)
}

fn handwritten_speed_fraction(requested: f64) -> f64 {
    if requested < 0.0 {
        0.0
    } else if requested > 1.0 {
        1.0
    } else {
        requested
    }
}

delegate_unary!(
    pub(crate) fn delegated_speed_fraction(requested: f64) -> f64
        => destiny_original_spec::clamp_speed_fraction
);

fn handwritten_saturating_increment(value: u64) -> u64 {
    value.saturating_add(1)
}

fn reusable_saturating_increment(value: u64) -> u64 {
    value.saturating_add(1)
}

kani_regression!(reusable_non_negative_setter_matches_handwritten_pattern, {
    let previous: f64 = kani::any();
    let requested: f64 = kani::any();

    let handwritten = handwritten_non_negative_setter(previous, requested);
    let reusable = reusable_non_negative_setter(previous, requested);

    assert_eq!(handwritten.to_bits(), reusable.to_bits());
});

kani_regression!(delegated_speed_fraction_matches_handwritten_pattern, {
    let requested: f64 = kani::any();

    let handwritten = handwritten_speed_fraction(requested);
    let delegated = delegated_speed_fraction(requested);

    assert_eq!(handwritten.to_bits(), delegated.to_bits());
});

kani_regression!(saturating_counter_helper_matches_handwritten_pattern, {
    let value: u64 = kani::any();
    assert_eq!(
        handwritten_saturating_increment(value),
        reusable_saturating_increment(value)
    );
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reusable_setter_matches_representative_values() {
        for (previous, requested) in [(3.0, -1.0), (3.0, 0.0), (3.0, 7.0)] {
            assert_eq!(
                handwritten_non_negative_setter(previous, requested).to_bits(),
                reusable_non_negative_setter(previous, requested).to_bits(),
            );
        }
    }

    #[test]
    fn delegation_macro_preserves_representative_clamp_values() {
        for value in [-1.0, 0.0, 0.25, 1.0, 2.0] {
            assert_eq!(
                handwritten_speed_fraction(value).to_bits(),
                delegated_speed_fraction(value).to_bits(),
            );
        }
    }
}
