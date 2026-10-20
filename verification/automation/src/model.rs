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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreparationReceipt {
    pub schema: u32,
    pub status: &'static str,
    pub default_engine: &'static str,
    pub kani_source_commit: &'static str,
    pub proof_credit: bool,
    pub manifest_path: String,
    pub output_dir: String,
    pub metadata_path: String,
    pub discovery_path: String,
    pub plan_path: String,
    pub counts: PlanCounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandDescriptor {
    pub name: String,
    pub synopsis: String,
    pub category: String,
    pub canonical_token: String,
    pub proof_credit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationCommand {
    Doctor,
    Setup,
    Discover { path: String },
    Plan { path: String },
    Prepare { manifest_path: String, output_dir: String },
    PrepareReviewed { manifest_path: String, output_dir: String },
    FinalizeReviewed { reviewed_dir: String, campaign_dir: String },
    ManifestReviewed { reviewed_dir: String },
    AuditReviewed { reviewed_dir: String },
    Explain,
    Verify { args: Vec<String> },
    ReplayKaniReference { args: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedPreparationReceipt {
    pub schema: u32,
    pub status: &'static str,
    pub default_engine: &'static str,
    pub proof_credit: bool,
    pub verus_campaign_checked: bool,
    pub preparation: PreparationReceipt,
    pub binding_path: String,
    pub binding_status: String,
    pub selected_count: usize,
    pub unresolved_count: usize,
    pub controller_obligation_count: usize,
    pub contract_inventory_count: usize,
    pub contract_clause_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedFinalizationReceipt {
    pub schema: u32,
    pub status: &'static str,
    pub engine: &'static str,
    pub kani_proof_credit: bool,
    pub verus_checked_scoped: bool,
    pub reviewed_dir: String,
    pub campaign_dir: String,
    pub binding_path: String,
    pub campaign_binding_path: String,
    pub campaign_binding_status: String,
    pub checked_receipt_count: usize,
    pub contract_inventory_count: usize,
    pub contract_clause_count: usize,
    pub translation_correctness: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewedBindingSummary {
    pub status: String,
    pub selected_count: usize,
    pub controller_obligation_count: usize,
    pub contract_inventory_count: usize,
    pub contract_clause_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignBindingSummary {
    pub status: String,
    pub checked_receipt_count: usize,
    pub contract_inventory_count: usize,
    pub contract_clause_count: usize,
    pub translation_correctness: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceArtifact {
    pub kind: String,
    pub canonical_token: String,
    pub path: String,
    pub status: String,
    pub proof_credit_source: String,
    pub digest_algorithm: &'static str,
    pub digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceArtifactKind {
    ReviewedPreparationReceipt,
    VerusBinding,
    VerusReceiptBinding,
    ReviewedFinalizationReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceArtifactSpec {
    pub kind: EvidenceArtifactKind,
    pub filename: &'static str,
    pub status: &'static str,
    pub proof_credit_source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedEvidenceManifest {
    pub schema: u32,
    pub status: &'static str,
    pub engine: &'static str,
    pub kani_proof_credit: bool,
    pub verus_checked_scoped: bool,
    pub reviewed_dir: String,
    pub selected_count: usize,
    pub controller_obligation_count: usize,
    pub checked_receipt_count: usize,
    pub contract_inventory_count: usize,
    pub contract_clause_count: usize,
    pub translation_correctness: String,
    pub proof_credit_source: &'static str,
    pub artifacts: Vec<EvidenceArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedEvidenceAuditReceipt {
    pub schema: u32,
    pub status: &'static str,
    pub engine: &'static str,
    pub kani_proof_credit: bool,
    pub verus_checked_scoped: bool,
    pub reviewed_dir: String,
    pub manifest_path: String,
    pub manifest_digest_algorithm: &'static str,
    pub manifest_digest: String,
    pub verified_artifact_count: usize,
    pub proof_credit_source: &'static str,
}
