//! Module: sns::commands::run::lookup
//!
//! Responsibility: run SNS lookup-style commands.
//! Does not own: clap command construction, live source reads, or rendering.
//! Boundary: maps shared lookup options into report requests.

use crate::{
    cli::common::write_text_or_json,
    sns::commands::{
        SnsCommandError, options::SnsLookupOptions, run::common::lookup_command_parts,
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsHostError, SnsLookupRequest, build_sns_info_report, build_sns_params_report,
    build_sns_swap_report, build_sns_token_report, build_sns_upgrade_report, sns_info_report_text,
    sns_params_report_text, sns_swap_report_text, sns_token_report_text, sns_upgrade_report_text,
};
use serde::Serialize;
pub(super) fn run_sns_info(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_info_report,
        sns_info_report_text,
    )
}

pub(super) fn run_sns_token(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_token_report,
        sns_token_report_text,
    )
}

pub(super) fn run_sns_params(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_params_report,
        sns_params_report_text,
    )
}

pub(super) fn run_sns_swap(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_swap_report,
        sns_swap_report_text,
    )
}

pub(super) fn run_sns_upgrade(matches: &ArgMatches, network: &str) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_upgrade_report,
        sns_upgrade_report_text,
    )
}

pub(super) fn run_sns_lookup<Report>(
    matches: &ArgMatches,
    network: &str,
    build_report: fn(&SnsLookupRequest) -> Result<Report, SnsHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SnsCommandError>
where
    Report: Serialize,
{
    let options = SnsLookupOptions::from_matches(matches, network);
    let parts = lookup_command_parts(options)?;
    let format = parts.format;
    let request = SnsLookupRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
    };
    let report = build_report(&request)?;
    write_text_or_json(format, &report, render_text)
}
