#![forbid(unsafe_code)]

use destiny_verification_automation::{
    DEFAULT_ENGINE, KANI_SOURCE_COMMIT, normalize_autoharness_metadata, parse_metadata,
};
use std::{
    env,
    ffi::OsString,
    fs::File,
    path::{Path, PathBuf},
    process::{Command, ExitCode, ExitStatus},
};

fn usage() -> ! {
    eprintln!(
        "usage: destiny-verification-automation \
         doctor | setup | discover <kani-metadata.json> | list <kani-metadata.json> | \
         explain | verify <verus-subset args...> | replay-kani-reference <replay args...>"
    );
    std::process::exit(2);
}

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn python() -> OsString {
    env::var_os("DESTINY_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            OsString::from("python")
        } else {
            OsString::from("python3")
        }
    })
}

fn run_python(script: &str, args: impl IntoIterator<Item = String>) -> Result<ExitStatus, std::io::Error> {
    Command::new(python())
        .arg("-B")
        .arg(project_root().join(script))
        .args(args)
        .status()
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

fn discovery(path: String) -> ExitCode {
    let result = File::open(path)
        .map_err(|e| e.to_string())
        .and_then(|file| parse_metadata(file).map_err(|e| e.to_string()))
        .map(|metadata| normalize_autoharness_metadata(&metadata))
        .and_then(|report| serde_json::to_string_pretty(&report).map_err(|e| e.to_string()));
    match result {
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
            discovery(path)
        }
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
