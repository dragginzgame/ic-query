mod cache;
mod cli;
mod cloud_engine;
mod ic;
mod icrc;
mod nns;
mod output;
mod progress;
mod sns;
mod storage;
mod system;

use crate::cli::clap::{parse_matches, prepare_command_tree, string_option};
use clap::{Arg, Command, error::ErrorKind};
use ic_query::subnet_catalog::MAINNET_NETWORK;
use std::ffi::OsString;
use thiserror::Error as ThisError;

const TOP_LEVEL_HELP_TEMPLATE: &str = "{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\nCommands:\n{subcommands}\n\nOptions:\n{options}{after-help}\n";

///
/// IcqCliError
///
/// Top-level CLI dispatch error.
///

#[derive(Debug, ThisError)]
pub enum IcqCliError {
    #[error("{0}")]
    Usage(String),

    #[error("cache: {0}")]
    Cache(#[from] cache::CacheCommandError),

    #[error("cloud-engine: {0}")]
    CloudEngine(#[from] cloud_engine::CloudEngineCommandError),

    #[error("nns: {0}")]
    Nns(#[from] nns::NnsCommandError),

    #[error("icrc: {0}")]
    Icrc(#[from] icrc::IcrcCommandError),

    #[error("ic: {0}")]
    Ic(#[from] ic::IcCommandError),

    #[error("sns: {0}")]
    Sns(#[from] sns::SnsCommandError),

    #[error("system: {0}")]
    System(#[from] system::SystemCommandError),
}

impl IcqCliError {
    /// Whether stdout closed before the command finished writing its report.
    #[must_use]
    pub fn is_broken_pipe(&self) -> bool {
        match self {
            Self::Cache(cache::CacheCommandError::Io(err))
            | Self::CloudEngine(cloud_engine::CloudEngineCommandError::Io(err))
            | Self::Ic(ic::IcCommandError::Io(err))
            | Self::Nns(nns::NnsCommandError::Io(err))
            | Self::Icrc(icrc::IcrcCommandError::Io(err))
            | Self::Sns(sns::SnsCommandError::Io(err))
            | Self::System(system::SystemCommandError::Io(err)) => {
                err.kind() == std::io::ErrorKind::BrokenPipe
            }
            Self::Usage(_)
            | Self::Cache(_)
            | Self::CloudEngine(_)
            | Self::Nns(_)
            | Self::Icrc(_)
            | Self::Ic(_)
            | Self::Sns(_)
            | Self::System(_) => false,
        }
    }

    /// Process exit code for this command error.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_)
            | Self::Ic(ic::IcCommandError::Usage(_))
            | Self::Nns(nns::NnsCommandError::Usage(_))
            | Self::Icrc(icrc::IcrcCommandError::Usage(_))
            | Self::Sns(sns::SnsCommandError::Usage(_))
            | Self::System(system::SystemCommandError::Usage(_)) => 2,
            Self::Cache(_)
            | Self::CloudEngine(_)
            | Self::Nns(_)
            | Self::Icrc(_)
            | Self::Ic(_)
            | Self::Sns(_)
            | Self::System(_) => 1,
        }
    }
}

/// Run the CLI from process arguments.
pub fn run_from_env() -> Result<(), IcqCliError> {
    run(std::env::args_os().skip(1))
}

/// Run the CLI from an argument iterator.
pub fn run<I>(args: I) -> Result<(), IcqCliError>
where
    I: IntoIterator<Item = OsString>,
{
    let command = cli_command();
    let matches = match parse_matches(command.clone(), args) {
        Ok(matches) => matches,
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp
                    | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                    | ErrorKind::DisplayVersion
            ) =>
        {
            print!("{error}");
            return Ok(());
        }
        Err(error) => return Err(IcqCliError::Usage(error.to_string())),
    };

    if let Some(help) = selected_namespace_help(command, &matches) {
        print!("{help}");
        return Ok(());
    }

    let selected_network = string_option(&matches, "network");
    let network = selected_network.as_deref().unwrap_or(MAINNET_NETWORK);
    let Some((command, matches)) = matches.subcommand() else {
        return Err(IcqCliError::Usage(usage()));
    };

    match command {
        "cache" => {
            reject_network_for_local_family(command, selected_network.as_deref())?;
            Ok(cache::run_matches(matches)?)
        }
        "cloud-engine" => Ok(cloud_engine::run_matches(matches, network)?),
        "ic" => {
            reject_network_for_endpoint_family(command, selected_network.as_deref())?;
            Ok(ic::run_matches(matches)?)
        }
        "icrc" => {
            reject_network_for_endpoint_family(command, selected_network.as_deref())?;
            Ok(icrc::run_matches(matches)?)
        }
        "nns" => Ok(nns::run_matches(matches, network)?),
        "sns" => Ok(sns::run_matches(
            matches,
            network,
            selected_network.is_some(),
        )?),
        "system" => Ok(system::run_matches(matches, network)?),
        _ => unreachable!("clap only returns declared top-level commands"),
    }
}

fn reject_network_for_endpoint_family(
    command: &str,
    selected_network: Option<&str>,
) -> Result<(), IcqCliError> {
    if selected_network.is_none() {
        return Ok(());
    }
    Err(IcqCliError::Usage(format!(
        "--network is not supported by `icq {command}`; use the command's --source-endpoint option to select its API endpoint\n\n{}",
        usage()
    )))
}

fn reject_network_for_local_family(
    command: &str,
    selected_network: Option<&str>,
) -> Result<(), IcqCliError> {
    if selected_network.is_none() {
        return Ok(());
    }
    Err(IcqCliError::Usage(format!(
        "--network is not supported by `icq {command}`; this command inspects every network under the local cache root\n\n{}",
        usage()
    )))
}

fn network_arg() -> Arg {
    Arg::new("network")
        .num_args(1)
        .long("network")
        .value_name("name")
        .value_parser([MAINNET_NETWORK])
        .help("Network identity for CloudEngine, NNS, SNS, and system commands; currently only ic")
}

fn top_level_command() -> Command {
    Command::new("icq")
        .version(env!("CARGO_PKG_VERSION"))
        .propagate_version(true)
        .about("Internet Computer metadata query CLI")
        .arg(network_arg())
        .subcommand_help_heading("Commands")
        .help_template(TOP_LEVEL_HELP_TEMPLATE)
        .after_help("Run `icq <command> --help` for command-specific help.")
        .subcommand(cache::command())
        .subcommand(cloud_engine::command())
        .subcommand(ic::command())
        .subcommand(icrc::command())
        .subcommand(nns::command())
        .subcommand(sns::command())
        .subcommand(system::command())
}

fn cli_command() -> Command {
    prepare_command_tree(top_level_command())
}

fn selected_namespace_help(mut command: Command, matches: &clap::ArgMatches) -> Option<String> {
    let mut selected_command = &mut command;
    let mut selected_matches = matches;
    while let Some((name, subcommand_matches)) = selected_matches.subcommand() {
        selected_command = selected_command.find_subcommand_mut(name)?;
        selected_matches = subcommand_matches;
    }

    let has_operational_subcommands = selected_command
        .get_subcommands()
        .any(|subcommand| subcommand.get_name() != "help");
    has_operational_subcommands.then(|| selected_command.render_help().to_string())
}

fn usage() -> String {
    let mut command = cli_command();
    command.render_help().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_lists_query_families_and_native_help_guidance() {
        let text = usage();

        assert!(text.contains("Usage: icq [OPTIONS] [COMMAND]"));
        assert!(text.contains("ic"));
        assert!(text.contains("Inspect official IC Dashboard data"));
        assert!(text.contains("cache"));
        assert!(text.contains("Inspect the local ic-query cache"));
        assert!(text.contains("cloud-engine"));
        assert!(text.contains("Inspect public CloudEngine control-plane metadata"));
        assert!(text.contains("icrc"));
        assert!(text.contains("Inspect generic ICRC ledgers"));
        assert!(text.contains("nns"));
        assert!(text.contains("Inspect NNS metadata"));
        assert!(text.contains("sns"));
        assert!(text.contains("Inspect SNS metadata"));
        assert!(text.contains("system"));
        assert!(text.contains("Inspect native IC system-canister metadata"));
        assert!(text.contains("Run `icq <command> --help`"));
    }

    #[test]
    fn every_subcommand_uses_alphabetical_help_order() {
        fn assert_equal_display_order(command: &Command, path: &mut Vec<String>) {
            for subcommand in command.get_subcommands() {
                path.push(subcommand.get_name().to_string());
                assert_eq!(
                    subcommand.get_display_order(),
                    0,
                    "non-alphabetical display rank for {}",
                    path.join(" ")
                );
                assert_equal_display_order(subcommand, path);
                path.pop();
            }
        }

        assert_equal_display_order(&cli_command(), &mut vec!["icq".to_string()]);
    }

    #[test]
    fn every_command_namespace_defaults_to_local_help() {
        fn assert_namespace_policy(command: &Command, path: &mut Vec<String>) {
            let has_operational_subcommands = command
                .get_subcommands()
                .any(|subcommand| subcommand.get_name() != "help");
            if has_operational_subcommands {
                assert!(
                    command.is_arg_required_else_help_set(),
                    "missing default help policy for {}",
                    path.join(" ")
                );
                assert!(
                    !command.is_subcommand_required_set(),
                    "terse missing-subcommand policy remains on {}",
                    path.join(" ")
                );
            }

            for subcommand in command
                .get_subcommands()
                .filter(|subcommand| subcommand.get_name() != "help")
            {
                path.push(subcommand.get_name().to_string());
                assert_namespace_policy(subcommand, path);
                path.pop();
            }
        }

        assert_namespace_policy(&cli_command(), &mut vec!["icq".to_string()]);
    }

    #[test]
    fn native_help_and_propagated_version_return_without_dispatch() {
        for args in [
            &["--help"][..],
            &["ic", "canister", "info", "--help"],
            &["cache", "status", "--help"],
            &[
                "icrc",
                "account",
                "transaction",
                "cache",
                "status",
                "--help",
            ],
            &["cloud-engine", "info", "--help"],
            &["nns", "topology", "providers", "--help"],
            &["sns", "proposal", "cache", "status", "--help"],
            &["system", "cycles", "--help"],
            &["--version"],
            &["nns", "subnet", "list", "--version"],
        ] {
            assert_run_ok(args);
        }
    }

    #[test]
    fn every_composed_command_path_supports_native_help() {
        fn collect_paths(
            command: &Command,
            prefix: &mut Vec<OsString>,
            paths: &mut Vec<Vec<OsString>>,
        ) {
            for subcommand in command.get_subcommands() {
                prefix.push(OsString::from(subcommand.get_name()));
                paths.push(prefix.clone());
                collect_paths(subcommand, prefix, paths);
                prefix.pop();
            }
        }

        let mut paths = Vec::new();
        collect_paths(&top_level_command(), &mut Vec::new(), &mut paths);
        assert!(!paths.is_empty());

        for mut path in paths {
            path.push(OsString::from("--help"));
            let error = parse_matches(top_level_command(), path.clone())
                .expect_err("native help must stop before typed dispatch");
            assert_eq!(
                error.kind(),
                ErrorKind::DisplayHelp,
                "unexpected result for {path:?}"
            );
        }
    }

    #[test]
    fn every_report_leaf_exposes_the_shared_json_flag() {
        fn assert_leaf_json(command: &Command, path: &mut Vec<String>) {
            let subcommands = command.get_subcommands().collect::<Vec<_>>();
            if subcommands.is_empty() {
                assert!(
                    command
                        .get_arguments()
                        .any(|argument| argument.get_id() == "json"),
                    "missing --json on {}",
                    path.join(" ")
                );
                return;
            }

            for subcommand in subcommands {
                path.push(subcommand.get_name().to_string());
                assert_leaf_json(subcommand, path);
                path.pop();
            }
        }

        assert_leaf_json(&top_level_command(), &mut vec!["icq".to_string()]);
    }

    #[test]
    fn clap_rejects_non_mainnet_and_command_local_network_options() {
        let error = run([
            OsString::from("--network"),
            OsString::from("local"),
            OsString::from("nns"),
            OsString::from("registry"),
            OsString::from("version"),
        ])
        .expect_err("non-mainnet network must fail in Clap");
        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("invalid value 'local'"));

        let error = run([
            OsString::from("nns"),
            OsString::from("registry"),
            OsString::from("version"),
            OsString::from("--network"),
            OsString::from("ic"),
        ])
        .expect_err("network remains a top-level option");
        assert_eq!(error.exit_code(), 2);
        assert!(
            error
                .to_string()
                .contains("unexpected argument '--network'")
        );
    }

    #[test]
    fn network_is_rejected_for_endpoint_identified_families() {
        for args in [
            &["--network", "ic", "ic", "canister", "count"][..],
            &[
                "--network",
                "ic",
                "icrc",
                "ledger",
                "token",
                "ryjl3-tyaaa-aaaaa-aaaba-cai",
            ],
        ] {
            let error = run(args.iter().map(OsString::from))
                .expect_err("endpoint-identified families must reject --network");
            assert_eq!(error.exit_code(), 2);
            assert!(error.to_string().contains("--source-endpoint"));
        }
    }

    #[test]
    fn explicit_network_is_rejected_for_cross_network_cache_status() {
        let error = run([
            OsString::from("--network"),
            OsString::from("ic"),
            OsString::from("cache"),
            OsString::from("status"),
        ])
        .expect_err("cross-network cache status must reject one selected network");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("every network"));
    }

    #[test]
    fn explicit_network_is_rejected_for_local_reward_diff() {
        let error = run([
            OsString::from("--network"),
            OsString::from("ic"),
            OsString::from("sns"),
            OsString::from("reward"),
            OsString::from("diff"),
            OsString::from("before.json"),
            OsString::from("after.json"),
        ])
        .expect_err("local reward diff must reject explicit network identity");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("local-only"));
    }

    #[test]
    fn targeted_sns_leaves_require_their_identifiers() {
        for args in [
            &["sns", "neuron", "list"][..],
            &["sns", "proposal", "refresh"][..],
            &["sns", "reward", "checkpoint"][..],
        ] {
            let error = run(args.iter().map(OsString::from))
                .expect_err("targeted SNS operation must require an SNS selector");
            assert_eq!(error.exit_code(), 2);
            assert!(error.to_string().contains("<id|root-principal>"));
        }
    }

    #[test]
    fn typed_cli_errors_preserve_exit_and_broken_pipe_semantics() {
        for usage in [
            IcqCliError::Ic(ic::IcCommandError::Usage("bad input".to_string())),
            IcqCliError::Icrc(icrc::IcrcCommandError::Usage("bad input".to_string())),
            IcqCliError::System(system::SystemCommandError::Usage("bad input".to_string())),
        ] {
            assert_eq!(usage.exit_code(), 2);
            assert!(!usage.is_broken_pipe());
        }

        for broken_pipe in [
            IcqCliError::CloudEngine(cloud_engine::CloudEngineCommandError::Io(
                std::io::Error::from(std::io::ErrorKind::BrokenPipe),
            )),
            IcqCliError::Ic(ic::IcCommandError::Io(std::io::Error::from(
                std::io::ErrorKind::BrokenPipe,
            ))),
            IcqCliError::Icrc(icrc::IcrcCommandError::Io(std::io::Error::from(
                std::io::ErrorKind::BrokenPipe,
            ))),
            IcqCliError::System(system::SystemCommandError::Io(std::io::Error::from(
                std::io::ErrorKind::BrokenPipe,
            ))),
        ] {
            assert_eq!(broken_pipe.exit_code(), 1);
            assert!(broken_pipe.is_broken_pipe());
        }
    }

    fn assert_run_ok(args: &[&str]) {
        let args = args.iter().copied().map(OsString::from).collect::<Vec<_>>();
        if let Err(err) = run(args.clone()) {
            panic!("expected {args:?} to succeed, got {err}");
        }
    }
}
