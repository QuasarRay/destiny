use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub enum BlueClassKind {
    #[default]
    Component,
    Resource,
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlueFieldInfo {
    pub name: String,
    pub readable: bool,
    pub writable: bool,
    pub persisted: bool,
}

impl BlueFieldInfo {
    #[must_use]
    pub fn new(name: impl Into<String>, readable: bool, writable: bool, persisted: bool) -> Self {
        Self {
            name: name.into(),
            readable,
            writable,
            persisted,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlueClassInfo {
    pub module: String,
    pub name: String,
    pub doc: String,
    pub kind: BlueClassKind,
    pub fields: Vec<BlueFieldInfo>,
}

impl BlueClassInfo {
    #[must_use]
    pub fn new(
        module: impl Into<String>,
        name: impl Into<String>,
        doc: impl Into<String>,
        kind: BlueClassKind,
        fields: Vec<BlueFieldInfo>,
    ) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
            doc: doc.into(),
            kind,
            fields,
        }
    }

    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.module, self.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlueFunctionInfo {
    pub module: String,
    pub name: String,
    pub doc: String,
}

impl BlueFunctionInfo {
    #[must_use]
    pub fn new(module: impl Into<String>, name: impl Into<String>, doc: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
            doc: doc.into(),
        }
    }

    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.module, self.name)
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlueEnumInfo {
    pub module: String,
    pub name: String,
    pub doc: String,
    pub variants: Vec<String>,
}

impl BlueEnumInfo {
    #[must_use]
    pub fn new(
        module: impl Into<String>,
        name: impl Into<String>,
        doc: impl Into<String>,
        variants: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
            doc: doc.into(),
            variants: variants.into_iter().map(Into::into).collect(),
        }
    }

    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.module, self.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub struct BlueLegacyTestInfo {
    pub source: String,
    pub legacy_name: String,
    pub rust_test: Option<String>,
}

impl BlueLegacyTestInfo {
    #[must_use]
    pub fn mapped(&self) -> bool {
        self.rust_test.is_some()
    }
}
