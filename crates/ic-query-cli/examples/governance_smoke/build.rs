//! Probe build, source fingerprint and Wasm metadata publication.

use crate::{
    Result,
    command::{self, CommandOptions, StderrMode},
    invalid, protocol,
};
use ic_host_artifacts::artifact::hash_reader;
use ic_host_fs::durable::write_bytes;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

pub fn command(root: &Path, program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    command
}

pub fn target_directory(root: &Path) -> Result<PathBuf> {
    let metadata = command::run(
        &mut command(
            root,
            "cargo",
            &[
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--locked",
                "--offline",
            ],
        ),
        &CommandOptions {
            stderr: StderrMode::Capture,
            ..Default::default()
        },
    )?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata)?;
    Ok(PathBuf::from(
        metadata["target_directory"]
            .as_str()
            .ok_or_else(|| invalid("Cargo metadata omitted target_directory"))?,
    ))
}

fn rust_sources(directory: &Path, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            rust_sources(&entry.path(), paths)?;
        } else if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            paths.insert(entry.path());
        }
    }
    Ok(())
}

pub fn source_digest(root: &Path) -> Result<String> {
    let mut paths: BTreeSet<_> = [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/ic-query/Cargo.toml",
        "crates/ic-query-cli/Cargo.toml",
        "tests/canister/probe.did",
    ]
    .map(|path| root.join(path))
    .into_iter()
    .collect();
    rust_sources(&root.join("crates/ic-query"), &mut paths)?;
    rust_sources(
        &root.join("crates/ic-query-cli/examples/governance_smoke"),
        &mut paths,
    )?;
    let mut hash = Sha256::new();
    for path in paths {
        hash.update(path.strip_prefix(root)?.as_os_str().as_encoded_bytes());
        hash.update([0]);
        io::copy(&mut fs::File::open(path)?, &mut hash)?;
        hash.update([0]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn leb128(mut number: usize, bytes: &mut Vec<u8>) {
    while number >= 128 {
        bytes.push(u8::try_from(number & 127).expect("seven-bit LEB128 digit") | 128);
        number >>= 7;
    }
    bytes.push(u8::try_from(number).expect("last LEB128 digit"));
}

pub fn metadata_section(name: &str, contents: &[u8]) -> Vec<u8> {
    let name = format!("icp:public {name}");
    let mut payload = Vec::new();
    leb128(name.len(), &mut payload);
    payload.extend_from_slice(name.as_bytes());
    payload.extend_from_slice(contents);
    let mut section = vec![0];
    leb128(payload.len(), &mut section);
    section.extend(payload);
    section
}

pub fn build_wasm(root: &Path, target: &Path) -> Result<()> {
    let output = env::var_os("ICP_WASM_OUTPUT_PATH")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("build-wasm requires ICP_WASM_OUTPUT_PATH"))?;
    command::run(
        &mut command(
            root,
            "cargo",
            &[
                "build",
                "-p",
                "ic-query",
                "--example",
                "governance_probe",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--no-default-features",
                "--features",
                "canister",
                "--locked",
                "--offline",
            ],
        ),
        &CommandOptions {
            capture: false,
            ..Default::default()
        },
    )?;
    let path = target.join("wasm32-unknown-unknown/release/examples/governance_probe.wasm");
    let mut bytes = protocol::read_wasm(&path)?;
    let build = json!({"schema_version":1,
        "rustc":command::run(&mut command(root,"rustc", &["--version"]), &CommandOptions::default())?,
        "cargo_lock_sha256":hash_reader(fs::File::open(root.join("Cargo.lock"))?, u64::MAX)?.sha256.to_string(),
        "source_sha256":source_digest(root)?});
    bytes.extend(metadata_section(
        "candid:service",
        &fs::read(root.join("tests/canister/probe.did"))?,
    ));
    bytes.extend(metadata_section(
        "ic-query:build",
        &serde_json::to_vec(&build)?,
    ));
    let artifacts = root.join("target/canister-smoke");
    fs::create_dir_all(&artifacts)?;
    write_bytes(&artifacts.join("governance_probe.wasm"), &bytes)?;
    write_bytes(Path::new(&output), &bytes)?;
    Ok(())
}
