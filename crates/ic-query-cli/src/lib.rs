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

pub use crate::cli::clap::prepare_command_tree;

use crate::cli::clap::{parse_matches, string_option};
use clap::{Arg, Command, error::ErrorKind};
use ic_query::subnet_catalog::MAINNET_NETWORK;
use std::ffi::OsString;
use thiserror::Error as ThisError;

const ENDPOINT_NETWORK_ADVICE: &str =
    "use the command's --source-endpoint option to select its API endpoint";

const TOP_LEVEL_HELP_TEMPLATE: &str = "{name} {version}\n{about-with-newline}\n{usage-heading} {usage}\n\nCommands:\n{subcommands}\n\nOptions:\n{options}{after-help}\n";

///
/// IcqCliError
///
/// Top-level CLI dispatch error.
///

#[derive(Debug, ThisError)]
pub enum IcqCliError {
    /// Writing command help or version output failed.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Usage(String),

    #[error("cache: {0}")]
    Cache(#[from] cache::CacheCommandError),

    #[error("cloud-engine: {0}")]
    CloudEngine(#[from] cloud_engine::CloudEngineCommandError),

    #[error("nns: {0}{advice}", advice = cli::diagnostics::nns_advice(.0))]
    Nns(#[from] nns::NnsCommandError),

    #[error("icrc: {0}")]
    Icrc(#[from] icrc::IcrcCommandError),

    #[error("ic: {0}")]
    Ic(#[from] ic::IcCommandError),

    #[error("sns: {0}{advice}", advice = cli::diagnostics::sns_advice(.0))]
    Sns(#[from] sns::SnsCommandError),

    #[error("system: {0}")]
    System(#[from] system::SystemCommandError),
}

impl IcqCliError {
    /// Whether stdout closed before the command finished writing its report.
    #[must_use]
    pub fn is_broken_pipe(&self) -> bool {
        match self {
            Self::Io(err)
            | Self::Cache(cache::CacheCommandError::Io(err))
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
            Self::Io(_)
            | Self::Cache(_)
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
#[expect(
    clippy::needless_collect,
    reason = "ArgsOs is not Send; own the arguments before awaiting command dispatch."
)]
pub async fn run_from_env() -> Result<(), IcqCliError> {
    run(std::env::args_os().skip(1).collect::<Vec<_>>()).await
}

/// Run the CLI from an argument iterator.
pub async fn run<I>(args: I) -> Result<(), IcqCliError>
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
            output::write_help(&error.to_string())?;
            return Ok(());
        }
        Err(error) => return Err(IcqCliError::Usage(error.to_string())),
    };

    if let Some(help) = selected_namespace_help(command, &matches) {
        output::write_help(&help)?;
        return Ok(());
    }

    let selected_network = string_option(&matches, "network");
    let network = selected_network.as_deref().unwrap_or(MAINNET_NETWORK);
    let Some((command, matches)) = matches.subcommand() else {
        return Err(IcqCliError::Usage(usage()));
    };

    match command {
        "cache" => {
            reject_network_option(
                command,
                selected_network.as_deref(),
                "this command inspects every network under the local cache root",
            )?;
            Ok(cache::run_matches(matches)?)
        }
        "cloud-engine" => Ok(cloud_engine::run_matches(matches, network).await?),
        "ic" => {
            reject_network_option(
                command,
                selected_network.as_deref(),
                ENDPOINT_NETWORK_ADVICE,
            )?;
            Ok(ic::run_matches(matches).await?)
        }
        "icrc" => {
            reject_network_option(
                command,
                selected_network.as_deref(),
                ENDPOINT_NETWORK_ADVICE,
            )?;
            Ok(icrc::run_matches(matches).await?)
        }
        "nns" => Ok(nns::run_matches(matches, network).await?),
        "sns" => Ok(sns::run_matches(matches, network).await?),
        "system" => Ok(system::run_matches(matches, network).await?),
        _ => unreachable!("clap only returns declared top-level commands"),
    }
}

fn reject_network_option(
    command: &str,
    selected_network: Option<&str>,
    explanation: &str,
) -> Result<(), IcqCliError> {
    if selected_network.is_none() {
        return Ok(());
    }
    Err(IcqCliError::Usage(format!(
        "--network is not supported by `icq {command}`; {explanation}\n\n{}",
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
mod tests;
