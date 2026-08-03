//! Module: sns::report::cache_attempt
//!
//! Responsibility: shared SNS refresh context and attempt-sidecar lifecycle.
//! Does not own: family page fetching, cache publication, or text rendering.
//! Boundary: one resolved context and attempt contract serves neuron and proposal refreshes.

use crate::{
    cache::CacheRefreshAttemptStatus,
    snapshot_cache::{
        PagedCollectionPage, SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
        SnapshotRefreshAttemptReadError, SnapshotRefreshProgress, current_attempt_timestamp,
        read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
        write_snapshot_refresh_attempt,
    },
    sns::report::{
        SnsHostError, SnsNeuronsRefreshRequest, SnsProposalsRefreshRequest,
        SnsRefreshAttemptStatus,
        source::{MainnetSns, SnsSourceRequest},
    },
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::Path;

pub(in crate::sns::report) const SNS_REFRESH_ATTEMPT_METADATA_FIELDS: &[&str] =
    &["id", "root_canister_id", "governance_canister_id"];

///
/// SnsRefreshAttemptMetadata
///
/// Snapshot refresh-attempt metadata shared by SNS neuron and proposal cache
/// refresh sidecars.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report) struct SnsRefreshAttemptMetadata {
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

pub(in crate::sns::report) type SnsRefreshAttempt =
    SnapshotRefreshAttempt<SnsRefreshAttemptMetadata>;

///
/// SnsRefreshRequestView
///
/// Request fields shared by SNS neuron and proposal refresh orchestration.
///

pub(in crate::sns::report) trait SnsRefreshRequestView {
    fn network(&self) -> &str;
    fn source_endpoint(&self) -> &str;
    fn now_unix_secs(&self) -> u64;
    fn input(&self) -> &str;
    fn cache_root(&self) -> &Path;
    fn page_size(&self) -> u32;
    fn max_pages(&self) -> Option<u32>;
}

macro_rules! impl_sns_refresh_request_view {
    ($request:ty) => {
        impl SnsRefreshRequestView for $request {
            fn network(&self) -> &str {
                &self.network
            }

            fn source_endpoint(&self) -> &str {
                &self.source_endpoint
            }

            fn now_unix_secs(&self) -> u64 {
                self.now_unix_secs
            }

            fn input(&self) -> &str {
                &self.input
            }

            fn cache_root(&self) -> &Path {
                &self.cache_root
            }

            fn page_size(&self) -> u32 {
                self.page_size
            }

            fn max_pages(&self) -> Option<u32> {
                self.max_pages
            }
        }
    };
}

impl_sns_refresh_request_view!(SnsNeuronsRefreshRequest);
impl_sns_refresh_request_view!(SnsProposalsRefreshRequest);

///
/// SnsRefreshContext
///
/// Resolved inputs and paging policy shared by one SNS collection refresh.
///

#[derive(Clone, Copy)]
pub(in crate::sns::report) struct SnsRefreshContext<'a> {
    pub(in crate::sns::report) path: &'a Path,
    pub(in crate::sns::report) request: &'a dyn SnsRefreshRequestView,
    pub(in crate::sns::report) fetch_request: &'a SnsSourceRequest,
    pub(in crate::sns::report) sns: &'a MainnetSns,
}

impl SnsRefreshContext<'_> {
    pub(in crate::sns::report) fn progress_text(
        self,
        collection: &str,
        page_count: u32,
        row_count: usize,
    ) -> String {
        format!(
            "refreshing SNS {collection} for {}: pages={page_count} rows={row_count}",
            self.sns.name
        )
    }

    pub(in crate::sns::report) fn max_pages_reached(self, page_count: u32) -> bool {
        self.request
            .max_pages()
            .is_some_and(|max_pages| page_count >= max_pages)
    }

    pub(in crate::sns::report) fn incomplete_refresh_error(
        page_count: u32,
        row_count: usize,
        reason: &'static str,
    ) -> SnsHostError {
        SnsHostError::IncompleteRefresh {
            pages_fetched: page_count,
            rows_fetched: row_count,
            reason: reason.to_string(),
        }
    }

    pub(in crate::sns::report) fn page_exhausts_collection(
        self,
        page: &PagedCollectionPage,
        has_next_cursor: bool,
    ) -> bool {
        page.exhausts_collection(self.request.page_size(), has_next_cursor)
    }
}

struct SnsRefreshAttemptParts<'a> {
    context: SnsRefreshContext<'a>,
    status: CacheRefreshAttemptStatus,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
}

fn attempt_from_parts(parts: SnsRefreshAttemptParts<'_>) -> SnsRefreshAttempt {
    SnsRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.context.request.network().to_string(),
        source_endpoint: parts.context.request.source_endpoint().to_string(),
        started_at: parts.context.fetch_request.fetched_at.clone(),
        updated_at: current_attempt_timestamp(&parts.context.fetch_request.fetched_at),
        metadata: SnsRefreshAttemptMetadata {
            id: parts.context.sns.id,
            root_canister_id: parts.context.sns.root_canister_id.clone(),
            governance_canister_id: parts.context.sns.governance_canister_id.clone(),
        },
        status: parts.status.to_string(),
        page_size: parts.context.request.page_size(),
        pages_fetched: parts.progress.pages_fetched,
        rows_fetched: parts.progress.rows_fetched,
        last_cursor: parts.progress.last_cursor,
        last_error: parts.last_error,
    }
}

