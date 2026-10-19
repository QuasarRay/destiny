use crate::{
    AutomationPlan, ContractDiscovery, ContractPlan, DEFAULT_ENGINE, DiscoveryCandidate,
    DiscoveryDisposition, DiscoveryReport, KANI_SOURCE_COMMIT, PlanCounts,
    PlannedContractClause, PreparationReceipt,
};
use cgp::prelude::*;
use kani_metadata::{
    AutoHarnessSkipReason, ContractClause, ContractClauseKind, KaniMetadata,
};
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
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

        let validated = ValidatedClauseTokens {
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

fn pretty_json_bytes(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub struct KVerusContext;

delegate_and_check_components! {
    KVerusContext {
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
        ArtifactStoreComponent:
            StdArtifactStore,
        AutomationPreparationComponent:
            KaniAutomationPreparation,
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
