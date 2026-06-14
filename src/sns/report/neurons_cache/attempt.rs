use super::{
    SnsHostError, SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshRequest, sns_cache_file_error,
};
use crate::{
    cache_file::write_text_atomically,
    sns::report::source::{MainnetSns, SnsFetchRequest},
    subnet_catalog::format_utc_timestamp_secs,
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct SnsNeuronsRefreshAttempt {
    schema_version: u32,
    network: String,
    source_endpoint: String,
    started_at: String,
    updated_at: String,
    root_canister_id: String,
    governance_canister_id: String,
    status: String,
    page_size: u32,
    pages_fetched: u32,
    rows_fetched: usize,
    last_cursor: Option<String>,
    last_error: Option<String>,
}

pub(super) struct SnsNeuronsAttemptParts<'a> {
    pub(super) request: &'a SnsNeuronsRefreshRequest,
    pub(super) fetch_request: &'a SnsFetchRequest,
    pub(super) sns: &'a MainnetSns,
    pub(super) status: &'static str,
    pub(super) pages_fetched: u32,
    pub(super) rows_fetched: usize,
    pub(super) last_cursor: Option<String>,
    pub(super) last_error: Option<String>,
}

pub(super) fn write_sns_neurons_attempt(
    path: &Path,
    attempt: &SnsNeuronsRefreshAttempt,
) -> Result<(), SnsHostError> {
    let data =
        serde_json::to_string_pretty(attempt).map_err(|source| SnsHostError::SerializeCache {
            path: path.to_path_buf(),
            source,
        })?;
    write_text_atomically(path, &data).map_err(sns_cache_file_error)
}

pub(super) fn failed_attempt_from_latest_progress(
    attempt_path: &Path,
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    err: &SnsHostError,
) -> SnsNeuronsRefreshAttempt {
    let latest = read_sns_neurons_attempt(attempt_path);
    let pages_fetched = latest.as_ref().map_or(0, |attempt| attempt.pages_fetched);
    let rows_fetched = latest.as_ref().map_or(0, |attempt| attempt.rows_fetched);
    let last_cursor = latest.and_then(|attempt| attempt.last_cursor);
    attempt_from_parts(SnsNeuronsAttemptParts {
        request,
        fetch_request,
        sns,
        status: "failed",
        pages_fetched,
        rows_fetched,
        last_cursor,
        last_error: Some(err.to_string()),
    })
}

pub(super) fn read_sns_neurons_attempt_status(
    path: &Path,
) -> Option<SnsNeuronsRefreshAttemptStatus> {
    read_sns_neurons_attempt(path).map(sns_neurons_attempt_status)
}

pub(super) fn attempt_from_parts(parts: SnsNeuronsAttemptParts<'_>) -> SnsNeuronsRefreshAttempt {
    SnsNeuronsRefreshAttempt {
        schema_version: SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.request.network.clone(),
        source_endpoint: parts.request.source_endpoint.clone(),
        started_at: parts.fetch_request.fetched_at.clone(),
        updated_at: current_timestamp_text(&parts.fetch_request.fetched_at),
        root_canister_id: parts.sns.root_canister_id.clone(),
        governance_canister_id: parts.sns.governance_canister_id.clone(),
        status: parts.status.to_string(),
        page_size: parts.request.page_size,
        pages_fetched: parts.pages_fetched,
        rows_fetched: parts.rows_fetched,
        last_cursor: parts.last_cursor,
        last_error: parts.last_error,
    }
}

fn read_sns_neurons_attempt(path: &Path) -> Option<SnsNeuronsRefreshAttempt> {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
}

fn sns_neurons_attempt_status(attempt: SnsNeuronsRefreshAttempt) -> SnsNeuronsRefreshAttemptStatus {
    SnsNeuronsRefreshAttemptStatus {
        status: attempt.status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    }
}

fn current_timestamp_text(fallback: &str) -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| fallback.to_string(),
        |duration| format_utc_timestamp_secs(duration.as_secs()),
    )
}
