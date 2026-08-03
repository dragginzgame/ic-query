//! System-canister command-line parsing and dispatch.

use crate::cli::{
    clap::required_string,
    common::{
        COLLECTION_MODE_LIVE, CurrentUnixSecsError, SOURCE_ENDPOINT_ARG, collection_help,
        current_unix_secs, json_arg, output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::system::cmc::{
    CmcHostError, CmcSourceRequest, DEFAULT_CMC_SOURCE_ENDPOINT, build_cmc_cycles_report,
    build_cmc_xdr_report, cmc_cycles_report_text, cmc_xdr_report_text,
};
use serde::Serialize;
use std::io;
use thiserror::Error as ThisError;

const SYSTEM_HELP_AFTER: &str = "\
Examples:
  icq system xdr
  icq system cycles";
const XDR_HELP_AFTER: &str = "\
Examples:
  icq system xdr
  icq system xdr --json";
const CYCLES_HELP_AFTER: &str = "\
Examples:
  icq system cycles
  icq system cycles --json";

///
/// SystemCommandError
///
/// Errors surfaced while parsing or running a system-canister command.
///

#[derive(Debug, ThisError)]
pub enum SystemCommandError {
    /// Command syntax or option validation failed.
    #[error("{0}")]
    Usage(String),
    /// Native CMC collection or evidence validation failed.
    #[error(transparent)]
    CmcHost(#[from] CmcHostError),
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

pub fn run_matches(matches: &ArgMatches, network: &str) -> Result<(), SystemCommandError> {
    match matches.subcommand() {
        Some(("xdr", matches)) => {
            run_report(matches, network, build_cmc_xdr_report, cmc_xdr_report_text)
        }
        Some(("cycles", matches)) => run_report(
            matches,
            network,
            build_cmc_cycles_report,
            cmc_cycles_report_text,
        ),
        _ => unreachable!("clap requires a known system subcommand"),
    }
}

fn run_report<Report>(
    matches: &ArgMatches,
    network: &str,
    build: fn(&CmcSourceRequest) -> Result<Report, CmcHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SystemCommandError>
where
    Report: Serialize,
{
    let request = CmcSourceRequest::from_unix_secs(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
        "ic-query",
    );
    let format = output_format(matches);
    let report = build(&request)?;
    write_text_or_json(format, &report, render_text)
}

pub fn command() -> ClapCommand {
    ClapCommand::new("system")
        .bin_name("icq system")
        .about("Inspect native IC system-canister metadata")
        .subcommand(report_command(
            "xdr",
            "Show the certified CMC ICP/XDR conversion rate",
            XDR_HELP_AFTER,
        ))
        .subcommand(report_command(
            "cycles",
            "Show cycles conversions derived from the certified CMC rate",
            CYCLES_HELP_AFTER,
        ))
        .after_help(SYSTEM_HELP_AFTER)
}

fn report_command(name: &'static str, about: &'static str, examples: &'static str) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(format!("icq system {name}"))
        .about(about)
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_CMC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native CMC query"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::clap::{parse_matches, render_help};
    use std::ffi::OsString;

    #[test]
    fn usage_describes_bounded_native_reports() {
        let text = render_help(command());
        assert!(text.contains("Usage: icq system [COMMAND]"));
        assert!(text.contains("xdr"));
        assert!(text.contains("cycles"));

        for text in [
            render_help(report_command(
                "xdr",
                "Show the certified CMC ICP/XDR conversion rate",
                XDR_HELP_AFTER,
            )),
            render_help(report_command(
                "cycles",
                "Show cycles conversions derived from the certified CMC rate",
                CYCLES_HELP_AFTER,
            )),
        ] {
            assert!(text.contains("--source-endpoint <url>"));
            assert!(text.contains("--json"));
            assert!(text.contains(COLLECTION_MODE_LIVE));
        }
    }

    #[test]
    fn help_and_version_do_not_make_live_calls() {
        for args in [
            &["system", "--help"][..],
            &["system", "xdr", "--help"],
            &["system", "cycles", "--help"],
            &["system", "--version"],
        ] {
            assert!(crate::run(args.iter().map(OsString::from)).is_ok());
        }
    }

    #[test]
    fn report_options_default_to_mainnet_and_native_endpoint() {
        let matches = parse_matches(
            report_command("xdr", "test", XDR_HELP_AFTER),
            Vec::<OsString>::new(),
        )
        .expect("parse default CMC options");

        assert_eq!(
            required_string(&matches, SOURCE_ENDPOINT_ARG),
            DEFAULT_CMC_SOURCE_ENDPOINT
        );
        assert_eq!(
            output_format(&matches),
            crate::cli::common::OutputFormat::Text
        );
    }
}
