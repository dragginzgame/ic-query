//! Module: sns::report::cache_attempt
//!
//! Responsibility: shared SNS cache refresh-attempt helper models.
//! Does not own: attempt sidecar IO, cache publication, or text rendering.
//! Boundary: carries in-progress page counters for neuron and proposal refresh attempts.

use crate::{
    snapshot_cache::{
        SnapshotRefreshAttempt, SnapshotRefreshAttemptReadError,
        read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
    },
    sns::report::{SnsHostError, SnsRefreshAttemptStatus},
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
/// SnsRefreshAttemptProgress
///
/// In-progress page and row counters persisted by SNS cache refresh attempts.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::report) struct SnsRefreshAttemptProgress {
    pub(in crate::sns::report) pages_fetched: u32,
    pub(in crate::sns::report) rows_fetched: usize,
    pub(in crate::sns::report) last_cursor: Option<String>,
}

impl SnsRefreshAttemptProgress {
    pub(in crate::sns::report) const fn new(
        pages_fetched: u32,
        rows_fetched: usize,
        last_cursor: Option<String>,
    ) -> Self {
        Self {
            pages_fetched,
            rows_fetched,
            last_cursor,
        }
    }

    pub(in crate::sns::report) const fn starting() -> Self {
        Self {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
        }
    }
}

pub(in crate::sns::report) fn validate_sns_refresh_attempt(
    path: &Path,
    expected_network: &str,
    attempt: &SnapshotRefreshAttempt<SnsRefreshAttemptMetadata>,
) -> Result<(), SnsHostError> {
    let invalid = |reason| SnsHostError::InvalidRefreshAttempt {
        path: path.to_path_buf(),
        reason,
    };
    validate_snapshot_refresh_attempt(attempt, expected_network).map_err(invalid)?;
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
    Ok(())
}

pub(in crate::sns::report) fn read_sns_refresh_attempt(
    path: &Path,
    expected_network: &str,
) -> Option<SnsRefreshAttempt> {
    let attempt =
        read_snapshot_refresh_attempt_strict(path, SNS_REFRESH_ATTEMPT_METADATA_FIELDS).ok()??;
    validate_sns_refresh_attempt(path, expected_network, &attempt).ok()?;
    Some(attempt)
}

pub(in crate::sns::report) fn read_sns_refresh_attempt_status(
    path: &Path,
    expected_network: &str,
) -> Option<SnsRefreshAttemptStatus> {
    read_sns_refresh_attempt(path, expected_network).map(SnsRefreshAttemptStatus::from)
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
        validate_sns_refresh_attempt(path, expected_network, &attempt)?;
        Ok(SnsRefreshAttemptStatus::from(attempt))
    })
    .transpose()
}
