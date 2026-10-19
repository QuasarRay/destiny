#![forbid(unsafe_code)]

mod components;
mod model;

pub use components::{
    CanBuildAutomationPlan, CanCanonicalizeContractClause, CanNormalizeMetadata,
    CanParseMetadataBytes, CanPrepareAutomation, CanReadWriteArtifacts, CanRunPythonTool,
    CanValidateContractClause, HasProjectRoot, KVerusContext,
};
pub use model::{
    AutomationPlan, ContractDiscovery, ContractPlan, DiscoveryCandidate,
    DiscoveryDisposition, DiscoveryReport, PlanCounts, PlannedContractClause,
    PreparationReceipt,
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
