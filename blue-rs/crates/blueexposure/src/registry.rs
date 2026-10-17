use std::collections::BTreeMap;

use bevy::prelude::{Reflect, Resource};
use delegate::delegate;

use crate::{BlueClassInfo, BlueEnumInfo, BlueFunctionInfo, BlueLegacyTestInfo};

#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct BlueRegistry {
    classes: BTreeMap<String, BlueClassInfo>,
    functions: BTreeMap<String, BlueFunctionInfo>,
    enums: BTreeMap<String, BlueEnumInfo>,
    legacy_tests: BTreeMap<String, BlueLegacyTestInfo>,
}

impl BlueRegistry {
    pub fn register_class(&mut self, info: BlueClassInfo) {
        self.classes.insert(info.qualified_name(), info);
    }

    pub fn register_function(&mut self, info: BlueFunctionInfo) {
        self.functions.insert(info.qualified_name(), info);
    }

    pub fn register_enum(&mut self, info: BlueEnumInfo) {
        self.enums.insert(info.qualified_name(), info);
    }

    pub fn register_legacy_test(&mut self, info: BlueLegacyTestInfo) {
        self.legacy_tests.insert(info.legacy_name.clone(), info);
    }

    #[must_use]
    pub fn class(&self, qualified_name: &str) -> Option<&BlueClassInfo> {
        self.classes.get(qualified_name)
    }

    #[must_use]
    pub fn enum_info(&self, qualified_name: &str) -> Option<&BlueEnumInfo> {
        self.enums.get(qualified_name)
    }

    delegate! {
        to self.classes {
            #[call(len)]
            pub fn class_count(&self) -> usize;
        }
        to self.functions {
            #[call(len)]
            pub fn function_count(&self) -> usize;
        }
        to self.enums {
            #[call(len)]
            pub fn enum_count(&self) -> usize;
        }
    }

    #[must_use]
    pub fn classes(&self) -> impl Iterator<Item = &BlueClassInfo> {
        self.classes.values()
    }

    #[must_use]
    pub fn functions(&self) -> impl Iterator<Item = &BlueFunctionInfo> {
        self.functions.values()
    }

    #[must_use]
    pub fn enums(&self) -> impl Iterator<Item = &BlueEnumInfo> {
        self.enums.values()
    }

    #[must_use]
    pub fn legacy_tests(&self) -> impl Iterator<Item = &BlueLegacyTestInfo> {
        self.legacy_tests.values()
    }

    #[must_use]
    pub fn class_names(&self) -> Vec<String> {
        self.classes.keys().cloned().collect()
    }

    #[must_use]
    pub fn mapped_legacy_test_count(&self) -> usize {
        self.legacy_tests.values().filter(|test| test.mapped()).count()
    }
}
