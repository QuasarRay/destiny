use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
use derive_deftly::{define_derive_deftly, Deftly};

/// Compile-time view of the Rust fields in Blue's metadata records.
/// This is intentionally separate from runtime class field metadata.
pub trait BlueMetadataShape {
    fn rust_field_names() -> &'static [&'static str];
}

/// Generate metadata schemas from Rust fields, without a second list.
define_derive_deftly! {
    BlueMetadataShape:
    impl crate::metadata::BlueMetadataShape for $ttype {
        fn rust_field_names() -> &'static [&'static str] {
            &[ $( stringify!($fname), ) ]
        }
    }
}


#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Reflect, Serialize, Deserialize)]
pub enum BlueClassKind {
    #[default]
    Component,
    Resource,
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize, Deftly)]
#[derive_deftly(BlueMetadataShape)]
pub struct BlueFieldInfo {
    pub name: String,
    /// Human-facing Blue field documentation, sourced from Rust doc comments.
    #[serde(default)]
    pub doc: String,
    pub readable: bool,
    pub writable: bool,
    pub persisted: bool,
}

impl BlueFieldInfo {
    #[must_use]
    pub fn new(name: impl Into<String>, readable: bool, writable: bool, persisted: bool) -> Self {
        Self {
            name: name.into(),
            doc: String::new(),
            readable,
            writable,
            persisted,
        }
    }

    #[must_use]
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = doc.into();
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize, Deftly)]
#[derive_deftly(BlueMetadataShape)]
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

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize, Deftly)]
#[derive_deftly(BlueMetadataShape)]
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


#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize, Deftly)]
#[derive_deftly(BlueMetadataShape)]
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

#[derive(Clone, Debug, Eq, PartialEq, Reflect, Serialize, Deserialize, Deftly)]
#[derive_deftly(BlueMetadataShape)]
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
