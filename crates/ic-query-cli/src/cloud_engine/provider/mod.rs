//! Module: cloud_engine::provider
//!
//! Responsibility: parse and dispatch official Dashboard CloudEngine provider reports.
//! Does not own: HTTP transport, source validation, report rendering, or caching.
//! Boundary: provider identity means an NNS node-provider principal, not an engine canister.

use super::{CloudEngineCommandError, report_args};
use crate::cli::{
    clap::{required_string, value_arg},
    common::{
        COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs,
        write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cloud_engine::CloudEngineProviderInfoRequest;
use ic_query::cloud_engine::CloudEngineProviderListRequest;
use ic_query::cloud_engine::DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::cloud_engine::build_cloud_engine_provider_info_report;
use ic_query::cloud_engine::build_cloud_engine_provider_list_report;
use ic_query_text::cloud_engine::cloud_engine_provider_info_report_text;
use ic_query_text::cloud_engine::cloud_engine_provider_list_report_text;

const SOURCE_ENDPOINT_HELP: &str = "Official Dashboard v3 endpoint used for provider metadata";

const INFO_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine provider info rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae
  icq cloud-engine provider info rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae --json

This command makes one exact official Dashboard request for an NNS node-provider
principal. It preserves the provider's raw CloudEngine and general node counts,
and explicitly reports when the provider has no CloudEngine evidence. Dashboard
responses are off-chain, uncertified, and not point-in-time guaranteed.";
const LIST_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine provider list
  icq cloud-engine provider list --json

This command makes one official Dashboard request for the complete node-provider
resource, validates every returned row up to 1,000, and then retains providers
with explicit CloudEngine counts or locations. Dashboard responses are off-chain,
uncertified, and not point-in-time guaranteed.";

pub(super) async fn run_matches(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), CloudEngineCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches, network).await,
        Some(("list", matches)) => run_list(matches, network).await,
        _ => unreachable!("clap requires a known cloud-engine provider subcommand"),
    }
}

async fn run_info(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(current_unix_secs()?);
    let request = CloudEngineProviderInfoRequest::new(
        network,
        dashboard,
        required_string(matches, "node-provider-id"),
    );
    let report = build_cloud_engine_provider_info_report(&request).await?;
    write_text_or_json(format, &report, cloud_engine_provider_info_report_text)
}

async fn run_list(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(current_unix_secs()?);
    let request = CloudEngineProviderListRequest::new(network, dashboard);
    let report = build_cloud_engine_provider_list_report(&request).await?;
    write_text_or_json(format, &report, cloud_engine_provider_list_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("provider")
        .bin_name("icq cloud-engine provider")
        .about("Inspect official Dashboard CloudEngine provider metadata")
        .subcommand(info_command())
        .subcommand(list_command())
        .after_help(
            "Examples:\n  icq cloud-engine provider list\n  icq cloud-engine provider info <node-provider-id>",
        )
}

fn info_command() -> ClapCommand {
    report_args(
        ClapCommand::new("info")
            .bin_name("icq cloud-engine provider info")
            .about("Show one exact node provider and its CloudEngine evidence")
            .arg(
                value_arg("node-provider-id")
                    .required(true)
                    .value_name("node-provider-id")
                    .help("Exact NNS node-provider principal; not an engine canister principal"),
            ),
        DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
        SOURCE_ENDPOINT_HELP,
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER))
}

fn list_command() -> ClapCommand {
    report_args(
        ClapCommand::new("list")
            .bin_name("icq cloud-engine provider list")
            .about("List providers carrying explicit CloudEngine evidence"),
        DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
        SOURCE_ENDPOINT_HELP,
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

#[cfg(test)]
mod tests;
