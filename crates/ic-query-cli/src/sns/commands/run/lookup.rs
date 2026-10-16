//! Module: sns::commands::run::lookup
//!
//! Responsibility: run SNS lookup-style commands.
//! Does not own: clap command construction, live source reads, or rendering.
//! Boundary: maps shared lookup options into report requests.

use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    sns::commands::{
        SnsCommandError,
        options::{SnsLookupOptions, SnsMetricsOptions},
    },
};
use clap::ArgMatches;
use ic_query::sns::{
    SnsHostError, SnsLookupRequest, SnsMetricsRequest, build_sns_info_report,
    build_sns_metrics_report, build_sns_params_report, build_sns_swap_report,
    build_sns_token_report, build_sns_upgrade_report, sns_info_report_text,
    sns_metrics_report_text, sns_params_report_text, sns_swap_report_text, sns_token_report_text,
    sns_upgrade_report_text,
};
use serde::Serialize;
pub(super) async fn run_sns_info(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_info_report,
        sns_info_report_text,
    )
    .await
}

pub(super) async fn run_sns_token(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_token_report,
        sns_token_report_text,
    )
    .await
}

pub(super) async fn run_sns_metrics(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsMetricsOptions::from_matches(matches, network);
    let time_window_seconds = options.time_window_seconds;
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let format = lookup.format;
    let request = SnsMetricsRequest::new(ic_query::sns::SnsLookupRequest::new(
        lookup.network,
        lookup.source_endpoint,
        now_unix_secs,
        lookup.input,
    ))
    .with_time_window_seconds(time_window_seconds);
    let report = build_sns_metrics_report(&request).await?;
    write_text_or_json(format, &report, sns_metrics_report_text)
}

pub(super) async fn run_sns_parameters(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_params_report,
        sns_params_report_text,
    )
    .await
}

pub(super) async fn run_sns_swap(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_swap_report,
        sns_swap_report_text,
    )
    .await
}

pub(super) async fn run_sns_upgrade(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    run_sns_lookup(
        matches,
        network,
        build_sns_upgrade_report,
        sns_upgrade_report_text,
    )
    .await
}

pub(super) async fn run_sns_lookup<Report>(
    matches: &ArgMatches,
    network: &str,
    build_report: impl AsyncFn(&SnsLookupRequest) -> Result<Report, SnsHostError>,
    render_text: fn(&Report) -> String,
) -> Result<(), SnsCommandError>
where
    Report: Serialize,
{
    let options = SnsLookupOptions::from_matches(matches, network);
    let now_unix_secs = current_unix_secs()?;
    let format = options.format;
    let request = SnsLookupRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs,
        input: options.input,
    };
    let report = build_report(&request).await?;
    write_text_or_json(format, &report, render_text)
}
