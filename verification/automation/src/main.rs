#![forbid(unsafe_code)]

use destiny_verification_automation::{
    AutomationCommand, CanAssembleReviewedEvidence, CanAuditReviewedEvidence,
    CanBuildAutomationPlan, CanDescribeCommands, CanExplainReviewed,
    CanLoadReviewedVerificationConfig,
    CanFinalizeReviewedCampaign, CanInspectReviewedLifecycle, CanPrepareAutomation,
    CanRunReviewedCampaign,
    CanValidateReviewedIntegrity, CanVerifyReviewed, CanVerifyReviewedWithConfig,
    CanPrepareReviewedBinding, CanRouteCommand, CanRunPythonTool, DEFAULT_ENGINE,
    DiscoveryReport, KANI_SOURCE_COMMIT, KVerusContext, normalize_autoharness_metadata,
    parse_metadata,
};
use std::{
    env,
    fs::File,
    process::{ExitCode, ExitStatus},
};

fn usage(reason: &str) -> ! {
    eprintln!("{reason}");
    eprintln!(
        "{}",
        KVerusContext
            .command_usage()
            .expect("static KVerus command catalog must remain valid")
    );
    std::process::exit(2);
}

fn run_python(
    script: &str,
    args: impl IntoIterator<Item = String>,
) -> Result<ExitStatus, std::io::Error> {
    let args = args.into_iter().collect::<Vec<_>>();
    KVerusContext.run_python_tool(script, &args)
}

fn exit_status(status: ExitStatus) -> ExitCode {
    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(status.code().and_then(|code| u8::try_from(code).ok()).unwrap_or(1))
    }
}

