use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LegacySymbol {
    pub source: String,
    pub qualified_name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct LegacyTest {
    pub source: String,
    pub legacy_name: String,
    pub line: Option<u32>,
    pub discovery: String,
    pub rust_test: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RustTest {
    pub crate_name: String,
    pub source: String,
    pub module_path: Vec<String>,
    pub function: String,
    pub semantic_name: String,
    pub body_name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TestMap {
    #[serde(default, rename = "test")]
    pub tests: Vec<TestMapping>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TestMapping {
    pub legacy: String,
    pub rust: String,
}

impl TestMap {
    #[must_use]
    pub fn by_legacy(&self) -> BTreeMap<&str, &str> {
        self.tests
            .iter()
            .map(|mapping| (mapping.legacy.as_str(), mapping.rust.as_str()))
            .collect()
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Inventory {
    pub symbols: Vec<LegacySymbol>,
    pub tests: Vec<LegacyTest>,
    pub rust_tests: Vec<RustTest>,
}
