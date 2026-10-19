use crate::{
    DEFAULT_ENGINE, KANI_SOURCE_COMMIT, ContractDiscovery, DiscoveryCandidate,
    DiscoveryDisposition, DiscoveryReport,
};
use cgp::prelude::*;
use kani_metadata::{
    AutoHarnessSkipReason, ContractClause, ContractClauseKind, KaniMetadata,
};
use proc_macro2::TokenStream;
use serde::Deserialize;
use std::str::FromStr;
use synthez::{ToTokens, quote::quote, syn};
use synthez::syn::parse::Parser as _;

#[derive(Deserialize)]
#[serde(untagged)]
enum MetadataEnvelope {
    One(KaniMetadata),
    Many(Vec<KaniMetadata>),
}

impl MetadataEnvelope {
    fn into_metadata_vec(self) -> Vec<KaniMetadata> {
        match self {
            Self::One(metadata) => vec![metadata],
            Self::Many(metadata) => metadata,
        }
    }
}

/// Tiny codegen wrapper used to exercise arcane-rs/synthez's safe `ToTokens`
/// derive at the validated syntax boundary.
#[derive(ToTokens)]
#[to_tokens(append(render))]
struct ValidatedClauseTokens {
    tokens: TokenStream,
}

impl ValidatedClauseTokens {
    fn render(&self) -> TokenStream {
        self.tokens.clone()
    }
}

#[cgp_component(MetadataParser)]
pub trait CanParseMetadataBytes {
    fn parse_metadata_bytes(
        &self,
        bytes: &[u8],
    ) -> Result<Vec<KaniMetadata>, serde_json::Error>;
}

#[cgp_impl(new JsonMetadataParser)]
impl MetadataParser {
    fn parse_metadata_bytes(
        &self,
        bytes: &[u8],
    ) -> Result<Vec<KaniMetadata>, serde_json::Error> {
        serde_json::from_slice::<MetadataEnvelope>(bytes)
            .map(MetadataEnvelope::into_metadata_vec)
    }
}

#[cgp_component(ContractClauseValidator)]
pub trait CanValidateContractClause {
    fn validate_contract_clause(&self, clause: &ContractClause) -> Result<(), String>;
}

#[cgp_impl(new ProcMacro2ContractClauseValidator)]
impl ContractClauseValidator {
    fn validate_contract_clause(&self, clause: &ContractClause) -> Result<(), String> {
        let tokens = TokenStream::from_str(&clause.expression).map_err(|error| {
            format!(
                "proc_macro2 rejected {:?} clause token stream {:?}: {error}",
                clause.kind, clause.expression
            )
        })?;

        match &clause.kind {
            ContractClauseKind::Requires | ContractClauseKind::Ensures => {
                syn::parse2::<syn::Expr>(tokens.clone()).map_err(|error| {
                    format!(
                        "synthez/syn rejected {:?} clause expression {:?}: {error}",
                        clause.kind, clause.expression
                    )
                })?;
            }
            ContractClauseKind::Modifies => {
                let parser =
                    syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
                parser.parse2(tokens.clone()).map_err(|error| {
                    format!(
                        "synthez/syn rejected modifies clause expression list {:?}: {error}",
                        clause.expression
                    )
                })?;
            }
        }

        // Render through synthez's proc-macro derive after Syn has accepted the
        // grammar. This is a syntax/codegen sanity check only: KVerus preserves
        // Kani's original clause string in its serialized evidence.
        let validated = ValidatedClauseTokens { tokens: tokens.clone() };
        let replay = quote!(#validated);
        if replay.is_empty() != tokens.is_empty() {
            return Err("synthez token replay changed clause emptiness".into());
        }
        Ok(())
    }
}

#[cgp_component(MetadataNormalizer)]
pub trait CanNormalizeMetadata {
    fn normalize_metadata(&self, metadata: &[KaniMetadata]) -> DiscoveryReport;
}

#[cgp_impl(new KaniMetadataNormalizer)]
#[uses(CanValidateContractClause)]
impl MetadataNormalizer {
    fn normalize_metadata(&self, metadata: &[KaniMetadata]) -> DiscoveryReport {
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
                for clause in &contract.clauses {
                    self.validate_contract_clause(clause).unwrap_or_else(|error| {
                        panic!(
                            "invalid compiler-produced Kani contract metadata for {}::{}: {error}",
                            md.crate_name, contract.function
                        )
                    });
                }

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
}

pub struct KVerusContext;

delegate_and_check_components! {
    KVerusContext {
        MetadataParserComponent:
            JsonMetadataParser,
        ContractClauseValidatorComponent:
            ProcMacro2ContractClauseValidator,
        MetadataNormalizerComponent:
            KaniMetadataNormalizer,
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
            format!(
                "Kani cannot construct unbounded arbitrary arguments: {}",
                format_args(args)
            ),
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
