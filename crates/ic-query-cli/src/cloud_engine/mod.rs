//! Module: cloud_engine
//!
//! Responsibility: parse and dispatch public CloudEngine metadata reports.
//! Does not own: native transport, report construction, or text rendering.
//! Boundary: exposes mainnet-only CloudEngine inventory and bounded public reports at the CLI root.

mod provider;

use crate::{
    cli::{
        clap::{required_string, value_arg},
        common::{
            COLLECTION_MODE_LIVE, CurrentUnixSecsError, SOURCE_ENDPOINT_ARG, collection_help,
            current_unix_secs, json_arg, output_format, source_endpoint_arg, write_text_or_json,
        },
    },
    progress::announce_missing_mainnet_cache,
    storage::{CacheRootError, cache_root},
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cloud_engine::{
    CloudEngineHostError, CloudEngineSourceRequest, DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT,
    build_cloud_engine_list_report, build_cloud_engine_operator_report,
    build_cloud_engine_prices_report, cloud_engine_list_report_text,
    cloud_engine_operator_report_text, cloud_engine_prices_report_text,
};
use ic_query::ic::IcHostError;
use ic_query::subnet_catalog::{
    DEFAULT_STALE_AFTER_SECONDS, DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT, SubnetCatalogCacheRequest,
    SubnetCatalogListRequest, subnet_catalog_path,
};
use std::io;
use thiserror::Error as ThisError;

const INFO_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine info 2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe
  icq cloud-engine info 2nl67-oqoc5-cmocj-otlhq-kr2kr-53hov-drrds-7ihcs-fhomv-2eyvu-6qe --json

This command makes one control-plane query to resolve the Subnet and four
public operator queries when an operator is registered. The responses are not
certified and the sequential calls are not an exact point-in-time snapshot.";
const LIST_COLLECTION_MODE: &str = "Cache-backed Registry inventory; refreshes missing or recoverably invalid catalog content, then makes one live operator-binding query per returned Subnet.";
const LIST_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine list
  icq cloud-engine list --json

The Registry supplies the complete CloudEngine Subnet inventory. The command
then makes one exact public control-plane operator-binding query per returned
Subnet, up to 100. Registry and control-plane provenance remain separate, and
one failed binding lookup is reported on its row without discarding the
Registry snapshot.";
const PRICES_HELP_AFTER: &str = "\
Examples:
  icq cloud-engine prices
  icq cloud-engine prices --json

This command makes exactly two control-plane queries: one for the network fee
and one for at most 1,000 public marketplace rows. The responses are not
certified or presented as an exact point-in-time snapshot.";

///
/// CloudEngineCommandError
///
/// Errors surfaced while parsing or running a CloudEngine command.
///

#[derive(Debug, ThisError)]
pub enum CloudEngineCommandError {
    /// Native CloudEngine collection or evidence validation failed.
    #[error(transparent)]
    Host(#[from] CloudEngineHostError),
    /// Official Dashboard collection or evidence validation failed.
    #[error(transparent)]
    Dashboard(#[from] IcHostError),
    /// The CLI cache root could not be resolved.
    #[error(transparent)]
    CacheRoot(#[from] CacheRootError),
    /// The process clock could not supply a Unix collection timestamp.
    #[error(transparent)]
    Clock(#[from] CurrentUnixSecsError),
    /// Writing the selected report output failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub fn run_matches(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    match matches.subcommand() {
        Some(("info", matches)) => run_info(matches, network),
        Some(("list", matches)) => run_list(matches, network),
        Some(("prices", matches)) => run_prices(matches, network),
        Some(("provider", matches)) => provider::run_matches(matches, network),
        _ => unreachable!("clap requires a known cloud-engine subcommand"),
    }
}

fn run_list(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let now_unix_secs = current_unix_secs()?;
    let registry_source_endpoint = required_string(matches, REGISTRY_SOURCE_ENDPOINT_ARG);
    let cache = SubnetCatalogCacheRequest::new(cache_root()?, network);
    announce_missing_mainnet_cache(
        network,
        "subnet catalog",
        &subnet_catalog_path(&cache.cache_root, &cache.network),
        &registry_source_endpoint,
    );
    let catalog_request = SubnetCatalogListRequest::new(
        cache,
        registry_source_endpoint,
        now_unix_secs,
        DEFAULT_STALE_AFTER_SECONDS,
    );
    let control_plane_request = source_request_at(matches, network, now_unix_secs);
    let report = build_cloud_engine_list_report(&catalog_request, &control_plane_request)?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_list_report_text,
    )
}

fn run_info(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let request = source_request(matches, network)?;
    let report =
        build_cloud_engine_operator_report(&request, &required_string(matches, "subnet-id"))?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_operator_report_text,
    )
}

fn run_prices(matches: &ArgMatches, network: &str) -> Result<(), CloudEngineCommandError> {
    let request = source_request(matches, network)?;
    let report = build_cloud_engine_prices_report(&request)?;
    write_text_or_json(
        output_format(matches),
        &report,
        cloud_engine_prices_report_text,
    )
}

fn source_request(
    matches: &ArgMatches,
    network: &str,
) -> Result<CloudEngineSourceRequest, CurrentUnixSecsError> {
    Ok(source_request_at(matches, network, current_unix_secs()?))
}

fn source_request_at(
    matches: &ArgMatches,
    network: &str,
    fetched_at_unix_secs: u64,
) -> CloudEngineSourceRequest {
    CloudEngineSourceRequest::from_unix_secs(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        fetched_at_unix_secs,
        "ic-query",
    )
}

pub fn command() -> ClapCommand {
    ClapCommand::new("cloud-engine")
        .bin_name("icq cloud-engine")
        .about("Inspect public CloudEngine metadata")
        .subcommand(info_command())
        .subcommand(list_command())
        .subcommand(prices_command())
        .subcommand(provider::command())
        .after_help(
            "Examples:\n  icq cloud-engine list\n  icq cloud-engine info <subnet-id>\n  icq cloud-engine prices\n  icq cloud-engine provider list",
        )
}

fn list_command() -> ClapCommand {
    report_args(
        ClapCommand::new("list")
            .bin_name("icq cloud-engine list")
            .about("List Registry CloudEngine Subnets and public operator bindings")
            .arg(
                value_arg(REGISTRY_SOURCE_ENDPOINT_ARG)
                    .long(REGISTRY_SOURCE_ENDPOINT_ARG)
                    .value_name("url")
                    .default_value(DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT)
                    .help("IC API endpoint used to refresh the Registry Subnet Catalog"),
            ),
    )
    .after_help(collection_help(LIST_COLLECTION_MODE, LIST_HELP_AFTER))
}

fn info_command() -> ClapCommand {
    report_args(
        ClapCommand::new("info")
            .bin_name("icq cloud-engine info")
            .about("Show the operator binding and public settings for one CloudEngine Subnet")
            .arg(
                value_arg("subnet-id")
                    .required(true)
                    .value_name("subnet-id")
                    .help("CloudEngine Subnet principal to resolve"),
            ),
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, INFO_HELP_AFTER))
}

fn prices_command() -> ClapCommand {
    report_args(
        ClapCommand::new("prices")
            .bin_name("icq cloud-engine prices")
            .about("Show the public CloudEngine network fee and marketplace prices"),
    )
    .after_help(collection_help(COLLECTION_MODE_LIVE, PRICES_HELP_AFTER))
}

fn report_args(command: ClapCommand) -> ClapCommand {
    command.arg(json_arg()).arg(
        source_endpoint_arg(DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT)
            .help("IC API endpoint used for native CloudEngine queries"),
    )
}

const REGISTRY_SOURCE_ENDPOINT_ARG: &str = "registry-source-endpoint";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{
        clap::{parse_matches, render_help},
        common::OutputFormat,
    };
    use std::ffi::OsString;

    #[test]
    fn usage_discloses_bounded_uncertified_reports() {
        let usage = render_help(command());
        assert!(usage.contains("Usage: icq cloud-engine [COMMAND]"));
        assert!(usage.contains("info"));
        assert!(usage.contains("list"));
        assert!(usage.contains("prices"));
        assert!(usage.contains("provider"));

        let info = render_help(info_command());
        assert!(info.contains("<subnet-id>"));
        assert!(info.contains("operator queries"));
        assert!(info.contains("certified"));

        let prices = render_help(prices_command());
        assert!(prices.contains("exactly two control-plane queries"));
        assert!(prices.contains("1,000"));
        assert!(prices.contains(COLLECTION_MODE_LIVE));

        let list = render_help(list_command());
        assert!(list.contains("one exact public control-plane"));
        assert!(list.contains("--registry-source-endpoint"));
        assert!(list.contains("up to 100"));
        assert!(list.contains(LIST_COLLECTION_MODE));
    }

    #[test]
    fn help_and_version_do_not_make_live_calls() {
        for args in [
            &["cloud-engine", "--help"][..],
            &["cloud-engine", "info", "--help"],
            &["cloud-engine", "list", "--help"],
            &["cloud-engine", "prices", "--help"],
            &["cloud-engine", "provider", "--help"],
            &["cloud-engine", "provider", "info", "--help"],
            &["cloud-engine", "provider", "list", "--help"],
            &["cloud-engine", "--version"],
        ] {
            assert!(crate::run(args.iter().map(OsString::from)).is_ok());
        }
    }

    #[test]
    fn report_options_default_to_native_endpoint_and_text() {
        let matches = parse_matches(prices_command(), Vec::<OsString>::new())
            .expect("parse default CloudEngine options");

        assert_eq!(
            required_string(&matches, SOURCE_ENDPOINT_ARG),
            DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT
        );
        assert_eq!(output_format(&matches), OutputFormat::Text);
    }

    #[test]
    fn list_options_keep_registry_and_control_plane_endpoints_explicit() {
        let matches = parse_matches(list_command(), Vec::<OsString>::new())
            .expect("parse default CloudEngine list options");

        assert_eq!(
            required_string(&matches, REGISTRY_SOURCE_ENDPOINT_ARG),
            DEFAULT_SUBNET_CATALOG_SOURCE_ENDPOINT
        );
        assert_eq!(
            required_string(&matches, SOURCE_ENDPOINT_ARG),
            DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT
        );
        assert_eq!(output_format(&matches), OutputFormat::Text);
    }
}
