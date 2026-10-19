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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanCounts {
    pub selected: usize,
    pub blocked: usize,
    pub unsupported: usize,
    pub excluded: usize,
    pub contracted_functions: usize,
    pub contract_clauses: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannedContractClause {
    pub kind: &'static str,
    pub expression: String,
    pub canonical_tokens: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractPlan {
    pub crate_name: String,
    pub function: String,
    pub file: String,
    pub harnesses: Vec<String>,
    pub clauses: Vec<PlannedContractClause>,
    pub planner_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AutomationPlan {
    pub schema: u32,
    pub default_engine: &'static str,
    pub kani_source_commit: &'static str,
    pub proof_credit: bool,
    pub counts: PlanCounts,
    pub selected: Vec<DiscoveryCandidate>,
    pub blocked: Vec<DiscoveryCandidate>,
    pub unsupported: Vec<DiscoveryCandidate>,
    pub excluded: Vec<DiscoveryCandidate>,
    pub contracted_functions: Vec<ContractPlan>,
}
