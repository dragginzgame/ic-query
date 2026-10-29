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
    SnsHostError, SnsLookupRequest, SnsMetricsRequest, build_sns_metrics_report,
    sns_metrics_report_text,
};
use serde::Serialize;
pub(super) async fn run_sns_metrics(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let options = SnsMetricsOptions::from_matches(matches, network);
    let time_window_seconds = options.time_window_seconds;
    let lookup = options.lookup;
    let now_unix_secs = current_unix_secs()?;
    let (format, lookup) = lookup.into_request(now_unix_secs);
    let request = SnsMetricsRequest::new(lookup).with_time_window_seconds(time_window_seconds);
    let report = build_sns_metrics_report(&request).await?;
    write_text_or_json(format, &report, sns_metrics_report_text)
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
    let (format, request) = options.into_request(now_unix_secs);
    let report = build_report(&request).await?;
    write_text_or_json(format, &report, render_text)
}
