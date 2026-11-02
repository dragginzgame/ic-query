//! Module: sns::commands::run::lookup
//!
//! Responsibility: run SNS lookup-style commands.
//! Does not own: clap command construction, live source reads, or rendering.
//! Boundary: maps shared lookup options into report requests.

use crate::{
    cli::common::{current_unix_secs, write_text_or_json},
    sns::commands::{
        SnsCommandError,
        options::{lookup_request, metrics_request},
    },
};
use clap::ArgMatches;
use ic_query::sns::SnsHostError;
use ic_query::sns::SnsLookupRequest;
use ic_query::sns::build_sns_metrics_report;
use ic_query_text::sns::sns_metrics_report_text;
use serde::Serialize;
pub(super) async fn run_sns_metrics(
    matches: &ArgMatches,
    network: &str,
) -> Result<(), SnsCommandError> {
    let (format, request) = metrics_request(matches, network, current_unix_secs()?)?;
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
    let (format, request) = lookup_request(matches, network, current_unix_secs()?)?;
    let report = build_report(&request).await?;
    write_text_or_json(format, &report, render_text)
}
