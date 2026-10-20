use crate::{
    AutomationCommand, AutomationPlan, CampaignBindingSummary, CampaignExecutionSummary,
    CommandDescriptor,
    ContractDiscovery, ContractPlan, DEFAULT_ENGINE, DiscoveryCandidate,
    DiscoveryDisposition, DiscoveryReport, EvidenceArtifact, EvidenceArtifactKind,
    EvidenceArtifactSpec, KANI_SOURCE_COMMIT, PlanCounts, PlannedContractClause,
    PreparationReceipt, ReviewedBindingSummary, ReviewedEvidenceAuditReceipt,
    ReviewedCampaignExecutionReceipt, ReviewedCargoTarget, ReviewedEvidenceManifest,
    ReviewedFinalizationReceipt, ReviewedIntegrityArtifact, ReviewedIntegrityReport,
    ReviewedLifecycleAction, ReviewedLifecycleReport, ReviewedLifecycleStage,
    ReviewedExplanationReport, ReviewedPreparationReceipt, ReviewedVerificationConfig,
    ReviewedVerificationLimits, ReviewedVerificationReceipt, ReviewedVerificationSelection,
    ReviewedVerificationStep,
};
use cgp::prelude::*;
use comprende::c;
use kani_metadata::{
    AutoHarnessSkipReason, ContractClause, ContractClauseKind, KaniMetadata,
};
use proc_macro2::TokenStream;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command, ExitStatus, Output},
    str::FromStr,
};
use synthez::syn::parse::Parser as _;
use synthez::{quote::quote, syn, ToTokens};

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

/// Safe Arcane-rs codegen wrapper. `synthez::ToTokens` expands without
/// requiring KVerus to permit unsafe code.
#[derive(ToTokens)]
#[to_tokens(append(render))]
struct SafeTokenReplay {
    tokens: TokenStream,
}

impl SafeTokenReplay {
    fn render(&self) -> TokenStream {
        self.tokens.clone()
    }
}

#[cgp_component(TokenIdentity)]
pub trait CanCanonicalizeStaticIdentity {
    fn canonicalize_static_identity(&self, name: &str) -> Result<String, String>;
}

#[cgp_impl(new ProcMacro2TokenIdentity)]
impl TokenIdentity {
    fn canonicalize_static_identity(&self, name: &str) -> Result<String, String> {
        canonical_static_identity(name)
    }
}

