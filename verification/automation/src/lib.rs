#![forbid(unsafe_code)]

mod components;
mod model;

pub use components::{
    CanAssembleReviewedEvidence, CanAuditReviewedEvidence, CanBuildAutomationPlan,
    CanBuildReviewedEvidenceManifest, CanCanonicalizeContractClause,
    CanCanonicalizeStaticIdentity, CanDecodeJsonReceipt, CanDescribeCommands,
    CanDescribeEvidenceArtifacts, CanDigestArtifactBytes, CanExplainReviewed,
    CanFinalizeReviewedCampaign, CanInspectArtifactPresence,
    CanInspectReviewedLifecycle, CanNormalizeMetadata, CanParseMetadataBytes,
    CanPrepareAutomation, CanPrepareReviewedBinding, CanReadJsonArtifact,
    CanReadWriteArtifacts, CanRequireReviewedTransition, CanRouteCommand,
    CanRunJsonPythonTool, CanRunPythonTool, CanRunReviewedCampaign,
    CanValidateCampaignBinding, CanValidateCampaignExecution,
    CanValidateReviewedIntegrity, CanVerifyReviewed,
    CanValidateContractClause, CanValidateReviewedBinding, HasProjectRoot, KVerusContext,
};
pub use model::{
    AutomationCommand, AutomationPlan, CampaignBindingSummary, CampaignExecutionSummary,
    CommandDescriptor,
    ContractDiscovery, ContractPlan, DiscoveryCandidate, DiscoveryDisposition,
    DiscoveryReport, EvidenceArtifact, EvidenceArtifactKind, EvidenceArtifactSpec,
    PlanCounts, PlannedContractClause, PreparationReceipt, ReviewedBindingSummary,
    ReviewedCampaignExecutionReceipt, ReviewedEvidenceAuditReceipt,
    ReviewedEvidenceManifest, ReviewedExplanationReport, ReviewedFinalizationReceipt,
    ReviewedIntegrityArtifact, ReviewedIntegrityReport,
    ReviewedLifecycleAction, ReviewedLifecycleReport, ReviewedLifecycleStage,
    ReviewedPreparationReceipt, ReviewedVerificationReceipt, ReviewedVerificationStep,
};

use kani_metadata::KaniMetadata;
use std::io::Read;

pub const DEFAULT_ENGINE: &str = "verus";
pub const KANI_SOURCE_COMMIT: &str = "0d2328a93f0e0ff66132d6bfa1a7d884877cf862";

/// Compatibility facade retained during the incremental CGP migration.
///
/// The actual JSON parser is selected by `KVerusContext` through CGP.
pub fn parse_metadata(mut reader: impl Read) -> Result<Vec<KaniMetadata>, serde_json::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(serde_json::Error::io)?;
    KVerusContext.parse_metadata_bytes(&bytes)
}

