use crate::{
    AutomationCommand, AutomationPlan, CampaignBindingSummary, CommandDescriptor,
    ContractDiscovery, ContractPlan, DEFAULT_ENGINE, DiscoveryCandidate,
    DiscoveryDisposition, DiscoveryReport, EvidenceArtifact, EvidenceArtifactKind,
    EvidenceArtifactSpec, KANI_SOURCE_COMMIT, PlanCounts, PlannedContractClause,
    PreparationReceipt, ReviewedBindingSummary, ReviewedEvidenceAuditReceipt,
    ReviewedEvidenceManifest, ReviewedFinalizationReceipt, ReviewedPreparationReceipt,
};
use cgp::prelude::*;
use kani_metadata::{
    AutoHarnessSkipReason, ContractClause, ContractClauseKind, KaniMetadata,
};
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
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
    COMMANDS
        .iter()
        .map(|(name, synopsis, category)| {
            Ok(CommandDescriptor {
                name: (*name).to_string(),
                synopsis: (*synopsis).to_string(),
                category: (*category).to_string(),
                canonical_token: context.canonicalize_static_identity(name)?,
                proof_credit: false,
            })
        })
        .collect()
}

fn static_command_usage(
    context: &impl CanCanonicalizeStaticIdentity,
) -> Result<String, String> {
    let commands = static_command_descriptors(context)?;
    Ok(format!(
        "usage: destiny-verification-automation {}",
        commands
            .iter()
            .map(|command| command.synopsis.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
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

fn parse_required_value(args: &[String], flag: &str) -> Result<String, String> {
    if args.len() != 2 || args[0] != flag {
        return Err(format!("exactly {flag} <value> is required"));
    }
    Ok(args[1].clone())
}

fn parse_required_pair(
    args: &[String],
    first_flag: &str,
    second_flag: &str,
) -> Result<(String, String), String> {
    let mut first = None;
    let mut second = None;
    let mut index = 0;

    while index < args.len() {
        let flag = args[index].as_str();
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?
            .clone();
        if flag == first_flag && first.is_none() {
            first = Some(value);
        } else if flag == second_flag && second.is_none() {
            second = Some(value);
        } else {
            return Err(format!("unexpected or duplicate argument {flag}"));
        }
        index += 2;
    }

    match (first, second) {
        (Some(first), Some(second)) => Ok((first, second)),
        _ => Err(format!("both {first_flag} and {second_flag} are required")),
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
        let mut selected = Vec::new();
        let mut blocked = Vec::new();
        let mut unsupported = Vec::new();
        let mut excluded = Vec::new();

        for candidate in &report.candidates {
            match candidate.disposition {
                DiscoveryDisposition::KaniEligible => selected.push(candidate.clone()),
                DiscoveryDisposition::Blocked => blocked.push(candidate.clone()),
                DiscoveryDisposition::Unsupported => unsupported.push(candidate.clone()),
                DiscoveryDisposition::Excluded => excluded.push(candidate.clone()),
            }
        }

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
        vec![
            EvidenceArtifactSpec {
                kind: EvidenceArtifactKind::ReviewedPreparationReceipt,
                filename: "reviewed-preparation-receipt.json",
                status: "KVERUS_REVIEWED_BINDING_PREPARED",
                proof_credit_source: "none",
            },
            EvidenceArtifactSpec {
                kind: EvidenceArtifactKind::VerusBinding,
                filename: "verus-binding.json",
                status: "BOUND_FOR_VERUS",
                proof_credit_source: "none",
            },
            EvidenceArtifactSpec {
                kind: EvidenceArtifactKind::VerusReceiptBinding,
                filename: "verus-receipt-binding.json",
                status: "KANI_SELECTED_VERUS_CHECKED_SCOPED",
                proof_credit_source: "verus_checked_scoped",
            },
            EvidenceArtifactSpec {
                kind: EvidenceArtifactKind::ReviewedFinalizationReceipt,
                filename: "reviewed-finalization-receipt.json",
                status: "KVERUS_REVIEWED_VERUS_FINALIZED",
                proof_credit_source: "verus_checked_scoped",
            },
        ]
    }

    fn evidence_artifact_kind_name(&self, kind: EvidenceArtifactKind) -> &'static str {
        match kind {
            EvidenceArtifactKind::ReviewedPreparationReceipt => {
                "reviewed-preparation-receipt"
            }
            EvidenceArtifactKind::VerusBinding => "verus-binding",
            EvidenceArtifactKind::VerusReceiptBinding => "verus-receipt-binding",
            EvidenceArtifactKind::ReviewedFinalizationReceipt => {
                "reviewed-finalization-receipt"
            }
        }
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

#[cgp_component(AutomationPreparation)]
pub trait CanPrepareAutomation {
    fn prepare_automation(
        &self,
        manifest_path: &str,
        output_dir: &str,
    ) -> Result<PreparationReceipt, String>;
}

#[cgp_impl(new KaniAutomationPreparation)]
#[uses(
    CanRunPythonTool,
    CanReadWriteArtifacts,
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
        let args = vec![
            "--manifest-path".to_string(),
            manifest_path.to_string(),
            "--output-dir".to_string(),
            output_dir.to_string(),
        ];
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
impl ReviewedBindingValidator {
    fn validate_reviewed_binding(
        &self,
        value: serde_json::Value,
    ) -> Result<ReviewedBindingSummary, String> {
        let binding: ReviewedBindingReceipt = serde_json::from_value(value)
            .map_err(|error| format!("invalid reviewed binder receipt: {error}"))?;

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
}

#[cgp_impl(new KaniReviewedBindingPreparation)]
#[uses(
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
        let preparation = self.prepare_automation(manifest_path, output_dir)?;
        let output_dir_path = PathBuf::from(output_dir);
        let binding_path = output_dir_path.join("verus-binding.json");
        let receipt_path = output_dir_path.join("reviewed-preparation-receipt.json");

        let args = vec![
            "bind".to_string(),
            "--normalized".to_string(),
            preparation.discovery_path.clone(),
            "--output".to_string(),
            binding_path.display().to_string(),
        ];
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
impl CampaignBindingValidator {
    fn validate_campaign_binding(
        &self,
        value: serde_json::Value,
    ) -> Result<CampaignBindingSummary, String> {
        let campaign: CampaignBindingReceipt = serde_json::from_value(value)
            .map_err(|error| format!("invalid campaign binding receipt: {error}"))?;

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

        let preparation = self.read_json_artifact(&preparation_path)?;
        let binding_value = self.read_json_artifact(&binding_path)?;
        let campaign_value = self.read_json_artifact(&campaign_binding_path)?;
        let finalization = self.read_json_artifact(&finalization_path)?;

        let binding = self.validate_reviewed_binding(binding_value)?;
        let campaign = self.validate_campaign_binding(campaign_value)?;

        require_json_u64(&preparation, "schema", 1)?;
        require_json_str(&preparation, "status", "KVERUS_REVIEWED_BINDING_PREPARED")?;
        require_json_str(&preparation, "default_engine", DEFAULT_ENGINE)?;
        require_json_bool(&preparation, "proof_credit", false)?;
        require_json_bool(&preparation, "verus_campaign_checked", false)?;
        require_json_str(
            &preparation,
            "binding_path",
            &binding_path.display().to_string(),
        )?;
        require_json_str(&preparation, "binding_status", &binding.status)?;
        require_json_usize(&preparation, "unresolved_count", 0)?;
        require_json_usize(&preparation, "selected_count", binding.selected_count)?;
        require_json_usize(
            &preparation,
            "controller_obligation_count",
            binding.controller_obligation_count,
        )?;
        require_json_usize(
            &preparation,
            "contract_inventory_count",
            binding.contract_inventory_count,
        )?;
        require_json_usize(
            &preparation,
            "contract_clause_count",
            binding.contract_clause_count,
        )?;

        require_json_u64(&finalization, "schema", 1)?;
        require_json_str(
            &finalization,
            "status",
            "KVERUS_REVIEWED_VERUS_FINALIZED",
        )?;
        require_json_str(&finalization, "engine", DEFAULT_ENGINE)?;
        require_json_bool(&finalization, "kani_proof_credit", false)?;
        require_json_bool(&finalization, "verus_checked_scoped", true)?;
        require_json_str(&finalization, "reviewed_dir", reviewed_dir)?;
        require_json_str(
            &finalization,
            "binding_path",
            &binding_path.display().to_string(),
        )?;
        require_json_str(
            &finalization,
            "campaign_binding_path",
            &campaign_binding_path.display().to_string(),
        )?;
        require_json_usize(
            &finalization,
            "checked_receipt_count",
            campaign.checked_receipt_count,
        )?;
        require_json_usize(
            &finalization,
            "contract_inventory_count",
            campaign.contract_inventory_count,
        )?;
        require_json_usize(
            &finalization,
            "contract_clause_count",
            campaign.contract_clause_count,
        )?;
        require_json_str(
            &finalization,
            "campaign_binding_status",
            &campaign.status,
        )?;
        require_json_str(
            &finalization,
            "translation_correctness",
            &campaign.translation_correctness,
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
#[uses(CanBuildReviewedEvidenceManifest, CanReadWriteArtifacts)]
impl ReviewedEvidenceAssembly {
    fn assemble_reviewed_evidence(
        &self,
        reviewed_dir: &str,
    ) -> Result<ReviewedEvidenceManifest, String> {
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

fn json_field<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a serde_json::Value, String> {
    value
        .get(field)
        .ok_or_else(|| format!("missing JSON field {field:?}"))
}

fn require_json_str(
    value: &serde_json::Value,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    let actual = json_field(value, field)?
        .as_str()
        .ok_or_else(|| format!("JSON field {field:?} is not a string"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("JSON field {field:?} expected {expected:?}, got {actual:?}"))
    }
}

fn require_json_bool(
    value: &serde_json::Value,
    field: &str,
    expected: bool,
) -> Result<(), String> {
    let actual = json_field(value, field)?
        .as_bool()
        .ok_or_else(|| format!("JSON field {field:?} is not a boolean"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("JSON field {field:?} expected {expected}, got {actual}"))
    }
}

fn require_json_u64(
    value: &serde_json::Value,
    field: &str,
    expected: u64,
) -> Result<(), String> {
    let actual = json_field(value, field)?
        .as_u64()
        .ok_or_else(|| format!("JSON field {field:?} is not an unsigned integer"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("JSON field {field:?} expected {expected}, got {actual}"))
    }
}

fn require_json_usize(
    value: &serde_json::Value,
    field: &str,
    expected: usize,
) -> Result<(), String> {
    let expected = u64::try_from(expected)
        .map_err(|_| format!("expected value for {field:?} does not fit u64"))?;
    require_json_u64(value, field, expected)
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
        ProjectLayoutComponent:
            CargoManifestProjectLayout,
        PythonToolRunnerComponent:
            SystemPythonToolRunner,
        JsonPythonToolRunnerComponent:
            CapturedJsonPythonToolRunner,
        ArtifactStoreComponent:
            StdArtifactStore,
        ArtifactDigestComponent:
            Sha256ArtifactDigest,
        EvidenceArtifactPolicyComponent:
            StaticEvidenceArtifactPolicy,
        JsonArtifactStoreComponent:
            SerdeJsonArtifactStore,
        AutomationPreparationComponent:
            KaniAutomationPreparation,
        ReviewedBindingValidatorComponent:
            StrictReviewedBindingValidator,
        CampaignBindingValidatorComponent:
            StrictCampaignBindingValidator,
        ReviewedBindingPreparationComponent:
            KaniReviewedBindingPreparation,
        ReviewedCampaignFinalizationComponent:
            KaniReviewedCampaignFinalization,
        ReviewedEvidenceBuilderComponent:
            StrictReviewedEvidenceBuilder,
        ReviewedEvidenceAssemblyComponent:
            ContentAddressedReviewedEvidenceAssembly,
        ReviewedEvidenceAuditComponent:
            ContentAddressedReviewedEvidenceAudit,
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
