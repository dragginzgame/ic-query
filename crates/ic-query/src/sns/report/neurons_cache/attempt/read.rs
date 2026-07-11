//! Module: sns::report::neurons_cache::attempt::read
//!
//! Responsibility: read SNS neuron refresh-attempt sidecars.
//! Does not own: attempt writes, cache snapshots, refresh execution, or text rendering.
//! Boundary: projects persisted attempt envelopes into public status report DTOs.

use super::model::SnsNeuronsRefreshAttempt;
use crate::{
    snapshot_cache::{SnapshotRefreshAttemptReadError, read_snapshot_refresh_attempt_strict},
    sns::report::{
        SnsHostError, SnsRefreshAttemptStatus,
        cache_attempt::{SNS_REFRESH_ATTEMPT_METADATA_FIELDS, validate_sns_refresh_attempt},
    },
};
use std::path::Path;

pub(in crate::sns::report::neurons_cache::attempt) fn read_sns_neurons_attempt(
    path: &Path,
    expected_network: &str,
) -> Option<SnsNeuronsRefreshAttempt> {
    let attempt =
        read_snapshot_refresh_attempt_strict(path, SNS_REFRESH_ATTEMPT_METADATA_FIELDS).ok()??;
    validate_sns_refresh_attempt(path, expected_network, &attempt).ok()?;
    Some(attempt)
}

pub(in crate::sns::report::neurons_cache) fn read_sns_neurons_attempt_status(
    path: &Path,
    expected_network: &str,
) -> Option<SnsRefreshAttemptStatus> {
    read_sns_neurons_attempt(path, expected_network).map(SnsRefreshAttemptStatus::from)
}

pub(in crate::sns::report::neurons_cache) fn read_sns_neurons_attempt_status_strict(
    path: &Path,
    expected_network: &str,
) -> Result<Option<SnsRefreshAttemptStatus>, SnsHostError> {
    read_snapshot_refresh_attempt_strict::<SnsNeuronsRefreshAttempt>(
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