fn canonical_static_identity(name: &str) -> Result<String, String> {
    let rust_name = name.replace('-', "_");
    let ident = syn::parse_str::<syn::Ident>(&rust_name)
        .map_err(|error| format!("invalid KVerus static identifier {name:?}: {error}"))?;
    let tokens = quote!(#ident);
    let replay = SafeTokenReplay { tokens };
    Ok(quote!(#replay).to_string())
}

#[cgp_component(CommandCatalog)]
pub trait CanDescribeCommands {
    fn command_descriptors(&self) -> Result<Vec<CommandDescriptor>, String>;
    fn command_usage(&self) -> Result<String, String>;
}

const COMMANDS: &[(&str, &str, &str)] = &[
    ("doctor", "doctor", "diagnostics"),
    ("setup", "setup", "bootstrap"),
    ("discover", "discover <kani-metadata.json>", "discovery"),
    ("list", "list <kani-metadata.json>", "discovery"),
    ("plan", "plan <kani-metadata.json>", "planning"),
    (
        "prepare",
        "prepare --manifest-path <Cargo.toml> --output-dir <dir>",
        "planning",
    ),
    (
        "prepare-reviewed",
        "prepare-reviewed --manifest-path <Cargo.toml> --output-dir <dir>",
        "reviewed_binding",
    ),
    (
        "verify-reviewed",
        "verify-reviewed --manifest-path <Cargo.toml> --reviewed-dir <dir> --campaign-dir <dir> --verus <path> [--config <json>]",
        "verification",
    ),
    (
        "run-reviewed",
        "run-reviewed --reviewed-dir <dir> --campaign-dir <dir> --verus <path>",
        "verification",
    ),
    (
        "finalize-reviewed",
        "finalize-reviewed --reviewed-dir <dir> --campaign-dir <dir>",
        "reviewed_binding",
    ),
    (
        "manifest-reviewed",
        "manifest-reviewed --reviewed-dir <dir>",
        "evidence",
    ),
    (
        "audit-reviewed",
        "audit-reviewed --reviewed-dir <dir>",
        "evidence",
    ),
    (
        "status-reviewed",
        "status-reviewed --reviewed-dir <dir>",
        "lifecycle",
    ),
    (
        "validate-reviewed",
        "validate-reviewed --reviewed-dir <dir>",
        "lifecycle",
    ),
    (
        "explain-reviewed",
        "explain-reviewed --reviewed-dir <dir>",
        "diagnostics",
    ),
    ("explain", "explain", "diagnostics"),
    ("verify", "verify <verus-subset args...>", "verification"),
    (
        "replay-kani-reference",
        "replay-kani-reference <replay args...>",
        "reference",
    ),
];

fn static_command_descriptors(
    context: &impl CanCanonicalizeStaticIdentity,
) -> Result<Vec<CommandDescriptor>, String> {
    Ok(c![
        CommandDescriptor {
            name: name.to_string(),
            synopsis: synopsis.to_string(),
            category: category.to_string(),
            canonical_token: context.canonicalize_static_identity(name)?,
            proof_credit: false,
        }
        for (name, synopsis, category) in COMMANDS.iter().copied()
    ])
}

fn static_command_usage(
    context: &impl CanCanonicalizeStaticIdentity,
) -> Result<String, String> {
    let commands = static_command_descriptors(context)?;
    Ok(format!(
        "usage: destiny-verification-automation {}",
        c![command.synopsis.as_str() for command in &commands].join(" | ")
    ))
}

#[cgp_impl(new StaticCommandCatalog)]
#[uses(CanCanonicalizeStaticIdentity)]
impl CommandCatalog {
    fn command_descriptors(&self) -> Result<Vec<CommandDescriptor>, String> {
        static_command_descriptors(self)
    }

    fn command_usage(&self) -> Result<String, String> {
        static_command_usage(self)
    }
}

#[cgp_component(CommandRouter)]
pub trait CanRouteCommand {
    fn route_command(&self, args: &[String]) -> Result<AutomationCommand, String>;
}

#[cgp_impl(new StaticCommandRouter)]
#[uses(CanDescribeCommands)]
impl CommandRouter {
    fn route_command(&self, args: &[String]) -> Result<AutomationCommand, String> {
        let Some(command) = args.first().map(String::as_str) else {
            return Err("missing command".into());
        };
        let rest = &args[1..];

        match command {
            "doctor" if rest.is_empty() => Ok(AutomationCommand::Doctor),
            "setup" if rest.is_empty() => Ok(AutomationCommand::Setup),
            "discover" | "list" if rest.len() == 1 => Ok(AutomationCommand::Discover {
                path: rest[0].clone(),
            }),
            "plan" if rest.len() == 1 => Ok(AutomationCommand::Plan {
                path: rest[0].clone(),
            }),
            "prepare" => {
                let (manifest_path, output_dir) =
                    parse_required_pair(rest, "--manifest-path", "--output-dir")?;
                Ok(AutomationCommand::Prepare {
                    manifest_path,
                    output_dir,
                })
            }
            "prepare-reviewed" => {
                let (manifest_path, output_dir) =
                    parse_required_pair(rest, "--manifest-path", "--output-dir")?;
                Ok(AutomationCommand::PrepareReviewed {
                    manifest_path,
                    output_dir,
                })
            }
            "verify-reviewed" => {
                let (values, config_path) = match rest.len() {
                    8 => (
                        parse_required_flags(
                            rest,
                            &["--manifest-path", "--reviewed-dir", "--campaign-dir", "--verus"],
                        )?,
                        None,
                    ),
                    10 => {
                        let values = parse_required_flags(
                            rest,
                            &[
                                "--manifest-path",
                                "--reviewed-dir",
                                "--campaign-dir",
                                "--verus",
                                "--config",
                            ],
                        )?;
                        (values[..4].to_vec(), Some(values[4].clone()))
                    }
                    _ => {
                        return Err(
                            "verify-reviewed expects four required named arguments and optional --config <json>"
                                .into(),
                        )
                    }
                };
                Ok(AutomationCommand::VerifyReviewed {
                    manifest_path: values[0].clone(),
                    reviewed_dir: values[1].clone(),
                    campaign_dir: values[2].clone(),
                    verus: values[3].clone(),
                    config_path,
                })
            }
            "run-reviewed" => {
                let values = parse_required_flags(
                    rest,
                    &["--reviewed-dir", "--campaign-dir", "--verus"],
                )?;
                Ok(AutomationCommand::RunReviewed {
                    reviewed_dir: values[0].clone(),
                    campaign_dir: values[1].clone(),
                    verus: values[2].clone(),
                })
            }
            "finalize-reviewed" => {
                let (reviewed_dir, campaign_dir) =
                    parse_required_pair(rest, "--reviewed-dir", "--campaign-dir")?;
                Ok(AutomationCommand::FinalizeReviewed {
                    reviewed_dir,
                    campaign_dir,
                })
            }
            "manifest-reviewed" => Ok(AutomationCommand::ManifestReviewed {
                reviewed_dir: parse_required_value(rest, "--reviewed-dir")?,
            }),
            "audit-reviewed" => Ok(AutomationCommand::AuditReviewed {
                reviewed_dir: parse_required_value(rest, "--reviewed-dir")?,
            }),
            "status-reviewed" => Ok(AutomationCommand::StatusReviewed {
                reviewed_dir: parse_required_value(rest, "--reviewed-dir")?,
            }),
            "validate-reviewed" => Ok(AutomationCommand::ValidateReviewed {
                reviewed_dir: parse_required_value(rest, "--reviewed-dir")?,
            }),
            "explain-reviewed" => Ok(AutomationCommand::ExplainReviewed {
                reviewed_dir: parse_required_value(rest, "--reviewed-dir")?,
            }),
            "explain" if rest.is_empty() => Ok(AutomationCommand::Explain),
            "verify" => Ok(AutomationCommand::Verify {
                args: rest.to_vec(),
            }),
            "replay-kani-reference" => Ok(AutomationCommand::ReplayKaniReference {
                args: rest.to_vec(),
            }),
            _ => Err(format!(
                "invalid KVerus command invocation; {}",
                self.command_usage()?
            )),
        }
    }
}

fn parse_required_flags(args: &[String], flags: &[&str]) -> Result<Vec<String>, String> {
    if args.len() != flags.len() * 2 {
        return Err(format!(
            "expected exactly {} named arguments: {}",
            flags.len(),
            flags.join(", ")
        ));
    }

    let mut values = vec![None; flags.len()];
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?
            .clone();
        let Some(position) = flags.iter().position(|candidate| *candidate == flag) else {
            return Err(format!("unexpected argument {flag}"));
        };
        if values[position].replace(value).is_some() {
            return Err(format!("duplicate argument {flag}"));
        }
        index += 2;
    }

    flags
        .iter()
        .zip(values)
        .map(|(flag, value)| value.ok_or_else(|| format!("missing required argument {flag}")))
        .collect()
}

fn parse_required_value(args: &[String], flag: &str) -> Result<String, String> {
    Ok(parse_required_flags(args, &[flag])?.remove(0))
}

fn parse_required_pair(
    args: &[String],
    first_flag: &str,
    second_flag: &str,
) -> Result<(String, String), String> {
    let values = parse_required_flags(args, &[first_flag, second_flag])?;
    Ok((values[0].clone(), values[1].clone()))
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

#[cgp_component(ContractClauseSyntax)]
pub trait CanCanonicalizeContractClause {
    fn canonicalize_contract_clause(
        &self,
        clause: &ContractClause,
    ) -> Result<String, String>;
}

#[cgp_impl(new ProcMacro2ContractClauseSyntax)]
impl ContractClauseSyntax {
    fn canonicalize_contract_clause(
        &self,
        clause: &ContractClause,
    ) -> Result<String, String> {
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

        let validated = SafeTokenReplay {
            tokens: tokens.clone(),
        };
        let replay = quote!(#validated);
        if replay.is_empty() != tokens.is_empty() {
            return Err("synthez token replay changed clause emptiness".into());
        }
        Ok(replay.to_string())
    }
}

#[cgp_component(ContractClauseValidator)]
pub trait CanValidateContractClause {
    fn validate_contract_clause(&self, clause: &ContractClause) -> Result<(), String>;
}

#[cgp_impl(new CanonicalContractClauseValidator)]
#[uses(CanCanonicalizeContractClause)]
impl ContractClauseValidator {
    fn validate_contract_clause(&self, clause: &ContractClause) -> Result<(), String> {
        self.canonicalize_contract_clause(clause).map(|_| ())
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

#[cgp_component(AutomationPlanner)]
pub trait CanBuildAutomationPlan {
    fn build_automation_plan(
        &self,
        report: &DiscoveryReport,
    ) -> Result<AutomationPlan, String>;
}

#[cgp_impl(new KaniAutomationPlanner)]
#[uses(CanCanonicalizeContractClause)]
impl AutomationPlanner {
    fn build_automation_plan(
        &self,
        report: &DiscoveryReport,
    ) -> Result<AutomationPlan, String> {
        let selected = c![candidate.clone() for candidate in &report.candidates if candidate.disposition == DiscoveryDisposition::KaniEligible];
        let blocked = c![candidate.clone() for candidate in &report.candidates if candidate.disposition == DiscoveryDisposition::Blocked];
        let unsupported = c![candidate.clone() for candidate in &report.candidates if candidate.disposition == DiscoveryDisposition::Unsupported];
        let excluded = c![candidate.clone() for candidate in &report.candidates if candidate.disposition == DiscoveryDisposition::Excluded];

        let mut contracted_functions = Vec::new();
        let mut contract_clause_count = 0;

        for contract in &report.contracted_functions {
            let mut has_modifies = false;
            let mut clauses = Vec::new();

            for clause in &contract.clauses {
                let kind = contract_clause_kind_name(&clause.kind);
                has_modifies |= matches!(clause.kind, ContractClauseKind::Modifies);
                clauses.push(PlannedContractClause {
                    kind,
                    expression: clause.expression.clone(),
                    canonical_tokens: self.canonicalize_contract_clause(clause)?,
                });
            }

            contract_clause_count += clauses.len();
            contracted_functions.push(ContractPlan {
                crate_name: contract.crate_name.clone(),
                function: contract.function.clone(),
                file: contract.file.clone(),
                harnesses: contract.harnesses.clone(),
                clauses,
                planner_status: if has_modifies {
                    "blocked_by_modifies_semantics"
                } else {
                    "downstream_subset_check_required"
                },
            });
        }

        Ok(AutomationPlan {
            schema: 1,
            default_engine: report.default_engine,
            kani_source_commit: report.kani_source_commit,
            proof_credit: false,
            counts: PlanCounts {
                selected: selected.len(),
                blocked: blocked.len(),
                unsupported: unsupported.len(),
                excluded: excluded.len(),
                contracted_functions: contracted_functions.len(),
                contract_clauses: contract_clause_count,
            },
            selected,
            blocked,
            unsupported,
            excluded,
            contracted_functions,
        })
    }
}

fn validate_unique_nonempty(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(format!("{label} entries must be non-empty"));
        }
        if !seen.insert(value) {
            return Err(format!("{label} contains duplicate entry {value:?}"));
        }
    }
    Ok(())
}

#[cgp_component(ReviewedVerificationConfigValidation)]
pub trait CanValidateReviewedVerificationConfig {
    fn validate_reviewed_verification_config(
        &self,
        config: ReviewedVerificationConfig,
    ) -> Result<ReviewedVerificationConfig, String>;
}

#[cgp_impl(new StrictReviewedVerificationConfigValidation)]
impl ReviewedVerificationConfigValidation {
    fn validate_reviewed_verification_config(
        &self,
        config: ReviewedVerificationConfig,
    ) -> Result<ReviewedVerificationConfig, String> {
        if config.schema != 1 {
            return Err(format!(
                "unsupported reviewed verification config schema {}",
                config.schema
            ));
        }
        validate_unique_nonempty(&config.selection.packages, "packages")?;
        validate_unique_nonempty(&config.selection.features, "features")?;
        validate_unique_nonempty(&config.selection.functions, "functions")?;
        if config.selection.all_features
            && (!config.selection.features.is_empty() || config.selection.no_default_features)
        {
            return Err(
                "all_features cannot be combined with explicit features or no_default_features"
                    .into(),
            );
        }
        match (
            config.selection.target.requires_name(),
            config.selection.target_name.as_deref(),
        ) {
            (true, Some(name)) if !name.trim().is_empty() => {}
            (true, _) => return Err("target=bin requires non-empty target_name".into()),
            (false, None) => {}
            (false, Some(_)) => {
                return Err("target_name is only valid when target=bin".into());
            }
        }
        if !(1..=120).contains(&config.limits.timeout_seconds) {
            return Err(
                "timeout_seconds must be between 1 and the maintained 120-second scoped Verus limit"
                    .into(),
            );
        }
        if matches!(config.limits.memory_mib, Some(0)) {
            return Err("memory_mib must be positive when specified".into());
        }
        if config.limits.max_concurrency != 1 {
            return Err(
                "max_concurrency must currently be 1 because the maintained Destiny controller serializes proof execution"
                    .into(),
            );
        }
        Ok(config)
    }
}

#[cgp_component(ReviewedVerificationConfigStore)]
pub trait CanLoadReviewedVerificationConfig {
    fn load_reviewed_verification_config(
        &self,
        path: &str,
    ) -> Result<ReviewedVerificationConfig, String>;
}

#[cgp_impl(new JsonReviewedVerificationConfigStore)]
#[uses(CanReadWriteArtifacts, CanValidateReviewedVerificationConfig)]
impl ReviewedVerificationConfigStore {
    fn load_reviewed_verification_config(
        &self,
        path: &str,
    ) -> Result<ReviewedVerificationConfig, String> {
        let bytes = self
            .read_artifact(Path::new(path))
            .map_err(|error| format!("cannot read reviewed verification config {path}: {error}"))?;
        let config: ReviewedVerificationConfig = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid reviewed verification config {path}: {error}"))?;
        self.validate_reviewed_verification_config(config)
    }
}

#[cgp_component(DiscoverySelectionArguments)]
pub trait CanBuildDiscoverySelectionArguments {
    fn discovery_selection_args(
        &self,
        selection: &ReviewedVerificationSelection,
    ) -> Result<Vec<String>, String>;
}

#[cgp_impl(new KaniDiscoverySelectionArguments)]
impl DiscoverySelectionArguments {
    fn discovery_selection_args(
        &self,
        selection: &ReviewedVerificationSelection,
    ) -> Result<Vec<String>, String> {
        let mut args = Vec::new();
        args.extend(
            c![vec!["--package".to_string(), package.clone()] for package in &selection.packages]
                .into_iter()
                .flatten(),
        );
        args.extend(
            c![vec!["--feature".to_string(), feature.clone()] for feature in &selection.features]
                .into_iter()
                .flatten(),
        );
        if selection.all_features {
            args.push("--all-features".to_string());
        }
        if selection.no_default_features {
            args.push("--no-default-features".to_string());
        }
        if let Some(flag) = selection.target.kani_flag() {
            args.push("--target-kind".to_string());
            args.push(flag.trim_start_matches("--").to_string());
            if selection.target.requires_name() {
                args.push("--target-name".to_string());
                args.push(
                    selection
                        .target_name
                        .clone()
                        .ok_or_else(|| "target=bin requires target_name".to_string())?,
                );
            }
        }
        args.extend(
            c![vec!["--function".to_string(), function.clone()] for function in &selection.functions]
                .into_iter()
                .flatten(),
        );
        Ok(args)
    }
}

#[cgp_component(ProjectLayout)]
pub trait HasProjectRoot {
    fn project_root(&self) -> PathBuf;
}

#[cgp_impl(new CargoManifestProjectLayout)]
impl ProjectLayout {
    fn project_root(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }
}

#[cgp_component(PythonToolRunner)]
pub trait CanRunPythonTool {
    fn run_python_tool(
        &self,
        script: &str,
        args: &[String],
    ) -> Result<ExitStatus, io::Error>;

    fn run_python_tool_output(
        &self,
        script: &str,
        args: &[String],
    ) -> Result<Output, io::Error>;
}

#[cgp_impl(new SystemPythonToolRunner)]
#[uses(HasProjectRoot)]
impl PythonToolRunner {
    fn run_python_tool(
        &self,
        script: &str,
        args: &[String],
    ) -> Result<ExitStatus, io::Error> {
        python_command(self, script, args).status()
    }

    fn run_python_tool_output(
        &self,
        script: &str,
        args: &[String],
    ) -> Result<Output, io::Error> {
        python_command(self, script, args).output()
    }
}

#[cgp_component(JsonPythonToolRunner)]
pub trait CanRunJsonPythonTool {
    fn run_python_json(
        &self,
        script: &str,
        args: &[String],
    ) -> Result<serde_json::Value, String>;
}

#[cgp_impl(new CapturedJsonPythonToolRunner)]
#[uses(CanRunPythonTool)]
impl JsonPythonToolRunner {
    fn run_python_json(
        &self,
        script: &str,
        args: &[String],
    ) -> Result<serde_json::Value, String> {
        let output = self
            .run_python_tool_output(script, args)
            .map_err(|error| format!("failed to execute {script}: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "{script} failed with {}: stdout={} stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        serde_json::from_slice(&output.stdout).map_err(|error| {
            format!(
                "{script} emitted invalid JSON: {error}; stdout={}",
                String::from_utf8_lossy(&output.stdout).trim()
            )
        })
    }
}

fn python_command(
    context: &impl HasProjectRoot,
    script: &str,
    args: &[String],
) -> Command {
    let mut command = Command::new(python_executable());
    command
        .arg("-B")
        .arg(context.project_root().join(script))
        .args(args);
    command
}

#[cgp_component(ArtifactStore)]
pub trait CanReadWriteArtifacts {
    fn read_artifact(&self, path: &Path) -> Result<Vec<u8>, io::Error>;
    fn write_artifact(&self, path: &Path, bytes: &[u8]) -> Result<(), io::Error>;
}

#[cgp_impl(new StdArtifactStore)]
impl ArtifactStore {
    fn read_artifact(&self, path: &Path) -> Result<Vec<u8>, io::Error> {
        fs::read(path)
    }

    fn write_artifact(&self, path: &Path, bytes: &[u8]) -> Result<(), io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file_name = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "artifact path has no file name"))?
            .to_string_lossy();
        let temporary = path.with_file_name(format!(
            ".{file_name}.{}.kverus-tmp",
            process::id()
        ));
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);

        match fs::rename(&temporary, path) {
            Ok(()) => Ok(()),
            Err(_) if path.exists() => {
                fs::remove_file(path)?;
                fs::rename(&temporary, path)
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                Err(error)
            }
        }
    }
}

#[cgp_component(ArtifactPresence)]
pub trait CanInspectArtifactPresence {
    fn artifact_exists(&self, path: &Path) -> Result<bool, String>;
}

#[cgp_impl(new StdArtifactPresence)]
impl ArtifactPresence {
    fn artifact_exists(&self, path: &Path) -> Result<bool, String> {
        match fs::metadata(path) {
            Ok(metadata) => Ok(metadata.is_file()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(format!("cannot inspect {}: {error}", path.display())),
        }
    }
}

const REVIEWED_LIFECYCLE_ARTIFACTS: &[&str] = &[
    "reviewed-preparation-receipt.json",
    "verus-binding.json",
    "verus-receipt-binding.json",
    "reviewed-finalization-receipt.json",
    "reviewed-evidence-manifest.json",
    "reviewed-evidence-audit.json",
];

#[cgp_component(ReviewedLifecycleInspection)]
pub trait CanInspectReviewedLifecycle {
    fn reviewed_lifecycle(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedLifecycleReport, String>;
}

#[cgp_impl(new ArtifactBackedReviewedLifecycleInspection)]
#[uses(CanInspectArtifactPresence, CanCanonicalizeStaticIdentity)]
impl ReviewedLifecycleInspection {
    fn reviewed_lifecycle(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedLifecycleReport, String> {
        let root = PathBuf::from(reviewed_dir);
        let mut presence = [false; 6];
        let mut present_artifacts = Vec::new();
        let mut missing_artifacts = Vec::new();

        for (index, filename) in REVIEWED_LIFECYCLE_ARTIFACTS.iter().enumerate() {
            if self.artifact_exists(&root.join(filename))? {
                presence[index] = true;
                present_artifacts.push((*filename).to_string());
            } else {
                missing_artifacts.push((*filename).to_string());
            }
        }

        let stage = match presence {
            [false, false, false, false, false, false] => ReviewedLifecycleStage::Empty,
            [true, true, false, false, false, false] => ReviewedLifecycleStage::Prepared,
            [true, true, true, true, false, false] => ReviewedLifecycleStage::Finalized,
            [true, true, true, true, true, false] => ReviewedLifecycleStage::Manifested,
            [true, true, true, true, true, true] => ReviewedLifecycleStage::Audited,
            _ => ReviewedLifecycleStage::Inconsistent,
        };
        let stage_name = stage.kverus_name();

        Ok(ReviewedLifecycleReport {
            schema: 1,
            status: "KVERUS_REVIEWED_LIFECYCLE",
            reviewed_dir: reviewed_dir.to_string(),
            stage: stage_name,
            canonical_stage: self.canonicalize_static_identity(stage_name)?,
            consistent: stage != ReviewedLifecycleStage::Inconsistent,
            present_artifacts,
            missing_artifacts,
            legal_next_actions: reviewed_lifecycle_legal_actions(stage),
            proof_credit: false,
        })
    }
}

#[cgp_component(ReviewedLifecycleTransition)]
pub trait CanRequireReviewedTransition {
    fn require_reviewed_action(
        &self,
        reviewed_dir: &str,
        action: ReviewedLifecycleAction,
    ) -> Result<ReviewedLifecycleReport, String>;
}

#[cgp_impl(new StrictReviewedLifecycleTransition)]
#[uses(CanInspectReviewedLifecycle)]
impl ReviewedLifecycleTransition {
    fn require_reviewed_action(
        &self,
        reviewed_dir: &str,
        action: ReviewedLifecycleAction,
    ) -> Result<ReviewedLifecycleReport, String> {
        let report = self.reviewed_lifecycle(reviewed_dir)?;
        let stage = ReviewedLifecycleStage::from_kverus_name(report.stage)
            .unwrap_or(ReviewedLifecycleStage::Inconsistent);
        let allowed = action.allowed_stages().contains(&stage);

        if !report.consistent || !allowed {
            return Err(format!(
                "reviewed lifecycle rejects {} from stage {} (present: {}; missing: {})",
                action.kverus_name(),
                report.stage,
                report.present_artifacts.join(", "),
                report.missing_artifacts.join(", "),
            ));
        }

        Ok(report)
    }
}

fn reviewed_lifecycle_legal_actions(stage: ReviewedLifecycleStage) -> Vec<&'static str> {
    c![
        action.kverus_name()
        for action in ReviewedLifecycleAction::ALL.iter().copied()
        if action.allowed_stages().contains(&stage)
    ]
}

#[cgp_component(ArtifactDigest)]
pub trait CanDigestArtifactBytes {
    fn digest_artifact_bytes(&self, bytes: &[u8]) -> String;
}

#[cgp_impl(new Sha256ArtifactDigest)]
impl ArtifactDigest {
    fn digest_artifact_bytes(&self, bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        format!("{digest:x}")
    }
}

#[cgp_component(EvidenceArtifactPolicy)]
pub trait CanDescribeEvidenceArtifacts {
    fn reviewed_evidence_artifact_specs(&self) -> Vec<EvidenceArtifactSpec>;
    fn evidence_artifact_kind_name(&self, kind: EvidenceArtifactKind) -> &'static str;
}

#[cgp_impl(new StaticEvidenceArtifactPolicy)]
impl EvidenceArtifactPolicy {
    fn reviewed_evidence_artifact_specs(&self) -> Vec<EvidenceArtifactSpec> {
        c![
            EvidenceArtifactSpec {
                kind,
                filename: kind.filename(),
                status: kind.status(),
                proof_credit_source: kind.proof_credit_source(),
            }
            for kind in EvidenceArtifactKind::ALL.iter().copied()
        ]
    }

    fn evidence_artifact_kind_name(&self, kind: EvidenceArtifactKind) -> &'static str {
        kind.kverus_name()
    }
}

#[cgp_component(JsonArtifactStore)]
pub trait CanReadJsonArtifact {
    fn read_json_artifact(&self, path: &Path) -> Result<serde_json::Value, String>;
}

#[cgp_impl(new SerdeJsonArtifactStore)]
#[uses(CanReadWriteArtifacts)]
impl JsonArtifactStore {
    fn read_json_artifact(&self, path: &Path) -> Result<serde_json::Value, String> {
        let bytes = self
            .read_artifact(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))
    }
}

#[cgp_component(JsonReceiptDecoder)]
pub trait CanDecodeJsonReceipt {
    fn decode_json_receipt<T>(
        &self,
        value: serde_json::Value,
        label: &str,
    ) -> Result<T, String>
    where
        T: DeserializeOwned;
}

#[cgp_impl(new SerdeJsonReceiptDecoder)]
impl JsonReceiptDecoder {
    fn decode_json_receipt<T>(
        &self,
        value: serde_json::Value,
        label: &str,
    ) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(value)
            .map_err(|error| format!("invalid {label}: {error}"))
    }
}

#[derive(Deserialize)]
struct PreparationCountsArtifact {
    contracted_functions: usize,
    contract_clauses: usize,
}

#[derive(Deserialize)]
struct PreparationArtifact {
    schema: u32,
    status: String,
    default_engine: String,
    kani_source_commit: String,
    proof_credit: bool,
    manifest_path: String,
    output_dir: String,
    metadata_path: String,
    discovery_path: String,
    plan_path: String,
    selection: ReviewedVerificationSelection,
    counts: PreparationCountsArtifact,
}

#[derive(Deserialize)]
struct ReviewedPreparationArtifact {
    schema: u32,
    status: String,
    default_engine: String,
    proof_credit: bool,
    verus_campaign_checked: bool,
    preparation: PreparationArtifact,
    binding_path: String,
    binding_status: String,
    selected_count: usize,
    unresolved_count: usize,
    controller_obligation_count: usize,
    contract_inventory_count: usize,
    contract_clause_count: usize,
}

#[derive(Deserialize)]
struct ReviewedFinalizationArtifact {
    schema: u32,
    status: String,
    engine: String,
    kani_proof_credit: bool,
    verus_checked_scoped: bool,
    reviewed_dir: String,
    binding_path: String,
    campaign_binding_path: String,
    campaign_binding_status: String,
    checked_receipt_count: usize,
    contract_inventory_count: usize,
    contract_clause_count: usize,
    translation_correctness: String,
}

#[derive(Deserialize)]
struct ReviewedEvidenceAuditArtifact {
    schema: u32,
    status: String,
    engine: String,
    kani_proof_credit: bool,
    verus_checked_scoped: bool,
    reviewed_dir: String,
    manifest_path: String,
    manifest_digest_algorithm: String,
    manifest_digest: String,
    verified_artifact_count: usize,
    proof_credit_source: String,
}

fn validate_reviewed_preparation_artifact(
    receipt: &ReviewedPreparationArtifact,
    binding: &ReviewedBindingSummary,
    binding_path: &Path,
    reviewed_dir: &str,
) -> Result<(), String> {
    let expected_binding_path = binding_path.display().to_string();
    let root = PathBuf::from(reviewed_dir);
    let preparation = &receipt.preparation;
    let preparation_valid = preparation.schema == 1
        && preparation.status == "KVERUS_PREPARED"
        && preparation.default_engine == DEFAULT_ENGINE
        && preparation.kani_source_commit == KANI_SOURCE_COMMIT
        && !preparation.proof_credit
        && !preparation.manifest_path.is_empty()
        && preparation.output_dir == reviewed_dir
        && preparation.metadata_path == root.join("kani-metadata.json").display().to_string()
        && preparation.discovery_path == root.join("normalized-discovery.json").display().to_string()
        && preparation.plan_path == root.join("automation-plan.json").display().to_string()
        && preparation.counts.contracted_functions == receipt.contract_inventory_count
        && preparation.counts.contract_clauses == receipt.contract_clause_count;

    if !preparation_valid
        || receipt.schema != 1
        || receipt.status != "KVERUS_REVIEWED_BINDING_PREPARED"
        || receipt.default_engine != DEFAULT_ENGINE
        || receipt.proof_credit
        || receipt.verus_campaign_checked
        || receipt.binding_path != expected_binding_path
        || receipt.binding_status != binding.status
        || receipt.unresolved_count != 0
        || receipt.selected_count != binding.selected_count
        || receipt.controller_obligation_count != binding.controller_obligation_count
        || receipt.contract_inventory_count != binding.contract_inventory_count
        || receipt.contract_clause_count != binding.contract_clause_count
    {
        return Err(format!(
            "reviewed preparation receipt violated KVerus invariants:              schema={}, status={}, engine={}, proof_credit={}, campaign_checked={},              binding_path={}, binding_status={}, selected={}, unresolved={},              controller_obligations={}, contract_inventory={}, contract_clauses={}",
            receipt.schema,
            receipt.status,
            receipt.default_engine,
            receipt.proof_credit,
            receipt.verus_campaign_checked,
            receipt.binding_path,
            receipt.binding_status,
            receipt.selected_count,
            receipt.unresolved_count,
            receipt.controller_obligation_count,
            receipt.contract_inventory_count,
            receipt.contract_clause_count
        ));
    }
    Ok(())
}

fn validate_reviewed_finalization_artifact(
    receipt: &ReviewedFinalizationArtifact,
    reviewed_dir: &str,
    binding_path: &Path,
    campaign_binding_path: &Path,
    campaign: &CampaignBindingSummary,
) -> Result<(), String> {
    let expected_binding_path = binding_path.display().to_string();
    let expected_campaign_binding_path = campaign_binding_path.display().to_string();
    if receipt.schema != 1
        || receipt.status != "KVERUS_REVIEWED_VERUS_FINALIZED"
        || receipt.engine != DEFAULT_ENGINE
        || receipt.kani_proof_credit
        || !receipt.verus_checked_scoped
        || receipt.reviewed_dir != reviewed_dir
        || receipt.binding_path != expected_binding_path
        || receipt.campaign_binding_path != expected_campaign_binding_path
        || receipt.campaign_binding_status != campaign.status
        || receipt.checked_receipt_count != campaign.checked_receipt_count
        || receipt.contract_inventory_count != campaign.contract_inventory_count
        || receipt.contract_clause_count != campaign.contract_clause_count
        || receipt.translation_correctness != campaign.translation_correctness
    {
        return Err(format!(
            "reviewed finalization receipt violated KVerus invariants:              schema={}, status={}, engine={}, kani_proof_credit={}, verus_checked_scoped={},              reviewed_dir={}, binding_path={}, campaign_binding_path={},              campaign_binding_status={}, checked_receipts={}, contract_inventory={},              contract_clauses={}, translation_correctness={}",
            receipt.schema,
            receipt.status,
            receipt.engine,
            receipt.kani_proof_credit,
            receipt.verus_checked_scoped,
            receipt.reviewed_dir,
            receipt.binding_path,
            receipt.campaign_binding_path,
            receipt.campaign_binding_status,
            receipt.checked_receipt_count,
            receipt.contract_inventory_count,
            receipt.contract_clause_count,
            receipt.translation_correctness
        ));
    }
    Ok(())
}

fn validate_reviewed_audit_artifact(
    receipt: &ReviewedEvidenceAuditArtifact,
    reviewed_dir: &str,
    manifest_path: &Path,
    manifest_digest: &str,
    verified_artifact_count: usize,
) -> Result<(), String> {
    let expected_manifest_path = manifest_path.display().to_string();
    if receipt.schema != 1
        || receipt.status != "KVERUS_REVIEWED_EVIDENCE_AUDITED"
        || receipt.engine != DEFAULT_ENGINE
        || receipt.kani_proof_credit
        || !receipt.verus_checked_scoped
        || receipt.reviewed_dir != reviewed_dir
        || receipt.manifest_path != expected_manifest_path
        || receipt.manifest_digest_algorithm != "sha256"
        || receipt.manifest_digest != manifest_digest
        || receipt.verified_artifact_count != verified_artifact_count
        || receipt.proof_credit_source != "existing_verus_checked_scoped_receipts_only"
    {
        return Err(format!(
            "reviewed audit receipt violated KVerus invariants:              schema={}, status={}, engine={}, kani_proof_credit={}, verus_checked_scoped={},              reviewed_dir={}, manifest_path={}, digest_algorithm={}, verified_artifacts={},              proof_credit_source={}",
            receipt.schema,
            receipt.status,
            receipt.engine,
            receipt.kani_proof_credit,
            receipt.verus_checked_scoped,
            receipt.reviewed_dir,
            receipt.manifest_path,
            receipt.manifest_digest_algorithm,
            receipt.verified_artifact_count,
            receipt.proof_credit_source
        ));
    }
    Ok(())
}

#[cgp_component(AutomationPreparation)]
pub trait CanPrepareAutomation {
    fn prepare_automation(
        &self,
        manifest_path: &str,
        output_dir: &str,
    ) -> Result<PreparationReceipt, String>;

    fn prepare_automation_with_selection(
        &self,
        manifest_path: &str,
        output_dir: &str,
        selection: &ReviewedVerificationSelection,
    ) -> Result<PreparationReceipt, String>;
}

#[cgp_impl(new KaniAutomationPreparation)]
#[uses(
    CanRunPythonTool,
    CanReadWriteArtifacts,
    CanBuildDiscoverySelectionArguments,
    CanParseMetadataBytes,
    CanNormalizeMetadata,
    CanBuildAutomationPlan
)]
impl AutomationPreparation {
    fn prepare_automation(
        &self,
        manifest_path: &str,
        output_dir: &str,
    ) -> Result<PreparationReceipt, String> {
        self.prepare_automation_with_selection(
            manifest_path,
            output_dir,
            &ReviewedVerificationSelection::default(),
        )
    }

