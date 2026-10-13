// Copyright © 2023 CCP ehf. MIT license, retained from original Destiny.
// Original Settings.h defaults and Settings.cpp Apply/Get/GetDefault/Reset at
// ef738406b61ad06bae2f402857e464735ab106a2. Python object type checking, global
// interpreter lifetime and the Blue property binding are outside this model.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginalSettingsConfiguration {
    pub collision_max_iterations: usize,
    pub use_iterative_collision: bool,
    pub use_dynamical_orientation: bool,
    pub disable_dynamical_orientation_for_missiles: bool,
    pub use_new_orbit: bool,
}

impl Default for OriginalSettingsConfiguration {
    fn default() -> Self {
        Self::original_defaults()
    }
}

impl OriginalSettingsConfiguration {
    #[must_use]
    pub const fn original_defaults() -> Self {
        Self {
            collision_max_iterations: 20,
            use_iterative_collision: false,
            use_dynamical_orientation: false,
            disable_dynamical_orientation_for_missiles: false,
            use_new_orbit: false,
        }
    }
}

pub struct OriginalGlobalSettings {
    configuration: OriginalSettingsConfiguration,
}

impl Default for OriginalGlobalSettings {
    fn default() -> Self {
        Self {
            configuration: OriginalSettingsConfiguration::original_defaults(),
        }
    }
}

impl OriginalGlobalSettings {
    pub fn apply(&mut self, configuration: OriginalSettingsConfiguration) {
        // The original copies every field; Get creates an independent object.
        self.configuration = configuration;
    }

    #[must_use]
    pub fn get(&self) -> OriginalSettingsConfiguration {
        self.configuration
    }

    #[must_use]
    pub fn get_default(&self) -> OriginalSettingsConfiguration {
        OriginalSettingsConfiguration::original_defaults()
    }

    pub fn reset(&mut self) {
        self.configuration = OriginalSettingsConfiguration::original_defaults();
    }
}
