//! Module: cloud_engine
//!
//! Responsibility: parse and dispatch public CloudEngine control-plane reports.
//! Does not own: native transport, report construction, or text rendering.
//! Boundary: exposes bounded, uncached, mainnet-only CloudEngine queries at the CLI root.

use crate::cli::{
    clap::{required_string, value_arg},
    common::{
        COLLECTION_MODE_LIVE, CurrentUnixSecsError, SOURCE_ENDPOINT_ARG, collection_help,
        current_unix_secs, json_arg, output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::cloud_engine::{
    CloudEngineHostError, CloudEngineSourceRequest, DEFAULT_CLOUD_ENGINE_SOURCE_ENDPOINT,
    build_cloud_engine_operator_report, build_cloud_engine_prices_report,
    cloud_engine_operator_report_text, cloud_engine_prices_report_text,
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
        Some(("prices", matches)) => run_prices(matches, network),
        _ => unreachable!("clap requires a known cloud-engine subcommand"),
    }
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
    Ok(CloudEngineSourceRequest::from_unix_secs(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
        "ic-query",
    ))
}

pub fn command() -> ClapCommand {
    ClapCommand::new("cloud-engine")
        .bin_name("icq cloud-engine")
        .about("Inspect public CloudEngine control-plane metadata")
        .subcommand(info_command())
        .subcommand(prices_command())
        .after_help("Examples:\n  icq cloud-engine info <subnet-id>\n  icq cloud-engine prices")
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
        assert!(usage.contains("prices"));

        let info = render_help(info_command());
        assert!(info.contains("<subnet-id>"));
        assert!(info.contains("operator queries"));
        assert!(info.contains("certified"));

        let prices = render_help(prices_command());
        assert!(prices.contains("exactly two control-plane queries"));
        assert!(prices.contains("1,000"));
        assert!(prices.contains(COLLECTION_MODE_LIVE));
    }

    #[test]
    fn help_and_version_do_not_make_live_calls() {
        for args in [
            &["cloud-engine", "--help"][..],
            &["cloud-engine", "info", "--help"],
            &["cloud-engine", "prices", "--help"],
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
}
