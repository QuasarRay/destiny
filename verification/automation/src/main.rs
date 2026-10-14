#![forbid(unsafe_code)]

use destiny_verification_automation::{
    DEFAULT_ENGINE, KANI_SOURCE_COMMIT, normalize_autoharness_metadata, parse_metadata,
};
use std::{env, fs::File, process::ExitCode};

fn usage() -> ! {
    eprintln!("usage: destiny-verification-automation doctor | discover <kani-metadata.json>");
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("doctor") if args.next().is_none() => {
            println!(
                "{{"default_engine":"{DEFAULT_ENGINE}","kani_source_commit":"{KANI_SOURCE_COMMIT}","proof_credit":false}}"
            );
            ExitCode::SUCCESS
        }
        Some("discover") => {
            let Some(path) = args.next() else { usage() };
            if args.next().is_some() { usage() }
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
        _ => usage(),
    }
}
