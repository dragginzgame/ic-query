//! System-canister command-line parsing and dispatch.

use crate::{
    cli::{
        clap::{
            parse_matches_or_usage, parse_required_subcommand_or_usage, passthrough_subcommand,
            render_help, required_string, required_typed,
        },
        common::{
            COLLECTION_MODE_LIVE, CurrentUnixSecsError, FORMAT_ARG, OutputFormat,
            SOURCE_ENDPOINT_ARG, collection_help, current_unix_secs, format_arg,
            source_endpoint_arg, write_text_or_json,
        },
        globals::internal_network_arg,
        help::collect_args_or_print_help_or_version,
    },
    version_text,
};
use clap::Command as ClapCommand;
use ic_query::{
    subnet_catalog::MAINNET_NETWORK,
    system::cmc::{
        CmcHostError, CmcSourceRequest, DEFAULT_CMC_SOURCE_ENDPOINT, build_cmc_cycles_report,
        build_cmc_xdr_report, cmc_cycles_report_text, cmc_xdr_report_text,
    },
};
use serde::Serialize;
use std::{ffi::OsString, io};
use thiserror::Error as ThisError;

const NETWORK_ARG: &str = "network";
const SYSTEM_HELP_AFTER: &str = "\
Examples:
  icq system xdr
  icq system cycles";
const XDR_HELP_AFTER: &str = "\
Examples:
  icq system xdr
  icq system xdr --format json";
const CYCLES_HELP_AFTER: &str = "\
Examples:
  icq system cycles
  icq system cycles --format json";

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

pub fn run<I>(args: I) -> Result<(), SystemCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, system_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(system_command(), args, system_usage)
        .map_err(SystemCommandError::Usage)?;
    match command.as_str() {
        "xdr" => run_report(
            args,
            report_command(
                "xdr",
                "Show the certified CMC ICP/XDR conversion rate",
                XDR_HELP_AFTER,
            ),
            xdr_usage,
            build_cmc_xdr_report,
            cmc_xdr_report_text,
        ),
        "cycles" => run_report(
            args,
            report_command(
                "cycles",
                "Show cycles conversions derived from the certified CMC rate",
                CYCLES_HELP_AFTER,
            ),
            cycles_usage,
            build_cmc_cycles_report,
            cmc_cycles_report_text,
        ),
        _ => unreachable!("system dispatch only defines known commands"),
    }
}

fn run_report<I, Report>(
    args: I,
    command: ClapCommand,
    usage: fn() -> String,
    build: fn(&CmcSourceRequest) -> Result<Report, CmcHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SystemCommandError>
where
    I: IntoIterator<Item = OsString>,
    Report: Serialize,
{
    let Some(args) = command_args(args, usage) else {
        return Ok(());
    };
    let matches =
        parse_matches_or_usage(command, args, usage).map_err(SystemCommandError::Usage)?;
    let request = CmcSourceRequest::from_unix_secs(
        required_string(&matches, NETWORK_ARG),
        required_string(&matches, SOURCE_ENDPOINT_ARG),
        current_unix_secs()?,
        "ic-query",
    );
    let format = required_typed::<OutputFormat>(&matches, FORMAT_ARG);
    let report = build(&request)?;
    write_text_or_json(format, &report, render_text)
}

fn command_args<I>(args: I, usage: impl FnOnce() -> String) -> Option<Vec<OsString>>
where
    I: IntoIterator<Item = OsString>,
{
    collect_args_or_print_help_or_version(args, usage, version_text())
}

fn system_command() -> ClapCommand {
    ClapCommand::new("system")
        .bin_name("icq system")
        .about("Inspect native IC system-canister metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("xdr").about("Show the certified CMC ICP/XDR conversion rate"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("cycles").about(
            "Show cycles conversions derived from the certified CMC rate",
        )))
        .after_help(SYSTEM_HELP_AFTER)
}

fn report_command(name: &'static str, about: &'static str, examples: &'static str) -> ClapCommand {
    ClapCommand::new(name)
        .bin_name(format!("icq system {name}"))
        .about(about)
        .disable_help_flag(true)
        .arg(format_arg())
        .arg(
            source_endpoint_arg(DEFAULT_CMC_SOURCE_ENDPOINT)
                .help("IC API endpoint used for the native CMC query"),
        )
        .arg(internal_network_arg().default_value(MAINNET_NETWORK))
        .after_help(collection_help(COLLECTION_MODE_LIVE, examples))
}

fn system_usage() -> String {
    render_help(system_command())
}

fn xdr_usage() -> String {
    render_help(report_command(
        "xdr",
        "Show the certified CMC ICP/XDR conversion rate",
        XDR_HELP_AFTER,
    ))
}

fn cycles_usage() -> String {
    render_help(report_command(
        "cycles",
        "Show cycles conversions derived from the certified CMC rate",
        CYCLES_HELP_AFTER,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_describes_bounded_native_reports() {
        let text = system_usage();
        assert!(text.contains("Usage: icq system [COMMAND]"));
        assert!(text.contains("xdr"));
        assert!(text.contains("cycles"));

        for text in [xdr_usage(), cycles_usage()] {
            assert!(text.contains("--source-endpoint <url>"));
            assert!(text.contains("--format <text|json>"));
            assert!(text.contains(COLLECTION_MODE_LIVE));
        }
    }

    #[test]
    fn help_and_version_do_not_make_live_calls() {
        for args in [
            &["help"][..],
            &["xdr", "help"],
            &["cycles", "help"],
            &["--version"],
        ] {
            assert!(run(args.iter().map(OsString::from)).is_ok());
        }
    }

    #[test]
    fn report_options_default_to_mainnet_and_native_endpoint() {
        let matches = parse_matches_or_usage(
            report_command("xdr", "test", XDR_HELP_AFTER),
            Vec::<OsString>::new(),
            xdr_usage,
        )
        .expect("parse default CMC options");

        assert_eq!(required_string(&matches, NETWORK_ARG), MAINNET_NETWORK);
        assert_eq!(
            required_string(&matches, SOURCE_ENDPOINT_ARG),
            DEFAULT_CMC_SOURCE_ENDPOINT
        );
        assert_eq!(
            required_typed::<OutputFormat>(&matches, FORMAT_ARG),
            OutputFormat::Text
        );
    }
}
