//! Development-only Governance smoke runner. Host owns files and child processes;
//! ic-agent owns IC protocol IO. No production CLI or canister mutation API is added.

mod attempt;
mod build;
mod command;
mod protocol;

#[cfg(test)]
mod tests;

use attempt::AttemptOperations;
use command::CommandOptions;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::path::canonicalize_allow_missing;
use serde_json::{Value, json};
use std::{
    env,
    error::Error,
    ffi::OsString,
    fs, io,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const GOVERNANCE: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";
const KINDS: [&str; 4] = [
    "economics",
    "metrics",
    "reward_event",
    "maturity_modulation",
];
const USAGE: &str = "Governance smoke runner\n\nCommands:\n  build\n  build-wasm\n  bundle\n  help\n  inspect-wasm <path>\n  local\n  verify-mainnet --canister <principal>";

fn invalid(message: &str) -> Box<dyn Error> {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}

///
/// Action
///
/// One supported development operation, admitted before build or network IO.
///

#[derive(Debug, Eq, PartialEq)]
enum Action {
    Build,
    BuildWasm,
    Bundle,
    Help,
    InspectWasm(PathBuf),
    Local,
    VerifyMainnet(String),
}

fn parse(args: &[OsString]) -> Result<Action> {
    fn text(value: &OsString) -> io::Result<&str> {
        value
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, USAGE))
    }
    match args {
        [] => Ok(Action::Help),
        [command] => match text(command)? {
            "build" => Ok(Action::Build),
            "build-wasm" => Ok(Action::BuildWasm),
            "bundle" => Ok(Action::Bundle),
            "help" | "--help" | "-h" => Ok(Action::Help),
            "local" => Ok(Action::Local),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
        },
        [command, path] if text(command)? == "inspect-wasm" => Ok(Action::InspectWasm(path.into())),
        [command, flag, principal]
            if text(command)? == "verify-mainnet" && text(flag)? == "--canister" =>
        {
            let principal = text(principal)?;
            candid::Principal::from_text(principal)?;
            Ok(Action::VerifyMainnet(principal.to_string()))
        }
        _ => Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE).into()),
    }
}

fn identity_home(root: &Path, project: &Path, target: &Path, selected: &Path) -> Result<()> {
    let refuse = || invalid("ICP_HOME must select persistent storage outside build/runtime state");
    if selected.as_os_str().is_empty() {
        return Err(refuse());
    }
    let home = canonicalize_allow_missing(selected, root)?;
    for disposable in [
        root.join("target"),
        target.to_path_buf(),
        project.join(".icp"),
    ] {
        if home.starts_with(canonicalize_allow_missing(&disposable, root)?) {
            return Err(refuse());
        }
    }
    Ok(())
}

fn icp_command(root: &Path, target: &Path, args: &[&str], selected: &Path) -> Result<Command> {
    let project = root.join("tests/canister");
    identity_home(root, &project, target, selected)?;
    let mut command = build::command(root, root.join(".tools/ic/bin/icp"), &[]);
    command
        .arg("--project-root-override")
        .arg(project)
        .args(args)
        .env("ICP_HOME", selected)
        .env("DO_NOT_TRACK", "1")
        .env_remove("ICP_NETWORK")
        .env_remove("ICP_ENVIRONMENT")
        .env_remove("ICP_PROJECT_ROOT");
    Ok(command)
}

///
/// Harness
///
/// Explicit project, configured Cargo output and this attempt's receipt destination.
///

struct Harness {
    root: PathBuf,
    target: PathBuf,
    identity_home: PathBuf,
    output: PathBuf,
}

impl AttemptOperations for Harness {
    fn icp(&mut self, args: &[&str], options: &CommandOptions<'_>) -> Result<String> {
        command::run(
            &mut icp_command(&self.root, &self.target, args, &self.identity_home)?,
            options,
        )
    }

    fn publish(&mut self, receipt: &Value) -> Result<()> {
        attempt::save_receipt(&self.output, receipt)
    }

    fn verify(&mut self, environment: &str, canister: &str, receipt: &mut Value) -> Result<()> {
        let endpoint = receipt["api_endpoint"]
            .as_str()
            .ok_or_else(|| invalid("missing api_endpoint"))?;
        let mut probe = protocol::AgentProbe::new(environment, endpoint, canister)?;
        let candid = fs::read(self.root.join("tests/canister/probe.did"))?;
        attempt::verify(
            &mut probe,
            canister,
            &self
                .root
                .join("target/canister-smoke/governance_probe.wasm"),
            &candid,
            &self.output,
            receipt,
            |receipt| attempt::save_receipt(&self.output, receipt),
        )
    }
}

