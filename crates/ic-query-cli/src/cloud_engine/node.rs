//! Module: cloud_engine::node
//!
//! Responsibility: parse and dispatch official Dashboard CloudEngine Type4 node reports.
//! Does not own: HTTP transport, source validation, report rendering, or caching.
//! Boundary: requests the explicit Type4 and complete current status scope.

use super::CloudEngineCommandError;
use crate::cli::{
    clap::{required_string, string_option, value_arg},
    common::{
        COLLECTION_MODE_LIVE, SOURCE_ENDPOINT_ARG, collection_help, current_unix_secs, json_arg,
        output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cloud_engine::{
    CloudEngineNodeInfoRequest, CloudEngineNodeListRequest,
    DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT, build_cloud_engine_node_info_report,
    build_cloud_engine_node_list_report, cloud_engine_node_info_report_text,
    cloud_engine_node_list_report_text,
};

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

pub(super) fn run_matches(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), CloudEngineCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches, network),
        Some(("list", matches)) => run_list(matches, network),
        _ => unreachable!("clap requires a known cloud-engine node subcommand"),
    }
}

fn run_info(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let request = CloudEngineNodeInfoRequest::new(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
        required_string(matches, "node-id"),
    );
    let report = build_cloud_engine_node_info_report(&request)?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_node_info_report_text,
    )
}

fn run_list(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let request = CloudEngineNodeListRequest::new(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
    );
    let request = match string_option(matches, "node-provider-id") {
        Some(provider) => request.with_node_provider_id(provider),
        None => request,
    };
    let report = build_cloud_engine_node_list_report(&request)?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_node_list_report_text,
    )
}

pub(super) fn command() -> ClapCommand {
    ClapCommand::new("node")
        .bin_name("icq cloud-engine node")
        .about("Inspect official Dashboard CloudEngine Type4 node observations")
        .subcommand(info_command())
        .subcommand(list_command())
        .after_help(
            "Examples:\n  icq cloud-engine node list\n  icq cloud-engine node info <node-id>",
        )
}

fn info_command() -> ClapCommand {
    report_args(
        ClapCommand::new("info")
            .bin_name("icq cloud-engine node info")
            .about("Show one exact node required to carry the Type4 reward type")
            .arg(
                value_arg("node-id")
                    .required(true)
                    .value_name("node-id")
                    .help("Exact CloudEngine Type4 node principal"),
            ),
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER))
}

fn list_command() -> ClapCommand {
    report_args(
        ClapCommand::new("list")
            .bin_name("icq cloud-engine node list")
            .about("List the complete explicitly scoped Dashboard Type4 node resource")
            .arg(
                value_arg("node-provider-id")
                    .long("node-provider-id")
                    .value_name("principal")
                    .help("Restrict the remote query to one exact node-provider principal"),
            ),
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

fn report_args(command: ClapCommand) -> ClapCommand {
    command.arg(json_arg()).arg(
        source_endpoint_arg(DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official Dashboard v3 endpoint used for Type4 node observations"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{
        clap::{parse_matches, render_help},
        common::OutputFormat,
    };
    use std::ffi::OsString;

    #[test]
    fn help_discloses_type4_scope_authority_and_bounds() {
        let namespace = render_help(command());
        assert!(namespace.contains("Usage: icq cloud-engine node [COMMAND]"));
        assert!(namespace.contains("info"));
        assert!(namespace.contains("list"));

        let info = render_help(info_command());
        assert!(info.contains("<node-id>"));
        assert!(info.contains("requires the"));
        assert!(info.contains("returned reward type to be Type4"));
        assert!(info.contains("uncertified"));

        let list = render_help(list_command());
        assert!(list.contains("explicit Type4"));
        assert!(list.contains("all four currently documented statuses"));
        assert!(list.contains("10,000"));
        assert!(list.contains("--node-provider-id <principal>"));
    }

    #[test]
    fn list_options_preserve_provider_endpoint_and_format() {
        let matches = parse_matches(
            list_command(),
            [
                "--node-provider-id",
                "bvcsg-3od6r-jnydw-eysln-aql7w-td5zn-ay5m6-sibd2-jzojt-anwag-mqe",
                "--json",
            ]
            .into_iter()
            .map(OsString::from),
        )
        .expect("parse Type4 node list options");

        assert_eq!(
            required_string(&matches, SOURCE_ENDPOINT_ARG),
            DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT
        );
        assert!(string_option(&matches, "node-provider-id").is_some());
        assert_eq!(output_format(&matches), OutputFormat::Json);
    }
}
