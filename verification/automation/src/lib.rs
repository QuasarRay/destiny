#![forbid(unsafe_code)]

use kani_metadata::{AutoHarnessSkipReason, ContractClause, ContractClauseKind, KaniMetadata};
use serde::{Deserialize, Serialize};
use std::io::Read;

pub const DEFAULT_ENGINE: &str = "verus";
pub const KANI_SOURCE_COMMIT: &str = "0d2328a93f0e0ff66132d6bfa1a7d884877cf862";

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

#[derive(Deserialize)]
#[serde(untagged)]
enum MetadataEnvelope {
    One(KaniMetadata),
    Many(Vec<KaniMetadata>),
}

pub fn parse_metadata(mut reader: impl Read) -> Result<Vec<KaniMetadata>, serde_json::Error> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(serde_json::Error::io)?;
    match serde_json::from_slice::<MetadataEnvelope>(&bytes)? {
        MetadataEnvelope::One(md) => Ok(vec![md]),
        MetadataEnvelope::Many(md) => Ok(md),
    }
}

pub fn normalize_autoharness_metadata(metadata: &[KaniMetadata]) -> DiscoveryReport {
    let mut candidates = Vec::new();
    let mut contracted_functions = Vec::new();
    let mut crates_with_autoharness_metadata = 0;

    for md in metadata {
        if let Some(auto) = &md.autoharness_md {
            crates_with_autoharness_metadata += 1;

            for function in &auto.chosen {
                candidates.push(DiscoveryCandidate {
                    crate_name: md.crate_name.clone(),
                    function: function.clone(),
                    disposition: DiscoveryDisposition::KaniEligible,
                    reason: "Kani generated an automatic harness; Verus support must still be established by the capability registry".into(),
                    provenance: "kani_metadata::AutoHarnessMetadata::chosen",
                });
            }

            for (function, reason) in &auto.skipped {
                let (disposition, explanation) = classify_skip(reason);
                candidates.push(DiscoveryCandidate {
                    crate_name: md.crate_name.clone(),
                    function: function.clone(),
                    disposition,
                    reason: explanation,
                    provenance: "kani_metadata::AutoHarnessMetadata::skipped",
                });
            }
        }

        for contract in &md.contracted_functions {
            contracted_functions.push(ContractDiscovery {
                crate_name: md.crate_name.clone(),
                function: contract.function.clone(),
                file: contract.file.clone(),
                harnesses: contract.harnesses.clone(),
                clauses: contract.clauses.clone(),
                provenance: "kani_metadata::KaniMetadata::contracted_functions",
            });
        }
    }

    candidates.sort_by(|a, b| {
        (&a.crate_name, &a.function).cmp(&(&b.crate_name, &b.function))
    });
    contracted_functions.sort_by(|a, b| {
        (&a.crate_name, &a.function, &a.file, &a.harnesses, &a.clauses)
            .cmp(&(&b.crate_name, &b.function, &b.file, &b.harnesses, &b.clauses))
    });
    contracted_functions.dedup_by(|a, b| {
        a.crate_name == b.crate_name
            && a.function == b.function
            && a.file == b.file
            && a.harnesses == b.harnesses
            && a.clauses == b.clauses
    });

    DiscoveryReport {
        default_engine: DEFAULT_ENGINE,
        kani_source_commit: KANI_SOURCE_COMMIT,
        crates_with_autoharness_metadata,
        candidates,
        contracted_functions,
        proof_credit: false,
    }
}

fn classify_skip(reason: &AutoHarnessSkipReason) -> (DiscoveryDisposition, String) {
    match reason {
        AutoHarnessSkipReason::KaniImpl => (
            DiscoveryDisposition::Excluded,
            "Kani-internal implementation or existing harness".into(),
        ),
        AutoHarnessSkipReason::UserFilter => (
            DiscoveryDisposition::Excluded,
            "excluded by the user's autoharness filter".into(),
        ),
        AutoHarnessSkipReason::NoBody => (
            DiscoveryDisposition::Unsupported,
            "function has no body".into(),
        ),
        AutoHarnessSkipReason::GenericFn(detail) => (
            DiscoveryDisposition::Unsupported,
            format!("generic function is not automatically instantiable: {detail}"),
        ),
        AutoHarnessSkipReason::MissingArbitraryImpl(args) => (
            DiscoveryDisposition::Unsupported,
            format!("Kani cannot construct unbounded arbitrary arguments: {}", format_args(args)),
        ),
        AutoHarnessSkipReason::RequiresBoundedArguments(args) => (
            DiscoveryDisposition::Blocked,
            format!("requires explicit bounded-argument scope: {}", format_args(args)),
        ),
    }
}

fn format_args(args: &[(String, String)]) -> String {
    args.iter()
        .map(|(name, typ)| format!("{name}: {typ}"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use kani_metadata::{
        AutoHarnessMetadata, AutoHarnessSkipReason, ContractedFunction,
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
}