    fn prepare_automation_with_selection(
        &self,
        manifest_path: &str,
        output_dir: &str,
        selection: &ReviewedVerificationSelection,
    ) -> Result<PreparationReceipt, String> {
        let mut args = vec![
            "--manifest-path".to_string(),
            manifest_path.to_string(),
            "--output-dir".to_string(),
            output_dir.to_string(),
        ];
        args.extend(self.discovery_selection_args(selection)?);
        let output = self
            .run_python_tool_output(
                "verification/tools/run_kani_autoharness_discovery.py",
                &args,
            )
            .map_err(|error| format!("failed to execute Kani discovery: {error}"))?;

        if !output.status.success() {
            return Err(format!(
                "Kani discovery failed with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let output_dir_path = PathBuf::from(output_dir);
        let metadata_path = output_dir_path.join("kani-metadata.json");
        let discovery_path = output_dir_path.join("normalized-discovery.json");
        let plan_path = output_dir_path.join("automation-plan.json");
        let receipt_path = output_dir_path.join("preparation-receipt.json");

        let metadata_bytes = self
            .read_artifact(&metadata_path)
            .map_err(|error| format!("cannot read {}: {error}", metadata_path.display()))?;
        let metadata = self
            .parse_metadata_bytes(&metadata_bytes)
            .map_err(|error| format!("cannot parse {}: {error}", metadata_path.display()))?;
        let discovery = self.normalize_metadata(&metadata);
        let plan = self.build_automation_plan(&discovery)?;

        self.write_artifact(&discovery_path, &pretty_json_bytes(&discovery)?)
            .map_err(|error| format!("cannot write {}: {error}", discovery_path.display()))?;
        self.write_artifact(&plan_path, &pretty_json_bytes(&plan)?)
            .map_err(|error| format!("cannot write {}: {error}", plan_path.display()))?;

        let receipt = PreparationReceipt {
            schema: 1,
            status: "KVERUS_PREPARED",
            default_engine: discovery.default_engine,
            kani_source_commit: discovery.kani_source_commit,
            proof_credit: false,
            manifest_path: manifest_path.to_string(),
            output_dir: output_dir.to_string(),
            metadata_path: metadata_path.display().to_string(),
            discovery_path: discovery_path.display().to_string(),
            plan_path: plan_path.display().to_string(),
            selection: selection.clone(),
            counts: plan.counts.clone(),
        };
        self.write_artifact(&receipt_path, &pretty_json_bytes(&receipt)?)
            .map_err(|error| format!("cannot write {}: {error}", receipt_path.display()))?;
        Ok(receipt)
    }
}

#[derive(Deserialize)]
struct ReviewedBindingReceipt {
    schema: u32,
    status: String,
    default_engine: String,
    kani_proof_credit: bool,
    selected: Vec<serde_json::Value>,
    controller_obligations: Vec<String>,
    contract_inventory_count: usize,
    contract_clause_count: usize,
    unresolved: Vec<serde_json::Value>,
}

#[cgp_component(ReviewedBindingValidator)]
pub trait CanValidateReviewedBinding {
    fn validate_reviewed_binding(
        &self,
        value: serde_json::Value,
    ) -> Result<ReviewedBindingSummary, String>;
}

#[cgp_impl(new StrictReviewedBindingValidator)]
#[uses(CanDecodeJsonReceipt)]
impl ReviewedBindingValidator {
    fn validate_reviewed_binding(
        &self,
        value: serde_json::Value,
    ) -> Result<ReviewedBindingSummary, String> {
        let binding: ReviewedBindingReceipt =
            self.decode_json_receipt(value, "reviewed binder receipt")?;

        if binding.schema != 1
            || binding.status != "BOUND_FOR_VERUS"
            || binding.default_engine != DEFAULT_ENGINE
            || binding.kani_proof_credit
            || binding.selected.is_empty()
            || !binding.unresolved.is_empty()
        {
            return Err(format!(
                "reviewed binder receipt violated KVerus invariants:                  schema={}, status={}, engine={}, kani_proof_credit={}, selected={}, unresolved={}",
                binding.schema,
                binding.status,
                binding.default_engine,
                binding.kani_proof_credit,
                binding.selected.len(),
                binding.unresolved.len()
            ));
        }

        Ok(ReviewedBindingSummary {
            status: binding.status,
            selected_count: binding.selected.len(),
            controller_obligations: binding.controller_obligations.clone(),
            controller_obligation_count: binding.controller_obligations.len(),
            contract_inventory_count: binding.contract_inventory_count,
            contract_clause_count: binding.contract_clause_count,
        })
    }
}

#[cgp_component(ReviewedBindingPreparation)]
pub trait CanPrepareReviewedBinding {
    fn prepare_reviewed_binding(
        &self,
        manifest_path: &str,
        output_dir: &str,
    ) -> Result<ReviewedPreparationReceipt, String>;

    fn prepare_reviewed_binding_with_selection(
        &self,
        manifest_path: &str,
        output_dir: &str,
        selection: &ReviewedVerificationSelection,
    ) -> Result<ReviewedPreparationReceipt, String>;
}

#[cgp_impl(new KaniReviewedBindingPreparation)]
#[uses(
    CanRequireReviewedTransition,
    CanPrepareAutomation,
    CanRunJsonPythonTool,
    CanReadJsonArtifact,
    CanValidateReviewedBinding,
    CanReadWriteArtifacts
)]
impl ReviewedBindingPreparation {
    fn prepare_reviewed_binding(
        &self,
        manifest_path: &str,
        output_dir: &str,
    ) -> Result<ReviewedPreparationReceipt, String> {
        self.prepare_reviewed_binding_with_selection(
            manifest_path,
            output_dir,
            &ReviewedVerificationSelection::default(),
        )
    }

