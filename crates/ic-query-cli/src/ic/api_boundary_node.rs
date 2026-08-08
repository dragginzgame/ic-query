//! Module: ic::api_boundary_node
//!
//! Responsibility: parse and dispatch certified API boundary-node commands.
//! Does not own: certificate authentication, state-tree projection, or rendering.
//! Boundary: exposes one complete live state-tree collection to the IC CLI facade.

use super::IcCommandError;
#[cfg(test)]
use super::parse_test_options;
use crate::cli::{
    clap::required_string,
    common::{
        COLLECTION_MODE_LIVE, OutputFormat, collection_help, current_unix_secs, json_arg,
        output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::ic::{
    DEFAULT_IC_STATE_SOURCE_ENDPOINT, IcApiBoundaryNodeRequest, build_ic_api_boundary_node_report,
    ic_api_boundary_node_report_text,
};

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

pub(super) fn run_matches(matches: &ArgMatches) -> Result<(), IcCommandError> {
    match matches.subcommand() {
        Some(("list", matches)) => run_list(matches),
        _ => unreachable!("clap requires a known ic api-boundary-node subcommand"),
    }
}

fn run_list(matches: &ArgMatches) -> Result<(), IcCommandError> {
    let options = ApiBoundaryNodeOptions::from_matches(matches);
    let request = IcApiBoundaryNodeRequest::new(options.source_endpoint, current_unix_secs()?);
    let report = build_ic_api_boundary_node_report(&request)?;
    write_text_or_json(options.format, &report, ic_api_boundary_node_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("api-boundary-node")
        .bin_name("icq ic api-boundary-node")
        .about("Inspect certified API boundary-node configuration")
        .subcommand(list_command())
}

fn list_command() -> ClapCommand {
    ClapCommand::new("list")
        .bin_name("icq ic api-boundary-node list")
        .about("List the complete certified API boundary-node state tree")
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_IC_STATE_SOURCE_ENDPOINT)
                .help("Mainnet IC API endpoint used for certified read_state"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApiBoundaryNodeOptions {
    format: OutputFormat,
    source_endpoint: String,
}

impl ApiBoundaryNodeOptions {
    fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            format: output_format(matches),
            source_endpoint: required_string(matches, "source-endpoint"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::render_help;

    #[test]
    fn usage_discloses_certificate_authority_and_semantic_limits() {
        let family = render_help(command());
        let list = render_help(list_command());

        assert!(family.contains("Usage: icq ic api-boundary-node [COMMAND]"));
        assert!(family.contains("list"));
        assert!(list.contains("exactly one response-bounded IC read_state request"));
        assert!(list.contains("built-in mainnet root key"));
        assert!(list.contains("HTTP gateways"));
        assert!(list.contains("operational"));
        assert!(list.contains("health"));
    }

    #[test]
    fn options_preserve_format_and_endpoint() {
        let options = parse_test_options(
            list_command(),
            &["--json", "--source-endpoint", "https://example.com"],
            ApiBoundaryNodeOptions::from_matches,
        )
        .expect("API boundary-node options");

        assert_eq!(options.format, OutputFormat::Json);
        assert_eq!(options.source_endpoint, "https://example.com");
    }
}