fn receipt_directory(artifacts: &Path) -> Result<PathBuf> {
    fs::create_dir_all(artifacts)?;
    for sequence in 0..100 {
        let name = format!(
            "receipt-{}-{}-{sequence}",
            std::process::id(),
            time::OffsetDateTime::now_utc().unix_timestamp_nanos()
        );
        let directory = artifacts.join(name);
        match fs::DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    Err(invalid("cannot create a unique receipt directory"))
}

fn execute(action: Action) -> Result<()> {
    match action {
        Action::Help => {
            println!("{USAGE}");
            return Ok(());
        }
        Action::InspectWasm(path) => {
            println!("{}", Sha256Digest::compute(&protocol::read_wasm(&path)?));
            return Ok(());
        }
        _ => {}
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let target = build::target_directory(&root)?;
    if action == Action::BuildWasm {
        return build::build_wasm(&root, &target);
    }
    let pins = env::var_os("IC_TOOL_PINS")
        .unwrap_or_else(|| root.join("ci/ic-tools.tsv").into_os_string());
    let mut check = build::command(&root, "bash", &[]);
    check
        .arg(root.join("scripts/dev/install-ic-tools.sh"))
        .arg("--pins")
        .arg(pins)
        .arg("--check");
    command::run(&mut check, &CommandOptions::default())?;
    let mut harness = Harness {
        identity_home: env::var_os("ICP_HOME")
            .map_or_else(|| root.join(".icp-smoke-home"), PathBuf::from),
        root,
        target,
        output: PathBuf::new(),
    };
    let artifacts = harness.root.join("target/canister-smoke");
    match action {
        Action::Build => {
            harness.icp(
                &["build", "-e", "local"],
                &CommandOptions {
                    capture: false,
                    ..Default::default()
                },
            )?;
        }
        Action::Bundle => {
            fs::create_dir_all(&artifacts)?;
            let path = artifacts.join("governance-probe.tar.gz");
            let path = path
                .to_str()
                .ok_or_else(|| invalid("bundle path must be UTF-8"))?;
            harness.icp(
                &["project", "bundle", "-e", "mainnet-smoke", "--output", path],
                &CommandOptions {
                    capture: false,
                    ..Default::default()
                },
            )?;
        }
        Action::Local | Action::VerifyMainnet(_) => {
            let (environment, canister) = match &action {
                Action::Local => ("local", "governance-probe"),
                Action::VerifyMainnet(principal) => ("mainnet-smoke", principal.as_str()),
                _ => unreachable!(),
            };
            harness.output = receipt_directory(&artifacts)?.join("receipt.json");
            let mut receipt = json!({"schema_version":1, "status":"running", "environment":environment,
                "evidence_scope":if environment == "local" { "local_nns" } else { "mainnet" },
                "icp":command::run(&mut build::command(&harness.root, harness.root.join(".tools/ic/bin/icp"), &["--version"]), &CommandOptions::default())?,
                "rustc":command::run(&mut build::command(&harness.root, "rustc", &["--version"]), &CommandOptions::default())?,
                "started_at":attempt::utc_now()?, "point_in_time_guaranteed":false});
            println!("Receipt: {}", harness.output.display());
            attempt::run_attempt(&mut harness, environment, canister, &mut receipt)?;
        }
        Action::Help | Action::InspectWasm(_) | Action::BuildWasm => unreachable!(),
    }
    Ok(())
}

fn main() {
    let result = (|| {
        let action = parse(&env::args_os().skip(1).collect::<Vec<_>>())?;
        let _signals = command::SignalGuard::install()?;
        execute(action)?;
        command::check_interruption()
    })();
    if let Err(error) = result {
        eprintln!("{error}");
        if let Some(command) = error.downcast_ref::<command::CommandFailure>() {
            if let Some(stderr) = &command.stderr {
                eprint!("{}", String::from_utf8_lossy(stderr));
            }
            for cleanup in &command.cleanup_errors {
                eprintln!("Command cleanup failed: {cleanup}");
            }
        }
        std::process::exit(command::interruption().map_or(1, |signal| 128 + signal));
    }
}
