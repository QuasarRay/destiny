#![forbid(unsafe_code)]

#[cfg(kani)]
mod collision_identity_contracts;

#[cfg(any(kani, test))]
#[path = "../../../src/iterative_collision.rs"]
mod iterative_collision;
#[cfg(any(kani, test))]
#[path = "../../../src/noniterative_collision.rs"]
mod noniterative_collision;
#[cfg(any(kani, test))]
mod noniterative_production_contracts;

#[cfg(any(kani, test))]
mod add_callback_contracts;
#[cfg(kani)]
mod ball_contracts;
#[cfg(any(kani, test))]
mod batcher_delivery_contracts;
#[cfg(any(kani, test))]
mod batcher_stream_contracts;
#[cfg(kani)]
mod bounded_list_contracts;
#[cfg(any(kani, test))]
mod box_contracts;
#[cfg(any(kani, test))]
mod bubble_updater_contracts;
#[cfg(any(kani, test))]
mod client_history_contracts;
#[cfg(any(kani, test))]
mod client_ticker_contracts;
#[cfg(kani)]
mod contracts;
#[cfg(kani)]
mod evolution_phase_contracts;
#[cfg(kani)]
mod ffi_lifecycle_contracts;
#[cfg(any(kani, test))]
mod formation_contracts;
#[cfg(any(kani, test))]
mod formation_refinement;
#[cfg(kani)]
mod proximity_kernel_contracts;
// Compile the actual kernel used by runtime.rs, without engine dependencies.
mod accuracy_production_contracts;
mod ball_default_production_contracts;
mod ball_lifecycle_contracts;
mod bubble_production_contracts;
mod callback_continuation_contracts;
#[cfg(any(kani, test))]
mod collision_contracts;
mod formation_drop_contracts;
#[cfg(any(kani, test))]
mod geometry_contracts;
mod harmonic_production_contracts;
#[cfg(any(kani, test))]
mod iterative_collision_contracts;
#[cfg(kani)]
mod lifecycle_contracts;
#[cfg(any(kani, test))]
mod lifetime_contracts;
#[cfg(any(kani, test))]
mod metaverification;
mod missile_production_contracts;
mod mushroom_production_contracts;
#[cfg(any(kani, test))]
mod native_object_contracts;
#[cfg(any(kani, test))]
mod orbit_contracts;
#[cfg(any(kani, test))]
mod park_contracts;
#[cfg(any(kani, test))]
mod park_evolution_contracts;
#[path = "../../../src/formation.rs"]
#[cfg(any(kani, test))]
mod production_formation;
#[cfg(any(kani, test))]
mod server_actions_contracts;
#[cfg(any(kani, test))]
mod server_park_action_contracts;
#[cfg(any(kani, test))]
mod server_ticker_contracts;
#[cfg(any(kani, test))]
mod settings_contracts;
#[cfg(kani)]
mod state_contracts;
#[cfg(any(kani, test))]
mod stream_contracts;
#[cfg(any(kani, test))]
mod update_batcher_contracts;
#[cfg(kani)]
mod visibility_contracts;
mod warp_entry_contracts;

use destiny_verification_macros::destiny_contract;
#[cfg(any(kani, test))]
use destiny_verification_macros::{DestinyModel, destiny_delegate, destiny_spec};

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
#[cfg(any(kani, test))]
struct SetterProbe {
    previous: f64,
}

#[destiny_spec(
    source = "python/destiny/test/ballpark/test_getters_and_setters.py",
    test = "test_setting_ball_mass_to_a_negative_value_is_ineffective"
)]
#[cfg(any(kani, test))]
fn model_non_negative_setter(previous: f64, requested: f64) -> f64 {
    destiny_original_spec::apply_non_negative_setter(previous, requested)
}

#[destiny_delegate(target = "destiny_original_spec::clamp_speed_fraction")]
#[cfg(any(kani, test))]
fn model_speed_fraction(requested: f64) -> f64 {
    destiny_original_spec::clamp_speed_fraction(requested)
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn proc_macro_metadata_and_derive_are_usable_in_normal_rust() {
        assert_eq!(
            ADD_DURING_EVOLVE.1,
            "test_balls_can_not_be_added_during_evolve"
        );
        assert_eq!(SetterProbe::DESTINY_FORMAL_MODEL, "SetterProbe");
        let probe = SetterProbe { previous: 3.0 };
        assert_eq!(model_non_negative_setter(probe.previous, -1.0), 3.0);
        assert_eq!(model_speed_fraction(2.0), 1.0);
    }
}
#[cfg(any(kani, test))]
mod iterative_sphere_production_contracts;