    fn prepare_reviewed_binding_with_selection(
        &self,
        manifest_path: &str,
        output_dir: &str,
        selection: &ReviewedVerificationSelection,
    ) -> Result<ReviewedPreparationReceipt, String> {
        self.require_reviewed_action(output_dir, ReviewedLifecycleAction::Prepare)?;
        let preparation =
            self.prepare_automation_with_selection(manifest_path, output_dir, selection)?;
        let output_dir_path = PathBuf::from(output_dir);
        let binding_path = output_dir_path.join("verus-binding.json");
        let receipt_path = output_dir_path.join("reviewed-preparation-receipt.json");

        let mut args = vec![
            "bind".to_string(),
            "--normalized".to_string(),
            preparation.discovery_path.clone(),
            "--output".to_string(),
            binding_path.display().to_string(),
        ];
        args.extend(
            c![vec!["--function".to_string(), function.clone()] for function in &selection.functions]
                .into_iter()
                .flatten(),
        );
        let stdout_binding = self.run_python_json(
            "verification/tools/bind_kani_verus.py",
            &args,
        )?;
        let file_binding = self.read_json_artifact(&binding_path)?;
        if stdout_binding != file_binding {
            return Err("reviewed binder stdout and persisted receipt diverged".into());
        }
        let binding = self.validate_reviewed_binding(file_binding)?;

        let receipt = ReviewedPreparationReceipt {
            schema: 1,
            status: "KVERUS_REVIEWED_BINDING_PREPARED",
            default_engine: DEFAULT_ENGINE,
            proof_credit: false,
            verus_campaign_checked: false,
            preparation,
            binding_path: binding_path.display().to_string(),
            binding_status: binding.status,
            selected_count: binding.selected_count,
            unresolved_count: 0,
            controller_obligation_count: binding.controller_obligation_count,
            contract_inventory_count: binding.contract_inventory_count,
            contract_clause_count: binding.contract_clause_count,
        };
        self.write_artifact(&receipt_path, &pretty_json_bytes(&receipt)?)
            .map_err(|error| format!("cannot write {}: {error}", receipt_path.display()))?;
        Ok(receipt)
    }
}

#[derive(Deserialize)]
struct CampaignRunnerReceipt {
    schema: u32,
    status: String,
    engine: String,
    kani_proof_credit: bool,
    verus_checked_scoped: bool,
    deduplicated_entries: usize,
    formal_audit: String,
    case_acceptance_exit: i32,
    obligations: Vec<String>,
    timeout_seconds: u64,
    memory_mib: Option<u64>,
    max_concurrency: usize,
}

#[cgp_component(CampaignExecutionValidator)]
pub trait CanValidateCampaignExecution {
    fn validate_campaign_execution(
        &self,
        value: serde_json::Value,
    ) -> Result<CampaignExecutionSummary, String>;
}

#[cgp_impl(new StrictCampaignExecutionValidator)]
#[uses(CanDecodeJsonReceipt)]
impl CampaignExecutionValidator {
    fn validate_campaign_execution(
        &self,
        value: serde_json::Value,
    ) -> Result<CampaignExecutionSummary, String> {
        let receipt: CampaignRunnerReceipt =
            self.decode_json_receipt(value, "Verus campaign runner receipt")?;

        if receipt.schema != 1
            || receipt.status != "CHECKED_SCOPED"
            || receipt.engine != DEFAULT_ENGINE
            || receipt.kani_proof_credit
            || !receipt.verus_checked_scoped
            || receipt.deduplicated_entries == 0
            || receipt.formal_audit != "CHECKED_SCOPED"
            || receipt.case_acceptance_exit != 0
            || receipt.obligations.is_empty()
            || receipt.obligations.len() != receipt.deduplicated_entries
            || receipt.timeout_seconds == 0
            || receipt.timeout_seconds > 120
            || matches!(receipt.memory_mib, Some(0))
            || receipt.max_concurrency != 1
        {
            return Err(format!(
                "Verus campaign runner receipt violated KVerus invariants:                  schema={}, status={}, engine={}, kani_proof_credit={},                  verus_checked_scoped={}, entries={}, formal_audit={},                  case_acceptance_exit={}",
                receipt.schema,
                receipt.status,
                receipt.engine,
                receipt.kani_proof_credit,
                receipt.verus_checked_scoped,
                receipt.deduplicated_entries,
                receipt.formal_audit,
                receipt.case_acceptance_exit
            ));
        }

        Ok(CampaignExecutionSummary {
            status: receipt.status,
            checked_entry_count: receipt.deduplicated_entries,
            case_acceptance_exit: receipt.case_acceptance_exit,
            obligations: receipt.obligations,
            timeout_seconds: receipt.timeout_seconds,
            memory_mib: receipt.memory_mib,
            max_concurrency: receipt.max_concurrency,
        })
    }
}

#[cgp_component(ReviewedCampaignExecution)]
pub trait CanRunReviewedCampaign {
    fn run_reviewed_campaign(
        &self,
        reviewed_dir: &str,
        campaign_dir: &str,
        verus: &str,
    ) -> Result<ReviewedCampaignExecutionReceipt, String>;

