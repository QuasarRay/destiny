#![forbid(unsafe_code)]

use destiny_verification_automation::{
    CanBuildAutomationPlan, CanPrepareAutomation, CanRunPythonTool, DEFAULT_ENGINE,
    DiscoveryReport, KANI_SOURCE_COMMIT, KVerusContext, normalize_autoharness_metadata,
    parse_metadata,
};
use std::{
    env,
    fs::File,
    process::{ExitCode, ExitStatus},
};

fn usage() -> ! {
    eprintln!(
        "usage: destiny-verification-automation \
         doctor | setup | discover <kani-metadata.json> | list <kani-metadata.json> | \
         plan <kani-metadata.json> | prepare --manifest-path <Cargo.toml> \
         --output-dir <dir> | explain | verify <verus-subset args...> | \
         replay-kani-reference <replay args...>"
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

fn prepare(mut args: impl Iterator<Item = String>) -> ExitCode {
    let mut manifest_path = None;
    let mut output_dir = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--manifest-path" if manifest_path.is_none() => manifest_path = args.next(),
            "--output-dir" if output_dir.is_none() => output_dir = args.next(),
            _ => usage(),
        }
    }
    let (Some(manifest_path), Some(output_dir)) = (manifest_path, output_dir) else {
        usage()
    };
    match KVerusContext.prepare_automation(&manifest_path, &output_dir) {
        Ok(receipt) => emit_json(&receipt),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("doctor") if args.next().is_none() => {
            println!(
                "{{\"default_engine\":\"{DEFAULT_ENGINE}\",\"kani_source_commit\":\"{KANI_SOURCE_COMMIT}\",\"proof_credit\":false}}"
            );
            ExitCode::SUCCESS
        }
        Some("setup") if args.next().is_none() => setup(),
        Some("discover" | "list") => {
            let Some(path) = args.next() else { usage() };
            if args.next().is_some() {
                usage()
            }
            discovery(&path)
        }
        Some("plan") => {
            let Some(path) = args.next() else { usage() };
            if args.next().is_some() {
                usage()
            }
            plan(&path)
        }
        Some("prepare") => prepare(args),
        Some("explain") if args.next().is_none() => {
            print!("{}", include_str!("../capabilities.json"));
            ExitCode::SUCCESS
        }
        Some("verify") => {
            let mut forwarded = vec!["verify".to_string()];
            forwarded.extend(args);
            delegate("verification/tools/verus_subset.py", forwarded)
        }
        Some("replay-kani-reference") => {
            delegate("verification/tools/replay_kani_fixture.py", args.collect::<Vec<_>>())
        }
        _ => usage(),
    }
}
