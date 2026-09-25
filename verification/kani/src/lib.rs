#![forbid(unsafe_code)]

mod ball_contracts;
mod contracts;
mod lifecycle_contracts;
mod metaverification;
mod state_contracts;
mod visibility_contracts;

use destiny_verification_macros::{DestinyModel, destiny_contract, destiny_delegate, destiny_spec};

destiny_contract!(
    ADD_DURING_EVOLVE,
    "python/destiny/test/ballpark/evolve/test_add.py",
    "test_balls_can_not_be_added_during_evolve"
);
destiny_contract!(
    NEGATIVE_MASS_SETTER,
    "python/destiny/test/ballpark/test_getters_and_setters.py",
    "test_setting_ball_mass_to_a_negative_value_is_ineffective"
);
destiny_contract!(
    SPEED_FRACTION_UPPER,
    "python/destiny/test/ballpark/test_getters_and_setters.py",
    "test_setting_speed_fraction_clamps_upper_bound_at_one"
);
destiny_contract!(
    SPEED_FRACTION_LOWER,
    "python/destiny/test/ballpark/test_getters_and_setters.py",
    "test_setting_speed_fraction_clamps_lowe_bound_at_zero"
);

#[derive(DestinyModel)]
struct SetterProbe {
    previous: f64,
}

#[destiny_spec(
    source = "python/destiny/test/ballpark/test_getters_and_setters.py",
    test = "test_setting_ball_mass_to_a_negative_value_is_ineffective"
)]
fn model_non_negative_setter(previous: f64, requested: f64) -> f64 {
    destiny_original_spec::apply_non_negative_setter(previous, requested)
}

#[destiny_delegate(target = "destiny_original_spec::clamp_speed_fraction")]
fn model_speed_fraction(requested: f64) -> f64 {
    destiny_original_spec::clamp_speed_fraction(requested)
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn proc_macro_metadata_and_derive_are_usable_in_normal_rust() {
        assert_eq!(ADD_DURING_EVOLVE.1, "test_balls_can_not_be_added_during_evolve");
        assert_eq!(SetterProbe::DESTINY_FORMAL_MODEL, "SetterProbe");
        let probe = SetterProbe { previous: 3.0 };
        assert_eq!(model_non_negative_setter(probe.previous, -1.0), 3.0);
        assert_eq!(model_speed_fraction(2.0), 1.0);
    }
}
