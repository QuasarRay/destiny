use derive_deftly::{Deftly, define_derive_deftly};
use kani_metadata::ContractClause;
use serde::{Deserialize, Serialize};

define_derive_deftly! {
    KVerusEvidenceArtifactMetadata:

    impl $ttype {
        pub(crate) const ALL: &'static [Self] = &[$(Self::$vname,)];

        pub(crate) const fn kverus_name(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(name)) as expr},)
            }
        }

        pub(crate) const fn filename(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(filename)) as expr},)
            }
        }

        pub(crate) const fn status(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(status)) as expr},)
            }
        }

        pub(crate) const fn proof_credit_source(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(proof_credit_source)) as expr},)
            }
        }
    }
}

define_derive_deftly! {
    KVerusLifecycleStageMetadata:

    impl $ttype {
        pub(crate) const ALL: &'static [Self] = &[$(Self::$vname,)];

        pub(crate) const fn kverus_name(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(name)) as expr},)
            }
        }

        pub(crate) const fn rank(self) -> Option<u8> {
            match self {
                $(Self::$vname => ${vmeta(kverus(rank)) as expr},)
            }
        }

        pub(crate) const fn recommended_next_action(self) -> Option<&'static str> {
            match self {
                $(Self::$vname => ${vmeta(kverus(next)) as expr},)
            }
        }

        pub(crate) fn from_kverus_name(name: &str) -> Option<Self> {
            Self::ALL
                .iter()
                .copied()
                .find(|stage| stage.kverus_name() == name)
        }
    }
}

define_derive_deftly! {
    KVerusLifecycleActionMetadata:

    impl $ttype {
        pub(crate) const ALL: &'static [Self] = &[$(Self::$vname,)];

        pub(crate) const fn kverus_name(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(name)) as expr},)
            }
        }

        pub(crate) const fn allowed_stages(self) -> &'static [ReviewedLifecycleStage] {
            match self {
                $(Self::$vname => ${vmeta(kverus(allowed)) as expr},)
            }
        }
    }
}

define_derive_deftly! {
    KVerusCargoTargetMetadata:

    impl $ttype {
        pub(crate) const fn kani_flag(self) -> Option<&'static str> {
            match self {
                $(Self::$vname => ${vmeta(kverus(flag)) as expr},)
            }
        }

        pub(crate) const fn requires_name(self) -> bool {
            match self {
                $(Self::$vname => ${vmeta(kverus(requires_name)) as expr},)
            }
        }
    }
}

define_derive_deftly! {
    KVerusReviewedWorkflowStepMetadata:

    impl $ttype {
        pub(crate) const ALL: &'static [Self] = &[$(Self::$vname,)];

        pub(crate) const fn kverus_name(self) -> &'static str {
            match self {
                $(Self::$vname => ${vmeta(kverus(name)) as expr},)
            }
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Deftly)]
#[derive_deftly(KVerusCargoTargetMetadata)]
#[serde(rename_all = "snake_case")]
pub enum ReviewedCargoTarget {
    #[deftly(kverus(flag = { None }, requires_name = { false }))]
    Default,
    #[deftly(kverus(flag = { Some("--lib") }, requires_name = { false }))]
    Lib,
    #[deftly(kverus(flag = { Some("--bins") }, requires_name = { false }))]
    Bins,
    #[deftly(kverus(flag = { Some("--bin") }, requires_name = { true }))]
    Bin,
}

