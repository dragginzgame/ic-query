//! Module: ic::api_boundary_node
//!
//! Responsibility: parse and dispatch certified API boundary-node commands.
//! Does not own: certificate authentication, state-tree projection, or rendering.
//! Boundary: exposes one complete live state-tree collection to the IC CLI facade.

use super::IcCommandError;
use crate::cli::clap::with_options;
use crate::cli::common::{
    COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs,
    write_text_or_json,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::DEFAULT_IC_STATE_SOURCE_ENDPOINT;
use ic_query::ic::IcApiBoundaryNodeRequest;
use ic_query::ic::build_ic_api_boundary_node_report;
use ic_query_text::ic::ic_api_boundary_node_report_text;

const LIST_HELP_AFTER: &str = "\
Examples:
  icq ic api-boundary-node list
  icq ic api-boundary-node list --json

This command makes exactly one response-bounded IC read_state request and
authenticates its certificate with the built-in mainnet root key. It returns
the complete certified api_boundary_nodes subtree at one certificate time,
including node principals, domains, and IPv4/IPv6 addresses. The fixed NNS
Registry effective canister only routes the read_state request; Registry
canister data is not queried. No cache or per-node follow-up is used.

Rows identify configured API boundary nodes, not HTTP gateways or operational
health, reachability, latency, ownership, or physical location.";

pub(super) async fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_list(matches).await,
        _ => unreachable!("clap requires a known ic api-boundary-node subcommand"),
    }
}

async fn run_list(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = SourceOutputOptions::from_matches(matches);
    let format = options.format();
    let request = IcApiBoundaryNodeRequest::new(options.source_endpoint, current_unix_secs()?);
    let report = build_ic_api_boundary_node_report(&request).await?;
    write_text_or_json(format, &report, ic_api_boundary_node_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("api-boundary-node")
        .about("Inspect certified API boundary-node configuration")
        .subcommand(list_command())
}

fn list_command() -> ClapCommand {
    with_options::<crate::cli::common::SourceOutputOptions>(
        ClapCommand::new("list")
            .about("List the complete certified API boundary-node state tree")
            .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER)),
    )
    .mut_arg("source-endpoint", |arg| {
        arg.default_value(DEFAULT_IC_STATE_SOURCE_ENDPOINT)
            .help("Mainnet IC API endpoint used for certified read_state")
    })
}

#[cfg(test)]
mod tests;