fn delegate(script: &str, args: impl IntoIterator<Item = String>) -> ExitCode {
    match run_python(script, args) {
        Ok(status) => exit_status(status),
        Err(error) => {
            eprintln!("failed to execute {script}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn setup() -> ExitCode {
    for script in [
        "verification/tools/check_vendor_integrity.py",
        "verification/tools/bootstrap_kani_vendor.py",
    ] {
        match run_python(script, std::iter::empty()) {
            Ok(status) if status.success() => {}
            Ok(status) => return exit_status(status),
            Err(error) => {
                eprintln!("failed to execute {script}: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn load_discovery(path: &str) -> Result<DiscoveryReport, String> {
    File::open(path)
        .map_err(|error| error.to_string())
        .and_then(|file| parse_metadata(file).map_err(|error| error.to_string()))
        .map(|metadata| normalize_autoharness_metadata(&metadata))
}

fn emit_json<T: serde::Serialize>(value: &T) -> ExitCode {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn discovery(path: &str) -> ExitCode {
    match load_discovery(path) {
        Ok(report) => emit_json(&report),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn plan(path: &str) -> ExitCode {
    let result = load_discovery(path)
        .and_then(|report| KVerusContext.build_automation_plan(&report));
    match result {
        Ok(plan) => emit_json(&plan),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn prepare(manifest_path: &str, output_dir: &str) -> ExitCode {
    match KVerusContext.prepare_automation(manifest_path, output_dir) {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn prepare_reviewed(manifest_path: &str, output_dir: &str) -> ExitCode {
    match KVerusContext.prepare_reviewed_binding(manifest_path, output_dir) {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn verify_reviewed(
    manifest_path: &str,
    reviewed_dir: &str,
    campaign_dir: &str,
    verus: &str,
    config_path: Option<&str>,
) -> ExitCode {
    let result = match config_path {
        Some(path) => KVerusContext
            .load_reviewed_verification_config(path)
            .and_then(|config| {
                KVerusContext.verify_reviewed_with_config(
                    manifest_path,
                    reviewed_dir,
                    campaign_dir,
                    verus,
                    &config,
                )
            }),
        None => KVerusContext.verify_reviewed(manifest_path, reviewed_dir, campaign_dir, verus),
    };
    match result {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run_reviewed(reviewed_dir: &str, campaign_dir: &str, verus: &str) -> ExitCode {
    match KVerusContext.run_reviewed_campaign(reviewed_dir, campaign_dir, verus) {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn finalize_reviewed(reviewed_dir: &str, campaign_dir: &str) -> ExitCode {
    match KVerusContext.finalize_reviewed_campaign(reviewed_dir, campaign_dir) {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn manifest_reviewed(reviewed_dir: &str) -> ExitCode {
    match KVerusContext.assemble_reviewed_evidence(reviewed_dir) {
        Ok(manifest) => emit_json(&manifest),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn audit_reviewed(reviewed_dir: &str) -> ExitCode {
    match KVerusContext.audit_reviewed_evidence(reviewed_dir) {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn status_reviewed(reviewed_dir: &str) -> ExitCode {
    match KVerusContext.reviewed_lifecycle(reviewed_dir) {
        Ok(report) => emit_json(&report),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn validate_reviewed(reviewed_dir: &str) -> ExitCode {
    match KVerusContext.validate_reviewed_integrity(reviewed_dir) {
        Ok(report) => {
            let valid = report.valid;
            match serde_json::to_string_pretty(&report) {
                Ok(json) => {
                    println!("{json}");
                    if valid {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::FAILURE
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn explain_reviewed(reviewed_dir: &str) -> ExitCode {
    match KVerusContext.explain_reviewed(reviewed_dir) {
        Ok(report) => emit_json(&report),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let command = KVerusContext
        .route_command(&args)
        .unwrap_or_else(|error| usage(&error));

    match command {
        AutomationCommand::Doctor => {
            println!(
                "{{\"default_engine\":\"{DEFAULT_ENGINE}\",\"kani_source_commit\":\"{KANI_SOURCE_COMMIT}\",\"proof_credit\":false}}"
            );
            ExitCode::SUCCESS
        }
        AutomationCommand::Setup => setup(),
        AutomationCommand::Discover { path } => discovery(&path),
        AutomationCommand::Plan { path } => plan(&path),
        AutomationCommand::Prepare {
            manifest_path,
            output_dir,
        } => prepare(&manifest_path, &output_dir),
        AutomationCommand::PrepareReviewed {
            manifest_path,
            output_dir,
        } => prepare_reviewed(&manifest_path, &output_dir),
        AutomationCommand::VerifyReviewed {
            manifest_path,
            reviewed_dir,
            campaign_dir,
            verus,
            config_path,
        } => verify_reviewed(
            &manifest_path,
            &reviewed_dir,
            &campaign_dir,
            &verus,
            config_path.as_deref(),
        ),
        AutomationCommand::RunReviewed {
            reviewed_dir,
            campaign_dir,
            verus,
        } => run_reviewed(&reviewed_dir, &campaign_dir, &verus),
        AutomationCommand::FinalizeReviewed {
            reviewed_dir,
            campaign_dir,
        } => finalize_reviewed(&reviewed_dir, &campaign_dir),
        AutomationCommand::ManifestReviewed { reviewed_dir } => {
            manifest_reviewed(&reviewed_dir)
        }
        AutomationCommand::AuditReviewed { reviewed_dir } => audit_reviewed(&reviewed_dir),
        AutomationCommand::StatusReviewed { reviewed_dir } => status_reviewed(&reviewed_dir),
        AutomationCommand::ValidateReviewed { reviewed_dir } => {
            validate_reviewed(&reviewed_dir)
        }
        AutomationCommand::ExplainReviewed { reviewed_dir } => explain_reviewed(&reviewed_dir),
        AutomationCommand::Explain => {
            print!("{}", include_str!("../capabilities.json"));
            ExitCode::SUCCESS
        }
        AutomationCommand::Verify { mut args } => {
            args.insert(0, "verify".to_string());
            delegate("verification/tools/verus_subset.py", args)
        }
        AutomationCommand::ReplayKaniReference { args } => {
            delegate("verification/tools/replay_kani_fixture.py", args)
        }
    }
}