    fn run_reviewed_campaign_with_limits(
        &self,
        reviewed_dir: &str,
        campaign_dir: &str,
        verus: &str,
        limits: &ReviewedVerificationLimits,
    ) -> Result<ReviewedCampaignExecutionReceipt, String>;
}

#[cgp_impl(new VerusReviewedCampaignExecution)]
#[uses(
    CanRequireReviewedTransition,
    CanInspectArtifactPresence,
    CanRunPythonTool,
    CanReadJsonArtifact,
    CanValidateReviewedBinding,
    CanValidateCampaignExecution,
    CanCanonicalizeStaticIdentity,
    CanReadWriteArtifacts,
    HasProjectRoot
)]
impl ReviewedCampaignExecution {
    fn run_reviewed_campaign(
        &self,
        reviewed_dir: &str,
        campaign_dir: &str,
        verus: &str,
    ) -> Result<ReviewedCampaignExecutionReceipt, String> {
        self.run_reviewed_campaign_with_limits(
            reviewed_dir,
            campaign_dir,
            verus,
            &ReviewedVerificationLimits::default(),
        )
    }

    fn run_reviewed_campaign_with_limits(
        &self,
        reviewed_dir: &str,
        campaign_dir: &str,
        verus: &str,
        limits: &ReviewedVerificationLimits,
    ) -> Result<ReviewedCampaignExecutionReceipt, String> {
        self.require_reviewed_action(reviewed_dir, ReviewedLifecycleAction::Run)?;

        let binding_path = PathBuf::from(reviewed_dir).join("verus-binding.json");
        let binding = self.validate_reviewed_binding(self.read_json_artifact(&binding_path)?)?;
        if binding.controller_obligations.is_empty() {
            return Err("reviewed Verus campaign cannot execute an empty obligation set".into());
        }

        let requested_campaign_dir = PathBuf::from(campaign_dir);
        let campaign_dir_path = if requested_campaign_dir.is_absolute() {
            requested_campaign_dir
        } else {
            self.project_root().join(requested_campaign_dir)
        };
        let runner_receipt_path = campaign_dir_path.join("verus-campaign-runner-receipt.json");
        let execution_receipt_path =
            campaign_dir_path.join("kverus-reviewed-campaign-execution.json");
        let freshness_paths = [
            runner_receipt_path.clone(),
            execution_receipt_path.clone(),
            campaign_dir_path.join("binding.json"),
            campaign_dir_path.join("audit.json"),
        ];
        for path in freshness_paths {
            if self.artifact_exists(&path)? {
                return Err(format!(
                    "reviewed Verus campaign requires fresh evidence; choose a different campaign directory because {} already exists",
                    path.display()
                ));
            }
        }
        let mut args = vec![
            "--verus".to_string(),
            verus.to_string(),
            "--campaign-dir".to_string(),
            campaign_dir_path.display().to_string(),
            "--receipt".to_string(),
            runner_receipt_path.display().to_string(),
            "--timeout-seconds".to_string(),
            limits.timeout_seconds.to_string(),
            "--max-concurrency".to_string(),
            limits.max_concurrency.to_string(),
        ];
        if let Some(memory_mib) = limits.memory_mib {
            args.push("--memory-mib".to_string());
            args.push(memory_mib.to_string());
        }
        args.extend(
            c![vec!["--obligation".to_string(), obligation.clone()] for obligation in &binding.controller_obligations]
                .into_iter()
                .flatten(),
        );

        let output = self
            .run_python_tool_output("verification/tools/run_verus_campaign.py", &args)
            .map_err(|error| format!("failed to execute reviewed Verus campaign: {error}"))?;

        let runner_value = self.read_json_artifact(&runner_receipt_path).map_err(|error| {
            format!(
                "reviewed Verus campaign did not produce {}: {error}",
                runner_receipt_path.display()
            )
        })?;

        if !output.status.success() {
            let runner_status = runner_value
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("UNKNOWN");
            let reason = runner_value
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("runner returned a non-zero exit status");
            return Err(format!(
                "reviewed Verus campaign failed with {}: status={runner_status}, reason={reason};                  stdout={} stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        let summary = self.validate_campaign_execution(runner_value)?;
        if summary.obligations != binding.controller_obligations {
            return Err(format!(
                "reviewed Verus campaign executed a different obligation inventory: expected {:?}, got {:?}",
                binding.controller_obligations, summary.obligations
            ));
        }
        if summary.timeout_seconds != limits.timeout_seconds
            || summary.memory_mib != limits.memory_mib
            || summary.max_concurrency != limits.max_concurrency
        {
            return Err(
                "reviewed Verus campaign resource limits diverged from the requested configuration"
                    .into(),
            );
        }
        let receipt = ReviewedCampaignExecutionReceipt {
            schema: 1,
            status: "KVERUS_REVIEWED_CAMPAIGN_EXECUTED",
            engine: DEFAULT_ENGINE,
            reviewed_dir: reviewed_dir.to_string(),
            campaign_dir: campaign_dir.to_string(),
            runner_receipt_path: runner_receipt_path.display().to_string(),
            execution_receipt_path: execution_receipt_path.display().to_string(),
            canonical_execution: self
                .canonicalize_static_identity("reviewed-campaign-execution")?,
            runner_status: summary.status,
            verus_checked_scoped: true,
            checked_entry_count: summary.checked_entry_count,
            case_acceptance_exit: summary.case_acceptance_exit,
            obligations: summary.obligations,
            limits: limits.clone(),
            proof_credit: false,
            proof_credit_source: "existing_verus_checked_scoped_receipts_only",
        };
        self.write_artifact(
            &execution_receipt_path,
            &pretty_json_bytes(&receipt)?,
        )
        .map_err(|error| {
            format!(
                "cannot write {}: {error}",
                execution_receipt_path.display()
            )
        })?;
        Ok(receipt)
    }
}

#[derive(Deserialize)]
struct CampaignBindingReceipt {
    schema: u32,
    status: String,
    engine: String,
    kani_proof_credit: bool,
    verus_checked_scoped: bool,
    receipts: Vec<serde_json::Value>,
    contract_inventory_count: usize,
    contract_clause_count: usize,
    translation_correctness: String,
}

#[cgp_component(CampaignBindingValidator)]
pub trait CanValidateCampaignBinding {
    fn validate_campaign_binding(
        &self,
        value: serde_json::Value,
    ) -> Result<CampaignBindingSummary, String>;
}

#[cgp_impl(new StrictCampaignBindingValidator)]
#[uses(CanDecodeJsonReceipt)]
impl CampaignBindingValidator {
    fn validate_campaign_binding(
        &self,
        value: serde_json::Value,
    ) -> Result<CampaignBindingSummary, String> {
        let campaign: CampaignBindingReceipt =
            self.decode_json_receipt(value, "campaign binding receipt")?;

        if campaign.schema != 1
            || campaign.status != "KANI_SELECTED_VERUS_CHECKED_SCOPED"
            || campaign.engine != DEFAULT_ENGINE
            || campaign.kani_proof_credit
            || !campaign.verus_checked_scoped
            || campaign.receipts.is_empty()
            || campaign.translation_correctness.is_empty()
        {
            return Err(format!(
                "campaign binding receipt violated KVerus invariants:                  schema={}, status={}, engine={}, kani_proof_credit={},                  verus_checked_scoped={}, receipts={}, translation_correctness_empty={}",
                campaign.schema,
                campaign.status,
                campaign.engine,
                campaign.kani_proof_credit,
                campaign.verus_checked_scoped,
                campaign.receipts.len(),
                campaign.translation_correctness.is_empty()
            ));
        }

        Ok(CampaignBindingSummary {
            status: campaign.status,
            checked_receipt_count: campaign.receipts.len(),
            contract_inventory_count: campaign.contract_inventory_count,
            contract_clause_count: campaign.contract_clause_count,
            translation_correctness: campaign.translation_correctness,
        })
    }
}

#[cgp_component(ReviewedCampaignFinalization)]
pub trait CanFinalizeReviewedCampaign {
    fn finalize_reviewed_campaign(
        &self,
        reviewed_dir: &str,
        campaign_dir: &str,
    ) -> Result<ReviewedFinalizationReceipt, String>;
}

#[cgp_impl(new KaniReviewedCampaignFinalization)]
#[uses(
    CanRequireReviewedTransition,
    CanRunJsonPythonTool,
    CanReadJsonArtifact,
    CanValidateCampaignBinding,
    CanReadWriteArtifacts
)]
impl ReviewedCampaignFinalization {
    fn finalize_reviewed_campaign(
        &self,
        reviewed_dir: &str,
        campaign_dir: &str,
    ) -> Result<ReviewedFinalizationReceipt, String> {
        self.require_reviewed_action(reviewed_dir, ReviewedLifecycleAction::Finalize)?;
        let reviewed_dir_path = PathBuf::from(reviewed_dir);
        let binding_path = reviewed_dir_path.join("verus-binding.json");
        let campaign_binding_path = reviewed_dir_path.join("verus-receipt-binding.json");
        let finalization_path = reviewed_dir_path.join("reviewed-finalization-receipt.json");

        let args = vec![
            "check-campaign".to_string(),
            "--binding".to_string(),
            binding_path.display().to_string(),
            "--campaign-dir".to_string(),
            campaign_dir.to_string(),
            "--output".to_string(),
            campaign_binding_path.display().to_string(),
        ];
        let stdout_receipt = self.run_python_json(
            "verification/tools/bind_kani_verus.py",
            &args,
        )?;
        let file_receipt = self.read_json_artifact(&campaign_binding_path)?;
        if stdout_receipt != file_receipt {
            return Err("campaign binder stdout and persisted receipt diverged".into());
        }

        let campaign = self.validate_campaign_binding(file_receipt)?;

        let receipt = ReviewedFinalizationReceipt {
            schema: 1,
            status: "KVERUS_REVIEWED_VERUS_FINALIZED",
            engine: DEFAULT_ENGINE,
            kani_proof_credit: false,
            verus_checked_scoped: true,
            reviewed_dir: reviewed_dir.to_string(),
            campaign_dir: campaign_dir.to_string(),
            binding_path: binding_path.display().to_string(),
            campaign_binding_path: campaign_binding_path.display().to_string(),
            campaign_binding_status: campaign.status,
            checked_receipt_count: campaign.checked_receipt_count,
            contract_inventory_count: campaign.contract_inventory_count,
            contract_clause_count: campaign.contract_clause_count,
            translation_correctness: campaign.translation_correctness,
        };
        self.write_artifact(&finalization_path, &pretty_json_bytes(&receipt)?)
            .map_err(|error| format!("cannot write {}: {error}", finalization_path.display()))?;
        Ok(receipt)
    }
}

#[cgp_component(ReviewedEvidenceBuilder)]
pub trait CanBuildReviewedEvidenceManifest {
    fn build_reviewed_evidence_manifest(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceManifest, String>;
}

#[cgp_impl(new StrictReviewedEvidenceBuilder)]
#[uses(
    CanReadJsonArtifact,
    CanDecodeJsonReceipt,
    CanReadWriteArtifacts,
    CanValidateReviewedBinding,
    CanValidateCampaignBinding,
    CanCanonicalizeStaticIdentity,
    CanDescribeEvidenceArtifacts,
    CanDigestArtifactBytes
)]
impl ReviewedEvidenceBuilder {
    fn build_reviewed_evidence_manifest(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceManifest, String> {
        let root = PathBuf::from(reviewed_dir);
        let preparation_path = root.join("reviewed-preparation-receipt.json");
        let binding_path = root.join("verus-binding.json");
        let campaign_binding_path = root.join("verus-receipt-binding.json");
        let finalization_path = root.join("reviewed-finalization-receipt.json");

        let preparation_value = self.read_json_artifact(&preparation_path)?;
        let binding_value = self.read_json_artifact(&binding_path)?;
        let campaign_value = self.read_json_artifact(&campaign_binding_path)?;
        let finalization_value = self.read_json_artifact(&finalization_path)?;

        let preparation: ReviewedPreparationArtifact =
            self.decode_json_receipt(preparation_value, "reviewed preparation receipt")?;
        let binding = self.validate_reviewed_binding(binding_value)?;
        let campaign = self.validate_campaign_binding(campaign_value)?;
        let finalization: ReviewedFinalizationArtifact =
            self.decode_json_receipt(finalization_value, "reviewed finalization receipt")?;

        validate_reviewed_preparation_artifact(
            &preparation,
            &binding,
            &binding_path,
            reviewed_dir,
        )?;
        validate_reviewed_finalization_artifact(
            &finalization,
            reviewed_dir,
            &binding_path,
            &campaign_binding_path,
            &campaign,
        )?;

        if binding.contract_inventory_count != campaign.contract_inventory_count
            || binding.contract_clause_count != campaign.contract_clause_count
        {
            return Err(
                "reviewed preparation and finalized campaign contract metadata counts diverged"
                    .into(),
            );
        }

        let mut artifacts = Vec::new();
        for spec in self.reviewed_evidence_artifact_specs() {
            let kind = self.evidence_artifact_kind_name(spec.kind);
            let path = root.join(spec.filename);
            let bytes = self
                .read_artifact(&path)
                .map_err(|error| format!("cannot read {} for digest: {error}", path.display()))?;
            artifacts.push(EvidenceArtifact {
                kind: kind.to_string(),
                canonical_token: self.canonicalize_static_identity(kind)?,
                path: path.display().to_string(),
                status: spec.status.to_string(),
                proof_credit_source: spec.proof_credit_source.to_string(),
                digest_algorithm: "sha256",
                digest: self.digest_artifact_bytes(&bytes),
            });
        }

        Ok(ReviewedEvidenceManifest {
            schema: 2,
            status: "KVERUS_REVIEWED_EVIDENCE_MANIFEST",
            engine: DEFAULT_ENGINE,
            kani_proof_credit: false,
            verus_checked_scoped: true,
            reviewed_dir: reviewed_dir.to_string(),
            selected_count: binding.selected_count,
            controller_obligation_count: binding.controller_obligation_count,
            checked_receipt_count: campaign.checked_receipt_count,
            contract_inventory_count: campaign.contract_inventory_count,
            contract_clause_count: campaign.contract_clause_count,
            translation_correctness: campaign.translation_correctness,
            proof_credit_source: "existing_verus_checked_scoped_receipts_only",
            artifacts,
        })
    }
}

#[cgp_component(ReviewedEvidenceAssembly)]
pub trait CanAssembleReviewedEvidence {
    fn assemble_reviewed_evidence(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceManifest, String>;
}

#[cgp_impl(new ContentAddressedReviewedEvidenceAssembly)]
#[uses(
    CanRequireReviewedTransition,
    CanBuildReviewedEvidenceManifest,
    CanReadWriteArtifacts
)]
impl ReviewedEvidenceAssembly {
    fn assemble_reviewed_evidence(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceManifest, String> {
        self.require_reviewed_action(reviewed_dir, ReviewedLifecycleAction::Manifest)?;
        let manifest = self.build_reviewed_evidence_manifest(reviewed_dir)?;
        let manifest_path = PathBuf::from(reviewed_dir).join("reviewed-evidence-manifest.json");
        self.write_artifact(&manifest_path, &pretty_json_bytes(&manifest)?)
            .map_err(|error| format!("cannot write {}: {error}", manifest_path.display()))?;
        Ok(manifest)
    }
}

#[cgp_component(ReviewedEvidenceAudit)]
pub trait CanAuditReviewedEvidence {
    fn audit_reviewed_evidence(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceAuditReceipt, String>;
}

#[cgp_impl(new ContentAddressedReviewedEvidenceAudit)]
#[uses(
    CanRequireReviewedTransition,
    CanBuildReviewedEvidenceManifest,
    CanReadJsonArtifact,
    CanReadWriteArtifacts,
    CanDigestArtifactBytes
)]
impl ReviewedEvidenceAudit {
    fn audit_reviewed_evidence(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceAuditReceipt, String> {
        self.require_reviewed_action(reviewed_dir, ReviewedLifecycleAction::Audit)?;
        let root = PathBuf::from(reviewed_dir);
        let manifest_path = root.join("reviewed-evidence-manifest.json");
        let audit_path = root.join("reviewed-evidence-audit.json");

        let stored_manifest = self.read_json_artifact(&manifest_path)?;
        let expected_manifest = self.build_reviewed_evidence_manifest(reviewed_dir)?;
        let expected_value = serde_json::to_value(&expected_manifest)
            .map_err(|error| format!("cannot serialize expected reviewed manifest: {error}"))?;

        if stored_manifest != expected_value {
            return Err(
                "reviewed evidence manifest diverged from the current content-addressed evidence"
                    .into(),
            );
        }

        let manifest_bytes = self
            .read_artifact(&manifest_path)
            .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
        let receipt = ReviewedEvidenceAuditReceipt {
            schema: 1,
            status: "KVERUS_REVIEWED_EVIDENCE_AUDITED",
            engine: DEFAULT_ENGINE,
            kani_proof_credit: false,
            verus_checked_scoped: true,
            reviewed_dir: reviewed_dir.to_string(),
            manifest_path: manifest_path.display().to_string(),
            manifest_digest_algorithm: "sha256",
            manifest_digest: self.digest_artifact_bytes(&manifest_bytes),
            verified_artifact_count: expected_manifest.artifacts.len(),
            proof_credit_source: "existing_verus_checked_scoped_receipts_only",
        };
        self.write_artifact(&audit_path, &pretty_json_bytes(&receipt)?)
            .map_err(|error| format!("cannot write {}: {error}", audit_path.display()))?;
        Ok(receipt)
    }
}

fn reviewed_integrity_artifact(
    context: &impl CanCanonicalizeStaticIdentity,
    kind: &str,
    path: &Path,
) -> Result<ReviewedIntegrityArtifact, String> {
    Ok(ReviewedIntegrityArtifact {
        kind: kind.to_string(),
        canonical_token: context.canonicalize_static_identity(kind)?,
        path: path.display().to_string(),
        status: "valid",
    })
}

#[cgp_component(ReviewedIntegrityValidation)]
pub trait CanValidateReviewedIntegrity {
    fn validate_reviewed_integrity(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedIntegrityReport, String>;
}

#[cgp_impl(new StrictReviewedIntegrityValidation)]
#[uses(
    CanInspectReviewedLifecycle,
    CanReadJsonArtifact,
    CanDecodeJsonReceipt,
    CanValidateReviewedBinding,
    CanValidateCampaignBinding,
    CanBuildReviewedEvidenceManifest,
    CanReadWriteArtifacts,
    CanDigestArtifactBytes,
    CanCanonicalizeStaticIdentity
)]
impl ReviewedIntegrityValidation {
    fn validate_reviewed_integrity(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedIntegrityReport, String> {
        let lifecycle = self.reviewed_lifecycle(reviewed_dir)?;
        let root = PathBuf::from(reviewed_dir);
        let mut artifacts = Vec::new();
        let mut violations = Vec::new();
        let rank = ReviewedLifecycleStage::from_kverus_name(lifecycle.stage)
            .and_then(ReviewedLifecycleStage::rank);

        if !lifecycle.consistent || rank.is_none() {
            violations.push(format!(
                "reviewed lifecycle is inconsistent at stage {} (present: {}; missing: {})",
                lifecycle.stage,
                lifecycle.present_artifacts.join(", "),
                lifecycle.missing_artifacts.join(", "),
            ));
        }

        if rank.is_some_and(|value| value >= 1) {
            let result = (|| -> Result<Vec<ReviewedIntegrityArtifact>, String> {
                let preparation_path = root.join("reviewed-preparation-receipt.json");
                let binding_path = root.join("verus-binding.json");
                let preparation_value = self.read_json_artifact(&preparation_path)?;
                let binding_value = self.read_json_artifact(&binding_path)?;
                let preparation: ReviewedPreparationArtifact = self.decode_json_receipt(
                    preparation_value,
                    "reviewed preparation receipt",
                )?;
                let binding = self.validate_reviewed_binding(binding_value)?;
                validate_reviewed_preparation_artifact(
                    &preparation,
                    &binding,
                    &binding_path,
                    reviewed_dir,
                )?;
                Ok(vec![
                    reviewed_integrity_artifact(
                        self,
                        "reviewed-preparation-receipt",
                        &preparation_path,
                    )?,
                    reviewed_integrity_artifact(self, "verus-binding", &binding_path)?,
                ])
            })();
            match result {
                Ok(rows) => artifacts.extend(rows),
                Err(error) => violations.push(error),
            }
        }

        if rank.is_some_and(|value| value >= 2) {
            let result = (|| -> Result<Vec<ReviewedIntegrityArtifact>, String> {
                let binding_path = root.join("verus-binding.json");
                let campaign_binding_path = root.join("verus-receipt-binding.json");
                let finalization_path = root.join("reviewed-finalization-receipt.json");

                let binding =
                    self.validate_reviewed_binding(self.read_json_artifact(&binding_path)?)?;
                let campaign = self.validate_campaign_binding(
                    self.read_json_artifact(&campaign_binding_path)?,
                )?;
                let finalization: ReviewedFinalizationArtifact = self.decode_json_receipt(
                    self.read_json_artifact(&finalization_path)?,
                    "reviewed finalization receipt",
                )?;
                validate_reviewed_finalization_artifact(
                    &finalization,
                    reviewed_dir,
                    &binding_path,
                    &campaign_binding_path,
                    &campaign,
                )?;

                if binding.contract_inventory_count != campaign.contract_inventory_count
                    || binding.contract_clause_count != campaign.contract_clause_count
                {
                    return Err(
                        "reviewed preparation and finalized campaign contract metadata counts diverged"
                            .into(),
                    );
                }

                Ok(vec![
                    reviewed_integrity_artifact(
                        self,
                        "verus-receipt-binding",
                        &campaign_binding_path,
                    )?,
                    reviewed_integrity_artifact(
                        self,
                        "reviewed-finalization-receipt",
                        &finalization_path,
                    )?,
                ])
            })();
            match result {
                Ok(rows) => artifacts.extend(rows),
                Err(error) => violations.push(error),
            }
        }

        if rank.is_some_and(|value| value >= 3) {
            let result = (|| -> Result<ReviewedIntegrityArtifact, String> {
                let manifest_path = root.join("reviewed-evidence-manifest.json");
                let stored_manifest = self.read_json_artifact(&manifest_path)?;
                let expected_manifest = self.build_reviewed_evidence_manifest(reviewed_dir)?;
                let expected_value = serde_json::to_value(&expected_manifest).map_err(|error| {
                    format!("cannot serialize expected reviewed manifest: {error}")
                })?;
                if stored_manifest != expected_value {
                    return Err(
                        "reviewed evidence manifest diverged from the current content-addressed evidence"
                            .into(),
                    );
                }
                reviewed_integrity_artifact(
                    self,
                    "reviewed-evidence-manifest",
                    &manifest_path,
                )
            })();
            match result {
                Ok(row) => artifacts.push(row),
                Err(error) => violations.push(error),
            }
        }

        if rank.is_some_and(|value| value >= 4) {
            let result = (|| -> Result<ReviewedIntegrityArtifact, String> {
                let manifest_path = root.join("reviewed-evidence-manifest.json");
                let audit_path = root.join("reviewed-evidence-audit.json");
                let expected_manifest = self.build_reviewed_evidence_manifest(reviewed_dir)?;
                let manifest_bytes = self
                    .read_artifact(&manifest_path)
                    .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
                let digest = self.digest_artifact_bytes(&manifest_bytes);
                let audit: ReviewedEvidenceAuditArtifact = self.decode_json_receipt(
                    self.read_json_artifact(&audit_path)?,
                    "reviewed evidence audit receipt",
                )?;
                validate_reviewed_audit_artifact(
                    &audit,
                    reviewed_dir,
                    &manifest_path,
                    &digest,
                    expected_manifest.artifacts.len(),
                )?;
                reviewed_integrity_artifact(
                    self,
                    "reviewed-evidence-audit",
                    &audit_path,
                )
            })();
            match result {
                Ok(row) => artifacts.push(row),
                Err(error) => violations.push(error),
            }
        }

        Ok(ReviewedIntegrityReport {
            schema: 1,
            status: "KVERUS_REVIEWED_INTEGRITY",
            reviewed_dir: reviewed_dir.to_string(),
            lifecycle_stage: lifecycle.stage.to_string(),
            canonical_stage: lifecycle.canonical_stage,
            canonical_validation: self
                .canonicalize_static_identity("reviewed-integrity-validation")?,
            valid: lifecycle.consistent && violations.is_empty(),
            validated_artifact_count: artifacts.len(),
            artifacts,
            violations,
            proof_credit: false,
        })
    }
}

#[cgp_component(ReviewedVerification)]
pub trait CanVerifyReviewed {
    fn verify_reviewed(
        &self,
        manifest_path: &str,
        reviewed_dir: &str,
        campaign_dir: &str,
        verus: &str,
    ) -> Result<ReviewedVerificationReceipt, String>;
}

#[cgp_impl(new FreshReviewedVerification)]
#[uses(
    CanInspectReviewedLifecycle,
    CanValidateReviewedIntegrity,
    CanReadJsonArtifact,
    CanDecodeJsonReceipt,
    CanPrepareReviewedBinding,
    CanRunReviewedCampaign,
    CanFinalizeReviewedCampaign,
    CanAssembleReviewedEvidence,
    CanAuditReviewedEvidence,
    CanCanonicalizeStaticIdentity
)]
impl ReviewedVerification {
    fn verify_reviewed(
        &self,
        manifest_path: &str,
        reviewed_dir: &str,
        campaign_dir: &str,
        verus: &str,
    ) -> Result<ReviewedVerificationReceipt, String> {
        let lifecycle = self.reviewed_lifecycle(reviewed_dir)?;
        let starting_stage = ReviewedLifecycleStage::from_kverus_name(lifecycle.stage)
            .ok_or_else(|| format!("unknown reviewed lifecycle stage {}", lifecycle.stage))?;
        if !lifecycle.consistent
            || !matches!(
                starting_stage,
                ReviewedLifecycleStage::Empty | ReviewedLifecycleStage::Prepared
            )
        {
            return Err(format!(
                "verify-reviewed requires reviewed-empty or reviewed-prepared so Verus proof execution is fresh; current stage is {}",
                lifecycle.stage
            ));
        }

        let preparation_reused = starting_stage == ReviewedLifecycleStage::Prepared;
        if preparation_reused {
            let integrity = self.validate_reviewed_integrity(reviewed_dir)?;
            if !integrity.valid {
                return Err(format!(
                    "verify-reviewed refuses invalid prepared evidence: {}",
                    integrity.violations.join("; ")
                ));
            }
            let preparation_path =
                PathBuf::from(reviewed_dir).join("reviewed-preparation-receipt.json");
            let preparation: ReviewedPreparationArtifact = self.decode_json_receipt(
                self.read_json_artifact(&preparation_path)?,
                "reviewed preparation receipt",
            )?;
            if preparation.preparation.manifest_path != manifest_path {
                return Err(format!(
                    "verify-reviewed manifest mismatch: prepared evidence is bound to {:?}, requested {:?}",
                    preparation.preparation.manifest_path, manifest_path
                ));
            }
        } else {
            self.prepare_reviewed_binding(manifest_path, reviewed_dir)?;
        }

        let execution = self.run_reviewed_campaign(reviewed_dir, campaign_dir, verus)?;
        let finalization = self.finalize_reviewed_campaign(reviewed_dir, campaign_dir)?;
        let manifest = self.assemble_reviewed_evidence(reviewed_dir)?;
        if finalization.checked_receipt_count != manifest.checked_receipt_count {
            return Err("verify-reviewed finalization and manifest receipt counts diverged".into());
        }
        let audit = self.audit_reviewed_evidence(reviewed_dir)?;
        if audit.verified_artifact_count != manifest.artifacts.len() {
            return Err("verify-reviewed audit and manifest artifact counts diverged".into());
        }
        let integrity = self.validate_reviewed_integrity(reviewed_dir)?;
        if !integrity.valid
            || integrity.lifecycle_stage != ReviewedLifecycleStage::Audited.kverus_name()
        {
            return Err(format!(
                "verify-reviewed final integrity validation failed at {}: {}",
                integrity.lifecycle_stage,
                integrity.violations.join("; ")
            ));
        }

        let executed_steps = c![
            step.kverus_name()
            for step in ReviewedVerificationStep::ALL.iter().copied()
            if !preparation_reused || step != ReviewedVerificationStep::Prepare
        ];
        let reused_steps = c![
            step.kverus_name()
            for step in ReviewedVerificationStep::ALL.iter().copied()
            if preparation_reused && step == ReviewedVerificationStep::Prepare
        ];

        Ok(ReviewedVerificationReceipt {
            schema: 1,
            status: "KVERUS_REVIEWED_WORKFLOW_COMPLETE",
            engine: DEFAULT_ENGINE,
            reviewed_dir: reviewed_dir.to_string(),
            campaign_dir: campaign_dir.to_string(),
            canonical_workflow: self.canonicalize_static_identity("reviewed-verification")?,
            starting_stage: lifecycle.stage.to_string(),
            preparation_reused,
            executed_steps,
            reused_steps,
            selected_count: manifest.selected_count,
            controller_obligation_count: manifest.controller_obligation_count,
            checked_entry_count: execution.checked_entry_count,
            checked_receipt_count: manifest.checked_receipt_count,
            verified_artifact_count: audit.verified_artifact_count,
            contract_inventory_count: manifest.contract_inventory_count,
            contract_clause_count: manifest.contract_clause_count,
            translation_correctness: manifest.translation_correctness,
            manifest_path: audit.manifest_path,
            manifest_digest_algorithm: audit.manifest_digest_algorithm,
            manifest_digest: audit.manifest_digest,
            final_stage: integrity.lifecycle_stage,
            integrity_valid: integrity.valid,
            proof_credit: false,
            proof_credit_source: "existing_verus_checked_scoped_receipts_only",
        })
    }
}

