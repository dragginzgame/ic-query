//! Module: ic
//!
//! Responsibility: compose and dispatch official IC report command families.
//! Does not own: family-specific parsing, transport, report construction, or rendering.
//! Boundary: exposes certified state and bounded Dashboard wiring to the top-level CLI.

mod api_boundary_node;
mod canister;
mod metrics;
mod network;
mod replica_version;

use crate::cli::common::CurrentUnixSecsError;
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::{IcApiBoundaryNodeHostError, IcHostError};
use std::io;
use thiserror::Error as ThisError;

///
/// IcCommandError
///
/// Errors surfaced while parsing or running an `icq ic` command.
///

#[derive(Debug, ThisError)]
pub enum IcCommandError {
    #[error("{0}")]
    Usage(String),

    #[error(transparent)]
    Host(#[from] IcHostError),

    /// Certified state-tree collection or report construction failed.
    #[error(transparent)]
    CertifiedState(#[from] IcApiBoundaryNodeHostError),

    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("api-boundary-node", matches)) => api_boundary_node::run_matches(matches),
        Some(("canister", matches)) => canister::run_matches(matches),
        Some(("metrics", matches)) => metrics::run_matches(matches),
        Some(("network", matches)) => network::run_matches(matches),
        Some(("replica-version", matches)) => replica_version::run_matches(matches),
        _ => unreachable!("clap requires a known ic subcommand"),
    }
}

pub fn command() -> ClapCommand {
    ClapCommand::new("ic")
        .bin_name("icq ic")
        .about("Inspect certified IC state and official Dashboard data")
        .subcommand(api_boundary_node::command())
        .subcommand(canister::command())
        .subcommand(metrics::command())
        .subcommand(network::command())
        .subcommand(replica_version::command())
}

#[cfg(test)]
fn parse_test_options<Options>(
    command: ClapCommand,
    args: &[&str],
    from_matches: fn(&ArgMatches) -> Options,
) -> Result<Options, IcCommandError> {
    let matches = crate::cli::clap::parse_matches(
        command,
        args.iter().copied().map(std::ffi::OsString::from),
    )
    .map_err(|error| IcCommandError::Usage(error.to_string()))?;
    Ok(from_matches(&matches))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;
    use std::ffi::OsString;

    #[test]
    fn usage_discloses_dashboard_command_families() {
        let usage = render_help(command());

        assert!(usage.contains("Usage: icq ic [COMMAND]"));
        assert!(usage.contains("api-boundary-node"));
        assert!(usage.contains("canister"));
        assert!(usage.contains("metrics"));
        assert!(usage.contains("network"));
        assert!(usage.contains("replica-version"));
    }

    #[test]
    fn family_and_nested_help_return_without_network_calls() {
        for args in [
            &["ic", "--help"][..],
            &["ic", "api-boundary-node", "--help"],
            &["ic", "api-boundary-node", "list", "--help"],
            &["ic", "canister", "--help"],
            &["ic", "canister", "info", "--help"],
            &["ic", "canister", "count", "--help"],
            &["ic", "canister", "page", "--help"],
            &["ic", "metrics", "--help"],
            &["ic", "network", "--help"],
            &["ic", "network", "boundary-node-data-centers", "--help"],
            &["ic", "network", "daily-stats", "--help"],
            &["ic", "replica-version", "--help"],
            &["ic", "replica-version", "info", "--help"],
            &["ic", "replica-version", "list", "--help"],
        ] {
            assert!(crate::run(args.iter().map(OsString::from)).is_ok());
        }
    }
}
