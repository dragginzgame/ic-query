//! Module: sns::report::proposals_cache::attempt::read
//!
//! Responsibility: read proposal refresh-attempt status metadata.
//! Does not own: cache snapshot loading, refresh orchestration, or text rendering.
//! Boundary: maps stored refresh attempts into public status report DTOs.

use crate::{
    snapshot_cache::{SnapshotRefreshAttemptReadError, read_snapshot_refresh_attempt_strict},
    sns::report::{
        SnsHostError, SnsRefreshAttemptStatus,
        cache_attempt::{SNS_REFRESH_ATTEMPT_METADATA_FIELDS, validate_sns_refresh_attempt},
        proposals_cache::model::SnsProposalsRefreshAttempt,
    },
};
use std::path::Path;

pub(in crate::sns::report::proposals_cache::attempt) fn read_sns_proposals_attempt(
    path: &Path,
    expected_network: &str,
) -> Option<SnsProposalsRefreshAttempt> {
    let attempt =
        read_snapshot_refresh_attempt_strict(path, SNS_REFRESH_ATTEMPT_METADATA_FIELDS).ok()??;
    validate_sns_refresh_attempt(path, expected_network, &attempt).ok()?;
    Some(attempt)
}

/// Read the latest proposal refresh-attempt status when present.
pub(in crate::sns::report::proposals_cache) fn read_sns_proposals_attempt_status(
    path: &Path,
    expected_network: &str,
) -> Option<SnsRefreshAttemptStatus> {
    let attempt = read_sns_proposals_attempt(path, expected_network)?;
    Some(SnsRefreshAttemptStatus::from(attempt))
}

pub(in crate::sns::report::proposals_cache) fn read_sns_proposals_attempt_status_strict(
    path: &Path,
    expected_network: &str,
) -> Result<Option<SnsRefreshAttemptStatus>, SnsHostError> {
    read_snapshot_refresh_attempt_strict::<SnsProposalsRefreshAttempt>(
        path,
        SNS_REFRESH_ATTEMPT_METADATA_FIELDS,
    )
    .map_err(|err| match err {
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
