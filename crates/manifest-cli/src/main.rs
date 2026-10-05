use std::{env, fs, process::ExitCode};

use pvrpc_protocol::SnapshotManifest;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    let command = arguments.next();
    let path = arguments.next();

    if command.as_deref() != Some("validate") || path.is_none() || arguments.next().is_some() {
        return Err("usage: pvrpc-manifest validate <manifest.json>".into());
    }

    let path = path.expect("path presence was checked");
    let contents =
        fs::read_to_string(&path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let manifest: SnapshotManifest =
        serde_json::from_str(&contents).map_err(|error| format!("cannot parse {path}: {error}"))?;
    manifest
        .validate()
        .map_err(|error| format!("invalid manifest: {error}"))?;

    println!("valid snapshot manifest");
    Ok(())
}