#[cgp_component(ReviewedExplanation)]
pub trait CanExplainReviewed {
    fn explain_reviewed(&self, reviewed_dir: &str) -> Result<ReviewedExplanationReport, String>;
}

#[cgp_impl(new ComposedReviewedExplanation)]
#[uses(CanInspectReviewedLifecycle, CanValidateReviewedIntegrity)]
impl ReviewedExplanation {
    fn explain_reviewed(&self, reviewed_dir: &str) -> Result<ReviewedExplanationReport, String> {
        let lifecycle = self.reviewed_lifecycle(reviewed_dir)?;
        let integrity = self.validate_reviewed_integrity(reviewed_dir)?;
        let stage = ReviewedLifecycleStage::from_kverus_name(lifecycle.stage)
            .unwrap_or(ReviewedLifecycleStage::Inconsistent);
        let integrity_valid = integrity.valid;
        let recommended_next_action = if integrity_valid {
            stage.recommended_next_action()
        } else {
            None
        };
        let complete = integrity_valid && stage == ReviewedLifecycleStage::Audited;
        let blockers = c![violation.clone() for violation in &integrity.violations];

        Ok(ReviewedExplanationReport {
            schema: 1,
            status: "KVERUS_REVIEWED_EXPLANATION",
            reviewed_dir: reviewed_dir.to_string(),
            lifecycle,
            integrity,
            recommended_next_action,
            ready_for_next_action: recommended_next_action.is_some(),
            complete,
            blockers,
            proof_credit: false,
        })
    }
}