pub(in crate::sns::report) fn write_starting_sns_refresh_attempt(
    context: SnsRefreshContext<'_>,
) -> Result<(), SnsHostError> {
    write_sns_refresh_attempt_status(
        context,
        CacheRefreshAttemptStatus::Running,
        SnapshotRefreshProgress::default(),
        None,
    )
}

pub(in crate::sns::report) fn write_running_sns_refresh_attempt(
    context: SnsRefreshContext<'_>,
    progress: SnapshotRefreshProgress,
) -> Result<(), SnsHostError> {
    write_sns_refresh_attempt_status(context, CacheRefreshAttemptStatus::Running, progress, None)
}

/// Write the running attempt evidence produced by one retained SNS page.
pub(in crate::sns::report) fn write_running_sns_refresh_page(
    context: SnsRefreshContext<'_>,
    page_count: u32,
    row_count: usize,
    page: &PagedCollectionPage,
) -> Result<(), SnsHostError> {
    write_running_sns_refresh_attempt(
        context,
        SnapshotRefreshProgress::new(page_count, row_count, page.last_cursor_text.clone()),
    )
}

pub(in crate::sns::report) fn write_complete_sns_refresh_attempt(
    context: SnsRefreshContext<'_>,
    progress: SnapshotRefreshProgress,
) -> Result<(), SnsHostError> {
    write_sns_refresh_attempt_status(context, CacheRefreshAttemptStatus::Complete, progress, None)
}

pub(in crate::sns::report) fn write_failed_sns_refresh_attempt(
    context: SnsRefreshContext<'_>,
    error: &SnsHostError,
) {
    let latest = read_sns_refresh_attempt(context.path, context.request.network());
    let progress = SnapshotRefreshProgress::new(
        latest
            .as_ref()
            .map_or(0, |(attempt, _status)| attempt.pages_fetched),
        latest
            .as_ref()
            .map_or(0, |(attempt, _status)| attempt.rows_fetched),
        latest.and_then(|(attempt, _status)| attempt.last_cursor),
    );
    let _ = write_sns_refresh_attempt_status(
        context,
        CacheRefreshAttemptStatus::Failed,
        progress,
        Some(error.to_string()),
    );
}

fn write_sns_refresh_attempt_status(
    context: SnsRefreshContext<'_>,
    status: CacheRefreshAttemptStatus,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
) -> Result<(), SnsHostError> {
    let attempt = attempt_from_parts(SnsRefreshAttemptParts {
        context,
        status,
        progress,
        last_error,
    });
    write_snapshot_refresh_attempt(
        context.path,
        &attempt,
        |path, source| SnsHostError::SerializeCache { path, source },
        SnsHostError::Cache,
    )
}

pub(in crate::sns::report) fn validate_sns_refresh_attempt(
    path: &Path,
    expected_network: &str,
    attempt: &SnapshotRefreshAttempt<SnsRefreshAttemptMetadata>,
) -> Result<CacheRefreshAttemptStatus, SnsHostError> {
    let invalid = |reason| SnsHostError::InvalidRefreshAttempt {
        path: path.to_path_buf(),
        reason,
    };
    let status = validate_snapshot_refresh_attempt(attempt, expected_network).map_err(invalid)?;
    if attempt.metadata.id == 0 {
        return Err(invalid("SNS list id must be greater than zero".to_string()));
    }
    let expected_root = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid("attempt path does not contain an SNS root identity".to_string()))?;
    if attempt.metadata.root_canister_id != expected_root {
        return Err(invalid(format!(
            "root_canister_id is {}, expected {expected_root}",
            attempt.metadata.root_canister_id
        )));
    }
    if attempt.metadata.governance_canister_id.is_empty() {
        return Err(invalid(
            "governance_canister_id must not be empty".to_string(),
        ));
    }
    Ok(status)
}

pub(in crate::sns::report) fn read_sns_refresh_attempt(
    path: &Path,
    expected_network: &str,
) -> Option<(SnsRefreshAttempt, CacheRefreshAttemptStatus)> {
    let attempt =
        read_snapshot_refresh_attempt_strict(path, SNS_REFRESH_ATTEMPT_METADATA_FIELDS).ok()??;
    let status = validate_sns_refresh_attempt(path, expected_network, &attempt).ok()?;
    Some((attempt, status))
}

pub(in crate::sns::report) fn read_sns_refresh_attempt_status(
    path: &Path,
    expected_network: &str,
) -> Option<SnsRefreshAttemptStatus> {
    read_sns_refresh_attempt(path, expected_network)
        .map(|(attempt, status)| SnsRefreshAttemptStatus::from_validated(attempt, status))
}

pub(in crate::sns::report) fn read_sns_refresh_attempt_status_strict(
    path: &Path,
    expected_network: &str,
) -> Result<Option<SnsRefreshAttemptStatus>, SnsHostError> {
    read_snapshot_refresh_attempt_strict::<SnsRefreshAttempt>(
        path,
        SNS_REFRESH_ATTEMPT_METADATA_FIELDS,
    )
    .map_err(|error| match error {
        SnapshotRefreshAttemptReadError::Read { path, source } => {
            SnsHostError::ReadCache { path, source }
        }
        SnapshotRefreshAttemptReadError::Parse { path, source } => {
            SnsHostError::ParseCache { path, source }
        }
        SnapshotRefreshAttemptReadError::Invalid { path, reason } => {
            SnsHostError::InvalidRefreshAttempt { path, reason }
        }
    })?
    .map(|attempt| {
        let status = validate_sns_refresh_attempt(path, expected_network, &attempt)?;
        Ok(SnsRefreshAttemptStatus::from_validated(attempt, status))
    })
    .transpose()
}
