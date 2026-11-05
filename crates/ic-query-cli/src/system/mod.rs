//! System-canister command-line parsing and dispatch.

use crate::cli::{
    clap::required_string,
    common::{
        COLLECTION_MODE_LIVE, CurrentUnixSecsError, SOURCE_ENDPOINT_ARG, collection_help,
        current_unix_secs, json_arg, output_format, source_endpoint_arg, write_text_or_json,
    },
};
use clap::{ArgMatches, Command as ClapCommand};
use ic_query::report::CollectionSourceRequest;
use ic_query::system::cmc::CmcHostError;
use ic_query::system::cmc::DEFAULT_CMC_SOURCE_ENDPOINT;
use ic_query::system::cmc::build_cmc_cycles_report;
use ic_query::system::cmc::build_cmc_xdr_report;
use ic_query_text::system::cmc::cmc_cycles_report_text;
use ic_query_text::system::cmc::cmc_xdr_report_text;
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

pub async fn run_matches(matches: &ArgMatches, network: &str) -> Result<(), SystemCommandError> {
    match matches.subcommand() {
        Some(("xdr", matches)) => {
            run_report(matches, network, build_cmc_xdr_report, cmc_xdr_report_text).await
        }
        Some(("cycles", matches)) => {
            run_report(
                matches,
                network,
                build_cmc_cycles_report,
                cmc_cycles_report_text,
            )
            .await
        }
        _ => unreachable!("clap requires a known system subcommand"),
    }
}

async fn run_report<Report>(
    matches: &ArgMatches,
    network: &str,
    build: impl AsyncFn(&CollectionSourceRequest) -> Result<Report, CmcHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SystemCommandError>
where
    Report: Serialize,
{
    let request = CollectionSourceRequest::from_unix_secs(
        network,
        required_string(matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
        "ic-query",
    );
    let format = output_format(matches);
    let report = build(&request).await?;
    write_text_or_json(format, &report, render_text)
}

pub fn command() -> ClapCommand {
    ClapCommand::new("system")
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
        .about(about)
        .arg(json_arg())
        .arg(
            source_endpoint_arg(DEFAULT_CMC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native CMC query"),
        )
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
}

#[cfg(test)]
mod tests;