fn pretty_json_bytes(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub struct KVerusContext;

delegate_and_check_components! {
    KVerusContext {
        TokenIdentityComponent:
            ProcMacro2TokenIdentity,
        CommandCatalogComponent:
            StaticCommandCatalog,
        CommandRouterComponent:
            StaticCommandRouter,
        MetadataParserComponent:
            JsonMetadataParser,
        ContractClauseSyntaxComponent:
            ProcMacro2ContractClauseSyntax,
        ContractClauseValidatorComponent:
            CanonicalContractClauseValidator,
        MetadataNormalizerComponent:
            KaniMetadataNormalizer,
        AutomationPlannerComponent:
            KaniAutomationPlanner,
        ReviewedVerificationConfigValidationComponent:
            StrictReviewedVerificationConfigValidation,
        ReviewedVerificationConfigStoreComponent:
            JsonReviewedVerificationConfigStore,
        DiscoverySelectionArgumentsComponent:
            KaniDiscoverySelectionArguments,
        ProjectLayoutComponent:
            CargoManifestProjectLayout,
        PythonToolRunnerComponent:
            SystemPythonToolRunner,
        JsonPythonToolRunnerComponent:
            CapturedJsonPythonToolRunner,
        ArtifactStoreComponent:
            StdArtifactStore,
        ArtifactPresenceComponent:
            StdArtifactPresence,
        ReviewedLifecycleInspectionComponent:
            ArtifactBackedReviewedLifecycleInspection,
        ReviewedLifecycleTransitionComponent:
            StrictReviewedLifecycleTransition,
        ArtifactDigestComponent:
            Sha256ArtifactDigest,
        EvidenceArtifactPolicyComponent:
            StaticEvidenceArtifactPolicy,
        JsonArtifactStoreComponent:
            SerdeJsonArtifactStore,
        JsonReceiptDecoderComponent:
            SerdeJsonReceiptDecoder,
        AutomationPreparationComponent:
            KaniAutomationPreparation,
        ReviewedBindingValidatorComponent:
            StrictReviewedBindingValidator,
        CampaignExecutionValidatorComponent:
            StrictCampaignExecutionValidator,
        CampaignBindingValidatorComponent:
            StrictCampaignBindingValidator,
        ReviewedBindingPreparationComponent:
            KaniReviewedBindingPreparation,
        ReviewedCampaignExecutionComponent:
            VerusReviewedCampaignExecution,
        ReviewedCampaignFinalizationComponent:
            KaniReviewedCampaignFinalization,
        ReviewedEvidenceBuilderComponent:
            StrictReviewedEvidenceBuilder,
        ReviewedEvidenceAssemblyComponent:
            ContentAddressedReviewedEvidenceAssembly,
        ReviewedEvidenceAuditComponent:
            ContentAddressedReviewedEvidenceAudit,
        ReviewedIntegrityValidationComponent:
            StrictReviewedIntegrityValidation,
        ReviewedVerificationComponent:
            FreshReviewedVerification,
        ReviewedExplanationComponent:
            ComposedReviewedExplanation,
    }
}

fn contract_clause_kind_name(kind: &ContractClauseKind) -> &'static str {
    match kind {
        ContractClauseKind::Requires => "requires",
        ContractClauseKind::Ensures => "ensures",
        ContractClauseKind::Modifies => "modifies",
    }
}

fn python_executable() -> OsString {
    env::var_os("DESTINY_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            OsString::from("python")
        } else {
            OsString::from("python3")
        }
    })
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
