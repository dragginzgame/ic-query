//! Module: cloud_engine::provider
//!
//! Responsibility: parse and dispatch official Dashboard CloudEngine provider reports.
//! Does not own: HTTP transport, source validation, report rendering, or caching.
//! Boundary: provider identity means an NNS node-provider principal, not an engine canister.

use super::CloudEngineCommandError;
use crate::cli::{
    clap::{required_string, value_arg},
    common::{
        COLLECTION_MODE_LIVE, SOURCE_ENDPOINT_ARG, collection_help, current_unix_secs, json_arg,
        output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cloud_engine::{
    CloudEngineProviderInfoRequest, CloudEngineProviderListRequest,
    DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT, build_cloud_engine_provider_info_report,
    build_cloud_engine_provider_list_report, cloud_engine_provider_info_report_text,
    cloud_engine_provider_list_report_text,
};

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

pub(super) fn run_matches(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), CloudEngineCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches, network),
        Some(("list", matches)) => run_list(matches, network),
        _ => unreachable!("clap requires a known cloud-engine provider subcommand"),
    }
}

fn run_info(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let request = CloudEngineProviderInfoRequest::new(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
        required_string(matches, "node-provider-id"),
    );
    let report = build_cloud_engine_provider_info_report(&request)?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_provider_info_report_text,
    )
}

fn run_list(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let request = CloudEngineProviderListRequest::new(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
    );
    let report = build_cloud_engine_provider_list_report(&request)?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_provider_list_report_text,
    )
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
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER))
}

fn list_command() -> ClapCommand {
    report_args(
        ClapCommand::new("list")
            .bin_name("icq cloud-engine provider list")
            .about("List providers carrying explicit CloudEngine evidence"),
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, LIST_HELP_AFTER))
}

fn report_args(command: ClapCommand) -> ClapCommand {
    command.arg(json_arg()).arg(
        source_endpoint_arg(DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT)
            .help("Official Dashboard v3 endpoint used for provider metadata"),
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
    fn help_discloses_provider_identity_authority_and_bounds() {
        let namespace = render_help(command());
        assert!(namespace.contains("Usage: icq cloud-engine provider [COMMAND]"));
        assert!(namespace.contains("info"));
        assert!(namespace.contains("list"));

        let info = render_help(info_command());
        assert!(info.contains("<node-provider-id>"));
        assert!(info.contains("not an engine canister principal"));
        assert!(info.contains("one exact official Dashboard request"));
        assert!(info.contains("no CloudEngine evidence"));

        let list = render_help(list_command());
        assert!(list.contains("complete node-provider"));
        assert!(list.contains("1,000"));
        assert!(list.contains("uncertified"));
        assert!(list.contains("--source-endpoint <url>"));
    }

    #[test]
    fn report_options_default_to_dashboard_and_text() {
        for command in [
            list_command(),
            info_command().mut_arg("node-provider-id", |arg| arg.required(false)),
        ] {
            let matches = parse_matches(command, Vec::<OsString>::new())
                .expect("parse default provider options");
            assert_eq!(
                required_string(&matches, SOURCE_ENDPOINT_ARG),
                DEFAULT_CLOUD_ENGINE_DASHBOARD_SOURCE_ENDPOINT
            );
            assert_eq!(output_format(&matches), OutputFormat::Text);
        }
    }
}