/// Compatibility facade retained for existing Python/CLI callers.
///
/// Normalization and clause-boundary validation are now CGP-owned components.
pub fn normalize_autoharness_metadata(metadata: &[KaniMetadata]) -> DiscoveryReport {
    KVerusContext.normalize_metadata(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kani_metadata::{
        AutoHarnessMetadata, AutoHarnessSkipReason, ContractClause, ContractClauseKind,
        ContractedFunction,
    };
    use std::collections::{BTreeMap, BTreeSet};

    fn metadata() -> KaniMetadata {
        let mut chosen = BTreeSet::new();
        chosen.insert("crate::good".to_string());

        let mut skipped = BTreeMap::new();
        skipped.insert("crate::filtered".to_string(), AutoHarnessSkipReason::UserFilter);
        skipped.insert(
            "crate::slice".to_string(),
            AutoHarnessSkipReason::RequiresBoundedArguments(vec![
                ("items".to_string(), "&[u8]".to_string()),
            ]),
        );

        KaniMetadata {
            crate_name: "fixture".to_string(),
            proof_harnesses: vec![],
            unsupported_features: vec![],
            test_harnesses: vec![],
            contracted_functions: vec![ContractedFunction {
                function: "crate::good".to_string(),
                file: "src/lib.rs".to_string(),
                harnesses: vec!["crate::good_contract".to_string()],
                clauses: vec![
                    ContractClause {
                        kind: ContractClauseKind::Requires,
                        expression: "x".to_string(),
                    },
                    ContractClause {
                        kind: ContractClauseKind::Ensures,
                        expression: "|_| x".to_string(),
                    },
                ],
            }],
            autoharness_md: Some(AutoHarnessMetadata { chosen, skipped }),
        }
    }

    #[test]
    fn preserves_kani_chosen_skipped_and_contract_metadata_without_proof_credit() {
        let report = normalize_autoharness_metadata(&[metadata()]);
        assert_eq!(report.default_engine, "verus");
        assert!(!report.proof_credit);
        assert_eq!(report.crates_with_autoharness_metadata, 1);
        assert_eq!(report.candidates.len(), 3);
        assert_eq!(report.candidates[0].function, "crate::filtered");
        assert_eq!(report.candidates[0].disposition, DiscoveryDisposition::Excluded);
        assert_eq!(report.candidates[1].function, "crate::good");
        assert_eq!(report.candidates[1].disposition, DiscoveryDisposition::KaniEligible);
        assert_eq!(report.candidates[2].function, "crate::slice");
        assert_eq!(report.candidates[2].disposition, DiscoveryDisposition::Blocked);
        assert!(report.candidates[2].reason.contains("items: &[u8]"));
        assert_eq!(report.contracted_functions.len(), 1);
        assert_eq!(report.contracted_functions[0].crate_name, "fixture");
        assert_eq!(report.contracted_functions[0].function, "crate::good");
        assert_eq!(report.contracted_functions[0].file, "src/lib.rs");
        assert_eq!(
            report.contracted_functions[0].harnesses,
            vec!["crate::good_contract".to_string()]
        );
        assert_eq!(report.contracted_functions[0].clauses.len(), 2);
        assert_eq!(
            report.contracted_functions[0].clauses[0],
            ContractClause {
                kind: ContractClauseKind::Requires,
                expression: "x".to_string(),
            }
        );
        assert_eq!(
            report.contracted_functions[0].clauses[1],
            ContractClause {
                kind: ContractClauseKind::Ensures,
                expression: "|_| x".to_string(),
            }
        );
        assert_eq!(
            report.contracted_functions[0].provenance,
            "kani_metadata::KaniMetadata::contracted_functions"
        );
    }

    #[test]
    fn deduplicates_identical_contract_metadata_across_combined_inputs() {
        let md = metadata();
        let report = normalize_autoharness_metadata(&[md.clone(), md]);
        assert_eq!(report.contracted_functions.len(), 1);
    }

    #[test]
    fn parses_single_metadata_object() {
        let bytes = serde_json::to_vec(&metadata()).unwrap();
        let parsed = parse_metadata(bytes.as_slice()).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].crate_name, "fixture");
    }

    #[test]
    fn parses_metadata_array() {
        let bytes = serde_json::to_vec(&vec![metadata(), metadata()]).unwrap();
        let parsed = parse_metadata(bytes.as_slice()).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn planner_groups_discovery_without_granting_proof_credit() {
        let report = normalize_autoharness_metadata(&[metadata()]);
        let plan = KVerusContext.build_automation_plan(&report).unwrap();

        assert_eq!(plan.schema, 1);
        assert!(!plan.proof_credit);
        assert_eq!(plan.counts.selected, 1);
        assert_eq!(plan.counts.blocked, 1);
        assert_eq!(plan.counts.unsupported, 0);
        assert_eq!(plan.counts.excluded, 1);
        assert_eq!(plan.counts.contracted_functions, 1);
        assert_eq!(plan.counts.contract_clauses, 2);
        assert_eq!(plan.selected[0].function, "crate::good");
        assert_eq!(plan.blocked[0].function, "crate::slice");
        assert_eq!(plan.excluded[0].function, "crate::filtered");

        let contract = &plan.contracted_functions[0];
        assert_eq!(contract.function, "crate::good");
        assert_eq!(contract.planner_status, "downstream_subset_check_required");
        assert_eq!(contract.clauses[0].kind, "requires");
        assert_eq!(contract.clauses[0].expression, "x");
        assert!(!contract.clauses[0].canonical_tokens.is_empty());
        assert_eq!(contract.clauses[1].kind, "ensures");
        assert!(!contract.clauses[1].canonical_tokens.is_empty());
    }

    #[test]
    fn command_catalog_is_cgp_owned_and_proc_macro_validated() {
        let commands = KVerusContext.command_descriptors().unwrap();
        let reviewed = commands
            .iter()
            .find(|command| command.name == "prepare-reviewed")
            .unwrap();
        assert_eq!(
            reviewed.synopsis,
            "prepare-reviewed --manifest-path <Cargo.toml> --output-dir <dir>"
        );
        assert_eq!(reviewed.category, "reviewed_binding");
        assert_eq!(reviewed.canonical_token, "prepare_reviewed");
        assert!(!reviewed.proof_credit);

        let usage = KVerusContext.command_usage().unwrap();
        assert!(usage.contains("prepare-reviewed --manifest-path <Cargo.toml> --output-dir <dir>"));
        assert!(usage.contains(
            "verify-reviewed --manifest-path <Cargo.toml> --reviewed-dir <dir> --campaign-dir <dir> --verus <path>"
        ));
        assert!(usage.contains(
            "run-reviewed --reviewed-dir <dir> --campaign-dir <dir> --verus <path>"
        ));
        assert!(usage.contains("finalize-reviewed --reviewed-dir <dir> --campaign-dir <dir>"));
        assert!(usage.contains("manifest-reviewed --reviewed-dir <dir>"));
        assert!(usage.contains("audit-reviewed --reviewed-dir <dir>"));
        assert!(usage.contains("status-reviewed --reviewed-dir <dir>"));
        assert!(usage.contains("validate-reviewed --reviewed-dir <dir>"));
        assert!(usage.contains("explain-reviewed --reviewed-dir <dir>"));
        assert!(usage.contains("replay-kani-reference <replay args...>"));

        let workflow = commands
            .iter()
            .find(|command| command.name == "verify-reviewed")
            .unwrap();
        assert_eq!(workflow.canonical_token, "verify_reviewed");
        assert_eq!(workflow.category, "verification");
        assert!(!workflow.proof_credit);

        let runner = commands
            .iter()
            .find(|command| command.name == "run-reviewed")
            .unwrap();
        assert_eq!(runner.canonical_token, "run_reviewed");
        assert_eq!(runner.category, "verification");
        assert!(!runner.proof_credit);

        let finalizer = commands
            .iter()
            .find(|command| command.name == "finalize-reviewed")
            .unwrap();
        assert_eq!(finalizer.canonical_token, "finalize_reviewed");
        assert_eq!(finalizer.category, "reviewed_binding");
        assert!(!finalizer.proof_credit);

        let manifest = commands
            .iter()
            .find(|command| command.name == "manifest-reviewed")
            .unwrap();
        assert_eq!(manifest.canonical_token, "manifest_reviewed");
        assert_eq!(manifest.category, "evidence");
        assert!(!manifest.proof_credit);

        assert_eq!(
            KVerusContext
                .canonicalize_static_identity("reviewed-finalization-receipt")
                .unwrap(),
            "reviewed_finalization_receipt"
        );

        let audit = commands
            .iter()
            .find(|command| command.name == "audit-reviewed")
            .unwrap();
        assert_eq!(audit.canonical_token, "audit_reviewed");
        assert_eq!(audit.category, "evidence");
        assert!(!audit.proof_credit);

        let status = commands
            .iter()
            .find(|command| command.name == "status-reviewed")
            .unwrap();
        assert_eq!(status.canonical_token, "status_reviewed");
        assert_eq!(status.category, "lifecycle");
        assert!(!status.proof_credit);

        let validation = commands
            .iter()
            .find(|command| command.name == "validate-reviewed")
            .unwrap();
        assert_eq!(validation.canonical_token, "validate_reviewed");
        assert_eq!(validation.category, "lifecycle");
        assert!(!validation.proof_credit);

        let explanation = commands
            .iter()
            .find(|command| command.name == "explain-reviewed")
            .unwrap();
        assert_eq!(explanation.canonical_token, "explain_reviewed");
        assert_eq!(explanation.category, "diagnostics");
        assert!(!explanation.proof_credit);
    }

    #[test]
    fn command_router_builds_typed_reviewed_preparation_invocation() {
        let command = KVerusContext
            .route_command(&[
                "prepare-reviewed".to_string(),
                "--output-dir".to_string(),
                ".aegis/out".to_string(),
                "--manifest-path".to_string(),
                "verification/spec/Cargo.toml".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::PrepareReviewed {
                manifest_path: "verification/spec/Cargo.toml".to_string(),
                output_dir: ".aegis/out".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_workflow_invocation() {
        let command = KVerusContext
            .route_command(&[
                "verify-reviewed".to_string(),
                "--verus".to_string(),
                "verus-dist/verus".to_string(),
                "--campaign-dir".to_string(),
                ".aegis/campaign".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
                "--manifest-path".to_string(),
                "verification/spec/Cargo.toml".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::VerifyReviewed {
                manifest_path: "verification/spec/Cargo.toml".to_string(),
                reviewed_dir: ".aegis/reviewed".to_string(),
                campaign_dir: ".aegis/campaign".to_string(),
                verus: "verus-dist/verus".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_execution_invocation() {
        let command = KVerusContext
            .route_command(&[
                "run-reviewed".to_string(),
                "--verus".to_string(),
                "verus-dist/verus".to_string(),
                "--campaign-dir".to_string(),
                ".aegis/campaign".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::RunReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
                campaign_dir: ".aegis/campaign".to_string(),
                verus: "verus-dist/verus".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_finalization_invocation() {
        let command = KVerusContext
            .route_command(&[
                "finalize-reviewed".to_string(),
                "--campaign-dir".to_string(),
                ".aegis/campaign".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::FinalizeReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
                campaign_dir: ".aegis/campaign".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_manifest_invocation() {
        let command = KVerusContext
            .route_command(&[
                "manifest-reviewed".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::ManifestReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_audit_invocation() {
        let command = KVerusContext
            .route_command(&[
                "audit-reviewed".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::AuditReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_validation_invocation() {
        let command = KVerusContext
            .route_command(&[
                "validate-reviewed".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::ValidateReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
            }
        );
    }

    #[test]
    fn command_router_builds_typed_reviewed_explanation_invocation() {
        let command = KVerusContext
            .route_command(&[
                "explain-reviewed".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::ExplainReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
            }
        );
    }

    #[test]
    fn campaign_execution_validator_requires_checked_scoped_verus_receipts() {
        let value = serde_json::json!({
            "schema": 1,
            "status": "CHECKED_SCOPED",
            "engine": "verus",
            "kani_proof_credit": false,
            "verus_checked_scoped": true,
            "deduplicated_entries": 3,
            "formal_audit": "CHECKED_SCOPED",
            "case_acceptance_exit": 0
        });
        let summary = KVerusContext.validate_campaign_execution(value).unwrap();
        assert_eq!(summary.status, "CHECKED_SCOPED");
        assert_eq!(summary.checked_entry_count, 3);
        assert_eq!(summary.case_acceptance_exit, 0);

        let rejected = serde_json::json!({
            "schema": 1,
            "status": "CHECKED_SCOPED",
            "engine": "verus",
            "kani_proof_credit": false,
            "verus_checked_scoped": true,
            "deduplicated_entries": 3,
            "formal_audit": "CHECKED_SCOPED",
            "case_acceptance_exit": 1
        });
        assert!(KVerusContext.validate_campaign_execution(rejected).is_err());
    }

    #[test]
    fn sha256_digest_provider_is_deterministic_and_standard() {
        assert_eq!(
            KVerusContext.digest_artifact_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn reviewed_evidence_policy_is_typed_and_complete() {
        let specs = KVerusContext.reviewed_evidence_artifact_specs();
        assert_eq!(specs.len(), 4);
        assert_eq!(
            KVerusContext.evidence_artifact_kind_name(specs[0].kind),
            "reviewed-preparation-receipt"
        );
        assert_eq!(specs[0].filename, "reviewed-preparation-receipt.json");
        assert_eq!(specs[0].proof_credit_source, "none");
        assert_eq!(
            KVerusContext.evidence_artifact_kind_name(specs[2].kind),
            "verus-receipt-binding"
        );
        assert_eq!(specs[2].proof_credit_source, "verus_checked_scoped");
    }

    #[test]
    fn command_router_builds_typed_reviewed_status_invocation() {
        let command = KVerusContext
            .route_command(&[
                "status-reviewed".to_string(),
                "--reviewed-dir".to_string(),
                ".aegis/reviewed".to_string(),
            ])
            .unwrap();

        assert_eq!(
            command,
            AutomationCommand::StatusReviewed {
                reviewed_dir: ".aegis/reviewed".to_string(),
            }
        );
    }

    #[test]
    fn reviewed_integrity_validates_semantics_beyond_presence() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "kverus-reviewed-integrity-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let root_string = root.display().to_string();
        let binding_path = root.join("verus-binding.json");

        let binding = serde_json::json!({
            "schema": 1,
            "status": "BOUND_FOR_VERUS",
            "default_engine": "verus",
            "kani_proof_credit": false,
            "selected": [{}],
            "controller_obligations": ["obligation-1"],
            "contract_inventory_count": 1,
            "contract_clause_count": 2,
            "unresolved": []
        });
        fs::write(
            &binding_path,
            serde_json::to_vec_pretty(&binding).unwrap(),
        )
        .unwrap();

        let preparation = serde_json::json!({
            "schema": 1,
            "status": "KVERUS_REVIEWED_BINDING_PREPARED",
            "default_engine": "verus",
            "proof_credit": false,
            "verus_campaign_checked": false,
            "preparation": {
                "schema": 1,
                "status": "KVERUS_PREPARED",
                "default_engine": "verus",
                "kani_source_commit": KANI_SOURCE_COMMIT,
                "proof_credit": false,
                "manifest_path": "verification/spec/Cargo.toml",
                "output_dir": root_string,
                "metadata_path": root.join("kani-metadata.json").display().to_string(),
                "discovery_path": root.join("normalized-discovery.json").display().to_string(),
                "plan_path": root.join("automation-plan.json").display().to_string(),
                "counts": {
                    "contracted_functions": 1,
                    "contract_clauses": 2
                }
            },
            "binding_path": binding_path.display().to_string(),
            "binding_status": "BOUND_FOR_VERUS",
            "selected_count": 1,
            "unresolved_count": 0,
            "controller_obligation_count": 1,
            "contract_inventory_count": 1,
            "contract_clause_count": 2
        });
        let preparation_path = root.join("reviewed-preparation-receipt.json");
        fs::write(
            &preparation_path,
            serde_json::to_vec_pretty(&preparation).unwrap(),
        )
        .unwrap();

        let valid = KVerusContext
            .validate_reviewed_integrity(&root_string)
            .unwrap();
        assert!(valid.valid, "{:?}", valid.violations);
        assert_eq!(valid.lifecycle_stage, "reviewed-prepared");
        assert_eq!(valid.canonical_stage, "reviewed_prepared");
        assert_eq!(valid.canonical_validation, "reviewed_integrity_validation");
        assert_eq!(valid.validated_artifact_count, 2);
        assert_eq!(
            valid.artifacts
                .iter()
                .map(|row| row.kind.as_str())
                .collect::<Vec<_>>(),
            vec!["reviewed-preparation-receipt", "verus-binding"]
        );
        assert!(!valid.proof_credit);

        let campaign = root.join("campaign");
        let mismatch = KVerusContext
            .verify_reviewed(
                "verification/other/Cargo.toml",
                &root_string,
                &campaign.display().to_string(),
                "missing-verus",
            )
            .unwrap_err();
        assert!(mismatch.contains("manifest mismatch"), "{mismatch}");

        let mut nested_corrupted = preparation.clone();
        nested_corrupted["preparation"]["default_engine"] = serde_json::json!("kani");
        fs::write(
            &preparation_path,
            serde_json::to_vec_pretty(&nested_corrupted).unwrap(),
        )
        .unwrap();
        let nested_invalid = KVerusContext
            .validate_reviewed_integrity(&root_string)
            .unwrap();
        assert!(!nested_invalid.valid);
        assert!(nested_invalid.violations[0].contains("preparation receipt"));

        fs::write(
            &preparation_path,
            serde_json::to_vec_pretty(&preparation).unwrap(),
        )
        .unwrap();

        let mut corrupted = preparation;
        corrupted["proof_credit"] = serde_json::json!(true);
        fs::write(
            &preparation_path,
            serde_json::to_vec_pretty(&corrupted).unwrap(),
        )
        .unwrap();

        let invalid = KVerusContext
            .validate_reviewed_integrity(&root_string)
            .unwrap();
        assert_eq!(invalid.lifecycle_stage, "reviewed-prepared");
        assert!(!invalid.valid);
        assert!(!invalid.violations.is_empty());
        assert!(invalid.violations[0].contains("preparation receipt"));
        assert!(!invalid.proof_credit);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reviewed_workflow_rejects_completed_lifecycle_before_execution() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "kverus-reviewed-workflow-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        for name in [
            "reviewed-preparation-receipt.json",
            "verus-binding.json",
            "verus-receipt-binding.json",
            "reviewed-finalization-receipt.json",
        ] {
            fs::write(root.join(name), b"{}\n").unwrap();
        }
        let error = KVerusContext
            .verify_reviewed(
                "verification/spec/Cargo.toml",
                &root.display().to_string(),
                &root.join("campaign").display().to_string(),
                "missing-verus",
            )
            .unwrap_err();
        assert!(error.contains("reviewed-empty or reviewed-prepared"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reviewed_lifecycle_is_typed_ordered_and_proc_macro_canonicalized() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "kverus-reviewed-lifecycle-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let root_string = root.display().to_string();

        let empty = KVerusContext.reviewed_lifecycle(&root_string).unwrap();
        assert_eq!(empty.stage, "reviewed-empty");
        assert_eq!(empty.canonical_stage, "reviewed_empty");
        assert_eq!(empty.legal_next_actions, vec!["prepare-reviewed"]);
        assert!(empty.consistent);
        assert!(!empty.proof_credit);

        for name in [
            "reviewed-preparation-receipt.json",
            "verus-binding.json",
        ] {
            fs::write(root.join(name), b"{}\n").unwrap();
        }
        let prepared = KVerusContext.reviewed_lifecycle(&root_string).unwrap();
        assert_eq!(prepared.stage, "reviewed-prepared");
        assert_eq!(prepared.canonical_stage, "reviewed_prepared");
        assert!(prepared.legal_next_actions.contains(&"run-reviewed"));
        assert!(prepared.legal_next_actions.contains(&"finalize-reviewed"));
        KVerusContext
            .require_reviewed_action(&root_string, ReviewedLifecycleAction::Run)
            .unwrap();
        KVerusContext
            .require_reviewed_action(&root_string, ReviewedLifecycleAction::Finalize)
            .unwrap();
        assert!(
            KVerusContext
                .require_reviewed_action(&root_string, ReviewedLifecycleAction::Manifest)
                .is_err()
        );

        for name in [
            "verus-receipt-binding.json",
            "reviewed-finalization-receipt.json",
        ] {
            fs::write(root.join(name), b"{}\n").unwrap();
        }
        let finalized = KVerusContext.reviewed_lifecycle(&root_string).unwrap();
        assert_eq!(finalized.stage, "reviewed-finalized");
        KVerusContext
            .require_reviewed_action(&root_string, ReviewedLifecycleAction::Manifest)
            .unwrap();

        fs::write(root.join("reviewed-evidence-manifest.json"), b"{}\n").unwrap();
        let manifested = KVerusContext.reviewed_lifecycle(&root_string).unwrap();
        assert_eq!(manifested.stage, "reviewed-manifested");
        KVerusContext
            .require_reviewed_action(&root_string, ReviewedLifecycleAction::Audit)
            .unwrap();

        fs::write(root.join("reviewed-evidence-audit.json"), b"{}\n").unwrap();
        let audited = KVerusContext.reviewed_lifecycle(&root_string).unwrap();
        assert_eq!(audited.stage, "reviewed-audited");
        assert!(
            KVerusContext
                .require_reviewed_action(&root_string, ReviewedLifecycleAction::Finalize)
                .is_err()
        );

        fs::remove_file(root.join("verus-binding.json")).unwrap();
        let inconsistent = KVerusContext.reviewed_lifecycle(&root_string).unwrap();
        assert_eq!(inconsistent.stage, "reviewed-inconsistent");
        assert!(!inconsistent.consistent);
        assert!(inconsistent.legal_next_actions.is_empty());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reviewed_explanation_composes_lifecycle_integrity_and_next_action() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "kverus-reviewed-explanation-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let root_string = root.display().to_string();

        let empty = KVerusContext.explain_reviewed(&root_string).unwrap();
        assert_eq!(empty.status, "KVERUS_REVIEWED_EXPLANATION");
        assert_eq!(empty.lifecycle.stage, "reviewed-empty");
        assert!(empty.integrity.valid);
        assert_eq!(empty.recommended_next_action, Some("prepare-reviewed"));
        assert!(empty.ready_for_next_action);
        assert!(!empty.complete);
        assert!(empty.blockers.is_empty());
        assert!(!empty.proof_credit);

        fs::write(root.join("reviewed-preparation-receipt.json"), b"{}\n").unwrap();
        let inconsistent = KVerusContext.explain_reviewed(&root_string).unwrap();
        assert_eq!(inconsistent.lifecycle.stage, "reviewed-inconsistent");
        assert!(!inconsistent.integrity.valid);
        assert_eq!(inconsistent.recommended_next_action, None);
        assert!(!inconsistent.ready_for_next_action);
        assert!(!inconsistent.complete);
        assert!(!inconsistent.blockers.is_empty());
        assert!(!inconsistent.proof_credit);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn proc_macro2_and_synthez_validate_contract_clause_syntax() {
        let valid = ContractClause {
            kind: ContractClauseKind::Ensures,
            expression: "|result: &u8| *result == 1".to_string(),
        };
        assert!(KVerusContext.validate_contract_clause(&valid).is_ok());

        let invalid = ContractClause {
            kind: ContractClauseKind::Requires,
            expression: "x +".to_string(),
        };
        assert!(KVerusContext.validate_contract_clause(&invalid).is_err());
    }
}
