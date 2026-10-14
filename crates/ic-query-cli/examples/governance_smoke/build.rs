//! Probe build, source fingerprint and Wasm metadata publication.

use crate::{
    Result,
    command::{self, CommandOptions, StderrMode},
    invalid, protocol,
};
use ic_build::artifacts::{InputEvidence, InputEvidenceSpec, InputEvidenceValue, WasmBuildSpec};
use ic_host_artifacts::artifact::hash_reader;
use ic_host_fs::durable::write_bytes;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
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

pub fn source_digest(root: &Path) -> Result<String> {
    let required = [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/ic-query/Cargo.toml",
        "crates/ic-query-cli/Cargo.toml",
        "tests/canister/probe.did",
    ]
    .map(|path| root.join(path));
    let roots = [
        root.join("crates/ic-query"),
        root.join("crates/ic-query-cli/examples/governance_smoke"),
    ];
    let spec = required
        .iter()
        .chain(&roots)
        .fold(InputEvidenceSpec::new(Vec::new()), |spec, path| {
            spec.with_input(path)
        });
    let inputs = InputEvidence::capture(&spec)?;
    for path in required.iter().chain(&roots) {
        if matches!(
            inputs.entries().get(path),
            Some(InputEvidenceValue::Missing)
        ) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing probe input: {}", path.display()),
            )
            .into());
        }
    }
    let mut hash = Sha256::new();
    for (path, value) in inputs.entries() {
        if !matches!(value, InputEvidenceValue::File { .. })
            || (!required.contains(path)
                && path.extension().is_none_or(|extension| extension != "rs"))
        {
            continue;
        }
        // Preserve Query's relative-path/NUL/raw-bytes/NUL fingerprint contract.
        hash.update(path.strip_prefix(root)?.as_os_str().as_encoded_bytes());
        hash.update([0]);
        io::copy(&mut fs::File::open(path)?, &mut hash)?;
        hash.update([0]);
    }
    inputs.validate_after(&inputs)?;
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
    let spec = WasmBuildSpec::new(root, target, &["ic-query"], "release")
        .with_example("governance_probe")
        .with_target("wasm32-unknown-unknown")
        .with_cargo_program("cargo")
        .with_cargo_profile_args([
            "--release",
            "--no-default-features",
            "--features",
            "canister",
            "--locked",
            "--offline",
        ]);
    command::run(
        &mut spec.cargo_build_command()?,
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
