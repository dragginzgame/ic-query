//! Probe build, source fingerprint and Wasm metadata publication.

use crate::{
    Result,
    command::{self, CommandOptions, StderrMode},
    invalid, protocol,
};
use ic_build::{
    artifacts::{
        CargoInputProbePolicy, InputEvidence, InputEvidenceSpec, InputEvidenceValue,
        ResolvedCargoBuildInputs, WasmBuildSpec, resolve_cargo_build_inputs_with_probe_policy,
    },
    cargo::{
        environment::ConfigurationLimits,
        observation::schema::Metadata,
        selection::{CargoFeatureSelection, CargoSelection},
    },
};
use ic_host_artifacts::{
    artifact::hash_reader,
    wasm::{CustomSection, encode_custom_section},
};
use ic_host_fs::durable::{PublicationMode, WriteOptions, write_bytes};
use serde_json::json;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

pub fn command(root: &Path, program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    command
}

pub fn check_ic_tools(root: &Path) -> Result<()> {
    command::run(
        &mut command(root, "make", &["--no-print-directory", "ic-tools-check"]),
        &CommandOptions::default(),
    )?;
    Ok(())
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
    let metadata: Metadata = serde_json::from_str(&metadata)?;
    Ok(metadata.target_directory.into_std_path_buf())
}

pub fn source_digest(root: &Path) -> Result<String> {
    let required = [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/ic-query/Cargo.toml",
        "crates/ic-query-cli/Cargo.toml",
        "tests/canister/probe-wire.json",
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
    let digest = inputs.digest_relative_to(root)?;
    inputs.validate_after(&spec)?;
    Ok(digest.to_string())
}

pub fn append_metadata_section(bytes: &mut Vec<u8>, name: &str, contents: &[u8]) -> Result<()> {
    let name = format!("icp:public {name}");
    let section = encode_custom_section(
        CustomSection {
            name: &name,
            data: contents,
        },
        protocol::WASM_BYTES.saturating_sub(bytes.len()),
    )?;
    bytes.extend(section);
    Ok(())
}

pub fn probe_spec(root: &Path, target: &Path) -> WasmBuildSpec {
    WasmBuildSpec::new(
        root,
        target,
        &["ic-query"],
        env::vars_os(),
        ConfigurationLimits {
            file_bytes: 1024 * 1024,
            total_bytes: 8 * 1024 * 1024,
            files: 128,
            depth: 16,
        },
    )
    .with_example("governance_probe")
    .with_target("wasm32-unknown-unknown")
    .with_cargo_program("cargo")
    .with_cargo_selection(CargoSelection {
        profile: Some("release".into()),
        features: CargoFeatureSelection {
            features: ["canister".into()].into(),
            default_features: false,
            all_features: false,
        },
        locked: true,
        offline: true,
    })
}

pub fn resolve_probe_inputs(
    spec: &WasmBuildSpec,
    timeout: Duration,
) -> Result<ResolvedCargoBuildInputs> {
    let mut cancelled = || command::interruption().is_some();
    Ok(resolve_cargo_build_inputs_with_probe_policy(
        spec,
        &mut CargoInputProbePolicy {
            deadline: Some(Instant::now() + timeout),
            cleanup: command::cleanup_policy(),
            cancelled: &mut cancelled,
        },
    )?)
}

pub fn build_wasm(root: &Path, target: &Path) -> Result<()> {
    let output = env::var_os("ICP_WASM_OUTPUT_PATH")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("build-wasm requires ICP_WASM_OUTPUT_PATH"))?;
    let spec = probe_spec(root, target);
    let timeout = CommandOptions::default().timeout;
    let inputs = resolve_probe_inputs(&spec, timeout)?;
    let [path] = inputs.artifact_paths()? else {
        return Err(invalid(
            "Governance probe requires exactly one Wasm artifact",
        ));
    };
    command::run(
        &mut spec.cargo_build_command()?,
        &CommandOptions {
            capture: false,
            ..Default::default()
        },
    )?;
    let after = resolve_probe_inputs(&spec, timeout)?;
    if after.fingerprint() != inputs.fingerprint()
        || after.validation_digest() != inputs.validation_digest()
    {
        return Err(invalid(
            "Governance probe inputs changed during compilation",
        ));
    }
    let mut bytes = protocol::read_wasm(path)?;
    let rustc = std::str::from_utf8(&inputs.facts().rustc_identity)?
        .lines()
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("captured compiler identity is empty"))?;
    let build = json!({"schema_version":1,
        "rustc":rustc,
        "cargo_lock_sha256":hash_reader(fs::File::open(root.join("Cargo.lock"))?, u64::MAX)?.sha256.to_string(),
        "source_sha256":source_digest(root)?});
    append_metadata_section(
        &mut bytes,
        "ic-query:protocol",
        &fs::read(root.join("tests/canister/probe-wire.json"))?,
    )?;
    append_metadata_section(&mut bytes, "ic-query:build", &serde_json::to_vec(&build)?)?;
    let artifacts = root.join("target/canister-smoke");
    fs::create_dir_all(&artifacts)?;
    let options = WriteOptions {
        mode: PublicationMode::Replace,
        permissions: 0o666,
    };
    write_bytes(&artifacts.join("governance_probe.wasm"), options, &bytes)?;
    write_bytes(Path::new(&output), options, &bytes)?;
    Ok(())
}
