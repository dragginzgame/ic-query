//! Shared SNS snapshot-refresh and local cache-inspection command orchestration.

use crate::{
    cli::common::{NetworkOutputOptions, current_unix_secs, write_text_or_json},
    progress::StderrQueryProgress,
    sns::commands::{
        SnsCommandError,
        options::{SnsCacheStatusOptions, SnsPagedRefreshOptions, cache_list_request},
    },
    storage::cache_root,
};
use clap::ArgMatches;
use ic_query::{
    QueryProgress,
    sns::{
        SnsCacheListReport, SnsCacheListRequest, SnsCacheStatusReport, SnsCacheStatusRequest,
        SnsHostError, SnsSnapshotRefreshReport, SnsSnapshotRefreshRequest,
        sns_snapshot_refresh_report_text,
    },
};

pub(super) async fn run_snapshot_refresh(
    matches: &ArgMatches,
    network: &str,
    refresh: impl AsyncFn(
        &SnsSnapshotRefreshRequest,
        &mut dyn QueryProgress,
    ) -> Result<SnsSnapshotRefreshReport, SnsHostError>,
) -> Result<(), SnsCommandError> {
    let options = SnsPagedRefreshOptions::from_matches(matches, network);
    let (format, request) = options.into_request(cache_root()?, current_unix_secs()?);
    let report = refresh(&request, &mut StderrQueryProgress::new()).await?;
    write_text_or_json(format, &report, sns_snapshot_refresh_report_text)
}

pub(super) fn run_cache_list(
    matches: &ArgMatches,
    network: &str,
    build: fn(&SnsCacheListRequest) -> Result<SnsCacheListReport, SnsHostError>,
    render: fn(&SnsCacheListReport) -> String,
) -> Result<(), SnsCommandError> {
    let options = NetworkOutputOptions::from_matches(matches, network);
    let (format, request) = cache_list_request(options, cache_root()?);
    let report = build(&request)?;
    write_text_or_json(format, &report, render)
}

pub(super) fn run_cache_status(
    matches: &ArgMatches,
    network: &str,
    build: fn(&SnsCacheStatusRequest) -> Result<SnsCacheStatusReport, SnsHostError>,
    render: fn(&SnsCacheStatusReport) -> String,
) -> Result<(), SnsCommandError> {
    let options = SnsCacheStatusOptions::from_matches(matches, network);
    let (format, request) = options.into_request(cache_root()?);
    let report = build(&request)?;
    write_text_or_json(format, &report, render)
}
