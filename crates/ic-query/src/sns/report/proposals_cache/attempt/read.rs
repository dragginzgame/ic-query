//! Module: sns::report::proposals_cache::attempt::read
//!
//! Responsibility: read proposal refresh-attempt status metadata.
//! Does not own: cache snapshot loading, refresh orchestration, or text rendering.
//! Boundary: maps stored refresh attempts into public status report DTOs.

use crate::{
    snapshot_cache::{
        SnapshotRefreshAttemptReadError, read_snapshot_refresh_attempt,
        read_snapshot_refresh_attempt_strict,
    },
    sns::report::{
        SnsHostError, SnsProposalsRefreshAttemptStatus,
        proposals_cache::model::SnsProposalsRefreshAttempt,
    },
};
use std::path::Path;

pub(in crate::sns::report::proposals_cache::attempt) fn read_sns_proposals_attempt(
    path: &Path,
) -> Option<SnsProposalsRefreshAttempt> {
    read_snapshot_refresh_attempt(path)
}

/// Read the latest proposal refresh-attempt status when present.
pub(in crate::sns::report::proposals_cache) fn read_sns_proposals_attempt_status(
    path: &Path,
) -> Option<SnsProposalsRefreshAttemptStatus> {
    let attempt = read_sns_proposals_attempt(path)?;
    Some(SnsProposalsRefreshAttemptStatus::from(attempt))
}

pub(in crate::sns::report::proposals_cache) fn read_sns_proposals_attempt_status_strict(
    path: &Path,
) -> Result<Option<SnsProposalsRefreshAttemptStatus>, SnsHostError> {
    read_snapshot_refresh_attempt_strict::<SnsProposalsRefreshAttempt>(path)
        .map(|attempt| attempt.map(SnsProposalsRefreshAttemptStatus::from))
        .map_err(|err| match err {
            SnapshotRefreshAttemptReadError::Read { path, source } => {
                SnsHostError::ReadCache { path, source }
            }
            SnapshotRefreshAttemptReadError::Parse { path, source } => {
                SnsHostError::ParseCache { path, source }
            }
        })
}