impl Default for ReviewedCargoTarget {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReviewedVerificationSelection {
    pub packages: Vec<String>,
    pub features: Vec<String>,
    pub all_features: bool,
    pub no_default_features: bool,
    pub target: ReviewedCargoTarget,
    pub target_name: Option<String>,
    pub functions: Vec<String>,
}

impl Default for ReviewedVerificationSelection {
    fn default() -> Self {
        Self {
            packages: Vec::new(),
            features: Vec::new(),
            all_features: false,
            no_default_features: false,
            target: ReviewedCargoTarget::Default,
            target_name: None,
            functions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReviewedVerificationLimits {
    pub timeout_seconds: u64,
    pub memory_mib: Option<u64>,
    pub max_concurrency: usize,
}

impl Default for ReviewedVerificationLimits {
    fn default() -> Self {
        Self {
            timeout_seconds: 120,
            memory_mib: None,
            max_concurrency: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedVerificationConfig {
    pub schema: u32,
    #[serde(default)]
    pub selection: ReviewedVerificationSelection,
    #[serde(default)]
    pub limits: ReviewedVerificationLimits,
}

impl Default for ReviewedVerificationConfig {
    fn default() -> Self {
        Self {
            schema: 1,
            selection: ReviewedVerificationSelection::default(),
            limits: ReviewedVerificationLimits::default(),
        }
    }
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
    pub selection: ReviewedVerificationSelection,
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
    VerifyReviewed {
        manifest_path: String,
        reviewed_dir: String,
        campaign_dir: String,
        verus: String,
        config_path: Option<String>,
    },
    RunReviewed {
        reviewed_dir: String,
        campaign_dir: String,
        verus: String,
    },
    FinalizeReviewed { reviewed_dir: String, campaign_dir: String },
    ManifestReviewed { reviewed_dir: String },
    AuditReviewed { reviewed_dir: String },
    StatusReviewed { reviewed_dir: String },
    ValidateReviewed { reviewed_dir: String },
    ExplainReviewed { reviewed_dir: String },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignExecutionSummary {
    pub status: String,
    pub checked_entry_count: usize,
    pub case_acceptance_exit: i32,
    pub obligations: Vec<String>,
    pub timeout_seconds: u64,
    pub memory_mib: Option<u64>,
    pub max_concurrency: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedCampaignExecutionReceipt {
    pub schema: u32,
    pub status: &'static str,
    pub engine: &'static str,
    pub reviewed_dir: String,
    pub campaign_dir: String,
    pub runner_receipt_path: String,
    pub execution_receipt_path: String,
    pub canonical_execution: String,
    pub runner_status: String,
    pub verus_checked_scoped: bool,
    pub checked_entry_count: usize,
    pub case_acceptance_exit: i32,
    pub obligations: Vec<String>,
    pub limits: ReviewedVerificationLimits,
    pub proof_credit: bool,
    pub proof_credit_source: &'static str,
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
    pub controller_obligations: Vec<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deftly)]
#[derive_deftly(KVerusEvidenceArtifactMetadata)]
pub enum EvidenceArtifactKind {
    #[deftly(kverus(name = { "reviewed-preparation-receipt" }, filename = { "reviewed-preparation-receipt.json" }, status = { "KVERUS_REVIEWED_BINDING_PREPARED" }, proof_credit_source = { "none" }))]
    ReviewedPreparationReceipt,
    #[deftly(kverus(name = { "verus-binding" }, filename = { "verus-binding.json" }, status = { "BOUND_FOR_VERUS" }, proof_credit_source = { "none" }))]
    VerusBinding,
    #[deftly(kverus(name = { "verus-receipt-binding" }, filename = { "verus-receipt-binding.json" }, status = { "KANI_SELECTED_VERUS_CHECKED_SCOPED" }, proof_credit_source = { "verus_checked_scoped" }))]
    VerusReceiptBinding,
    #[deftly(kverus(name = { "reviewed-finalization-receipt" }, filename = { "reviewed-finalization-receipt.json" }, status = { "KVERUS_REVIEWED_VERUS_FINALIZED" }, proof_credit_source = { "verus_checked_scoped" }))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deftly)]
#[derive_deftly(KVerusLifecycleStageMetadata)]
pub enum ReviewedLifecycleStage {
    #[deftly(kverus(name = { "reviewed-empty" }, rank = { Some(0u8) }, next = { Some("prepare-reviewed") }))]
    Empty,
    #[deftly(kverus(name = { "reviewed-prepared" }, rank = { Some(1u8) }, next = { Some("run-reviewed") }))]
    Prepared,
    #[deftly(kverus(name = { "reviewed-finalized" }, rank = { Some(2u8) }, next = { Some("manifest-reviewed") }))]
    Finalized,
    #[deftly(kverus(name = { "reviewed-manifested" }, rank = { Some(3u8) }, next = { Some("audit-reviewed") }))]
    Manifested,
    #[deftly(kverus(name = { "reviewed-audited" }, rank = { Some(4u8) }, next = { None }))]
    Audited,
    #[deftly(kverus(name = { "reviewed-inconsistent" }, rank = { None }, next = { None }))]
    Inconsistent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deftly)]
#[derive_deftly(KVerusLifecycleActionMetadata)]
pub enum ReviewedLifecycleAction {
    #[deftly(kverus(name = { "prepare-reviewed" }, allowed = { &[ReviewedLifecycleStage::Empty, ReviewedLifecycleStage::Prepared] }))]
    Prepare,
    #[deftly(kverus(name = { "run-reviewed" }, allowed = { &[ReviewedLifecycleStage::Prepared] }))]
    Run,
    #[deftly(kverus(name = { "finalize-reviewed" }, allowed = { &[ReviewedLifecycleStage::Prepared, ReviewedLifecycleStage::Finalized] }))]
    Finalize,
    #[deftly(kverus(name = { "manifest-reviewed" }, allowed = { &[ReviewedLifecycleStage::Finalized, ReviewedLifecycleStage::Manifested] }))]
    Manifest,
    #[deftly(kverus(name = { "audit-reviewed" }, allowed = { &[ReviewedLifecycleStage::Manifested, ReviewedLifecycleStage::Audited] }))]
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deftly)]
#[derive_deftly(KVerusReviewedWorkflowStepMetadata)]
pub enum ReviewedVerificationStep {
    #[deftly(kverus(name = { "prepare-reviewed" }))]
    Prepare,
    #[deftly(kverus(name = { "run-reviewed" }))]
    Run,
    #[deftly(kverus(name = { "finalize-reviewed" }))]
    Finalize,
    #[deftly(kverus(name = { "manifest-reviewed" }))]
    Manifest,
    #[deftly(kverus(name = { "audit-reviewed" }))]
    Audit,
    #[deftly(kverus(name = { "validate-reviewed" }))]
    Validate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedIntegrityArtifact {
    pub kind: String,
    pub canonical_token: String,
    pub path: String,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedIntegrityReport {
    pub schema: u32,
    pub status: &'static str,
    pub reviewed_dir: String,
    pub lifecycle_stage: String,
    pub canonical_stage: String,
    pub canonical_validation: String,
    pub valid: bool,
    pub validated_artifact_count: usize,
    pub artifacts: Vec<ReviewedIntegrityArtifact>,
    pub violations: Vec<String>,
    pub proof_credit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedLifecycleReport {
    pub schema: u32,
    pub status: &'static str,
    pub reviewed_dir: String,
    pub stage: &'static str,
    pub canonical_stage: String,
    pub consistent: bool,
    pub present_artifacts: Vec<String>,
    pub missing_artifacts: Vec<String>,
    pub legal_next_actions: Vec<&'static str>,
    pub proof_credit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedExplanationReport {
    pub schema: u32,
    pub status: &'static str,
    pub reviewed_dir: String,
    pub lifecycle: ReviewedLifecycleReport,
    pub integrity: ReviewedIntegrityReport,
    pub recommended_next_action: Option<&'static str>,
    pub ready_for_next_action: bool,
    pub complete: bool,
    pub blockers: Vec<String>,
    pub proof_credit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewedVerificationReceipt {
    pub schema: u32,
    pub status: &'static str,
    pub engine: &'static str,
    pub reviewed_dir: String,
    pub campaign_dir: String,
    pub canonical_workflow: String,
    pub starting_stage: String,
    pub preparation_reused: bool,
    pub executed_steps: Vec<&'static str>,
    pub reused_steps: Vec<&'static str>,
    pub selected_count: usize,
    pub controller_obligation_count: usize,
    pub checked_entry_count: usize,
    pub checked_receipt_count: usize,
    pub verified_artifact_count: usize,
    pub contract_inventory_count: usize,
    pub contract_clause_count: usize,
    pub translation_correctness: String,
    pub manifest_path: String,
    pub manifest_digest_algorithm: &'static str,
    pub manifest_digest: String,
    pub final_stage: String,
    pub integrity_valid: bool,
    pub proof_credit: bool,
    pub proof_credit_source: &'static str,
    pub selection: ReviewedVerificationSelection,
    pub limits: ReviewedVerificationLimits,
}
