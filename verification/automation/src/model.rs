use kani_metadata::ContractClause;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryDisposition {
    KaniEligible,
    Excluded,
    Unsupported,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveryCandidate {
    pub crate_name: String,
    pub function: String,
    pub disposition: DiscoveryDisposition,
    pub reason: String,
    pub provenance: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractDiscovery {
    pub crate_name: String,
    pub function: String,
    pub file: String,
    pub harnesses: Vec<String>,
    pub clauses: Vec<ContractClause>,
    pub provenance: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveryReport {
    pub default_engine: &'static str,
    pub kani_source_commit: &'static str,
    pub crates_with_autoharness_metadata: usize,
    pub candidates: Vec<DiscoveryCandidate>,
    pub contracted_functions: Vec<ContractDiscovery>,
    pub proof_credit: bool,
}
