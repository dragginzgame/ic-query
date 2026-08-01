//! Module: ic
//!
//! Responsibility: compose and dispatch official IC Dashboard command families.
//! Does not own: family-specific parsing, REST transport, report construction, or rendering.
//! Boundary: exposes bounded live Dashboard command wiring to the top-level CLI.

mod canister;
mod metrics;
mod network;

use crate::cli::common::CurrentUnixSecsError;
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::IcHostError;
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

    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("canister", matches)) => canister::run_matches(matches),
        Some(("metrics", matches)) => metrics::run_matches(matches),
        Some(("network", matches)) => network::run_matches(matches),
        _ => unreachable!("clap requires a known ic subcommand"),
    }
}

pub fn command() -> ClapCommand {
    ClapCommand::new("ic")
        .bin_name("icq ic")
        .about("Inspect official IC Dashboard data")
        .subcommand_required(true)
        .subcommand(canister::command())
        .subcommand(metrics::command())
        .subcommand(network::command())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;
    use std::ffi::OsString;

    #[test]
    fn usage_discloses_dashboard_command_families() {
        let usage = render_help(command());

        assert!(usage.contains("Usage: icq ic <COMMAND>"));
        assert!(usage.contains("canister"));
        assert!(usage.contains("metrics"));
        assert!(usage.contains("network"));
    }

    #[test]
    fn family_and_nested_help_return_without_network_calls() {
        for args in [
            &["ic", "--help"][..],
            &["ic", "canister", "--help"],
            &["ic", "canister", "info", "--help"],
            &["ic", "canister", "count", "--help"],
            &["ic", "canister", "page", "--help"],
            &["ic", "metrics", "--help"],
            &["ic", "network", "--help"],
            &["ic", "network", "boundary-node-data-centers", "--help"],
            &["ic", "network", "daily-stats", "--help"],
        ] {
            assert!(crate::run(args.iter().map(OsString::from)).is_ok());
        }
    }
}
