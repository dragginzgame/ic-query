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
use ic_query::ic::IcApiBoundaryNodeHostError;
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

pub async fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("api-boundary-node", matches)) => api_boundary_node::run_matches(matches).await,
        Some(("canister", matches)) => canister::run_matches(matches).await,
        Some(("metrics", matches)) => metrics::run_matches(matches).await,
        Some(("network", matches)) => network::run_matches(matches).await,
        Some(("replica-version", matches)) => replica_version::run_matches(matches).await,
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
mod tests;
