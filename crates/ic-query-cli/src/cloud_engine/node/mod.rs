//! Module: cloud_engine::node
//!
//! Responsibility: parse and dispatch official Dashboard CloudEngine Type4 node reports.
//! Does not own: HTTP transport, source validation, report rendering, or caching.
//! Boundary: requests the explicit Type4 and complete current status scope.

use super::{CloudEngineCommandError, report_args};
use crate::cli::clap::{parsed_options, with_options};
use crate::cli::common::{
    COLLECTION_MODE_LIVE, SourceOutputOptions, collection_help, current_unix_secs,
    write_text_or_json,
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cloud_engine::CloudEngineNodeInfoRequest;
use ic_query::cloud_engine::CloudEngineNodeListRequest;
use ic_query::cloud_engine::DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT;
use ic_query::cloud_engine::build_cloud_engine_node_info_report;
use ic_query::cloud_engine::build_cloud_engine_node_list_report;
use ic_query_text::cloud_engine::cloud_engine_node_info_report_text;
use ic_query_text::cloud_engine::cloud_engine_node_list_report_text;

const SOURCE_ENDPOINT_HELP: &str =
    "Official Dashboard v3 endpoint used for Type4 node observations";

const INFO_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine node info 53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe
  icq cloud-engine node info 53amq-7hjxu-6lxaj-o2sp6-kmngy-qa22h-b7bo6-oeyyn-fkqnv-7tauf-7qe --json

This command makes one exact official Dashboard node request and requires the
returned reward type to be Type4. The observation is off-chain, uncertified,
not point-in-time guaranteed, and does not replace Registry or native
CloudEngine control-plane evidence.";
const LIST_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine node list
  icq cloud-engine node list --node-provider-id bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe
  icq cloud-engine node list --json

This command makes one official Dashboard request with the explicit Type4
reward filter and all four currently documented statuses. It accepts at most
10,000 rows and may apply one exact provider filter remotely. A null
cloud_engine_subnet_id is preserved as an unassigned observation. The result
is off-chain, uncertified, uncached, and not point-in-time guaranteed.";

pub(super) async fn run_matches(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), CloudEngineCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches, network).await,
        Some(("list", matches)) => run_list(matches, network).await,
        _ => unreachable!("clap requires a known cloud-engine node subcommand"),
    }
}

async fn run_info(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let options: NodeInfoOptions = parsed_options(matches);
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(current_unix_secs()?);
    let request = CloudEngineNodeInfoRequest::new(network, dashboard, options.node_id);
    let report = build_cloud_engine_node_info_report(&request).await?;
    write_text_or_json(format, &report, cloud_engine_node_info_report_text)
}

async fn run_list(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let options: NodeListOptions = parsed_options(matches);
    let (format, dashboard) =
        SourceOutputOptions::from_matches(matches).into_dashboard_request(current_unix_secs()?);
    let request = CloudEngineNodeListRequest::new(network, dashboard);
    let request = match options.node_provider_id {
        Some(provider) => {
            let mut configured_request = request;
            configured_request.node_provider_id = Some(provider);
            configured_request
        }
        None => request,
    };
    let report = build_cloud_engine_node_list_report(&request).await?;
    write_text_or_json(format, &report, cloud_engine_node_list_report_text)
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("node")
        .about("Inspect official Dashboard CloudEngine Type4 node observations")
        .subcommand(info_command())
        .subcommand(list_command())
        .after_help(
            "Examples:\n  icq cloud-engine node list\n  icq cloud-engine node info <node-id>",
        )
}

///
/// NodeInfoOptions
///
/// Typed arguments for the info_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct NodeInfoOptions {
    #[arg(
        id = "node-id",
        value_name = "node-id",
        help = "Exact CloudEngine Type4 node principal"
    )]
    pub(super) node_id: String,
}

fn info_command() -> ClapCommand {
    with_options::<NodeInfoOptions>(
        report_args(
            ClapCommand::new("info")
                .about("Show one exact node required to carry the Type4 reward type"),
            DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
            SOURCE_ENDPOINT_HELP,
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER)),
    )
}

///
/// NodeListOptions
///
/// Typed arguments for the list_command operation.
///

#[derive(clap::Args)]
#[group(skip)]
pub(super) struct NodeListOptions {
    #[arg(
        id = "node-provider-id",
        long,
        value_name = "principal",
        help = "Restrict the remote query to one exact node-provider principal"
    )]
    pub(super) node_provider_id: Option<String>,
}

fn list_command() -> ClapCommand {
    report_args(
        with_options::<NodeListOptions>(
            ClapCommand::new("list")
                .about("List the complete explicitly scoped Dashboard Type4 node resource"),
        ),
        DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT,
        SOURCE_ENDPOINT_HELP,
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

#[cfg(test)]
mod tests;
