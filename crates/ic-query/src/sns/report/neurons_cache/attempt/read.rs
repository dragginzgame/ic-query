//! Module: sns::report::neurons_cache::attempt::read
//!
//! Responsibility: read SNS neuron refresh-attempt sidecars.
//! Does not own: attempt writes, cache snapshots, refresh execution, or text rendering.
//! Boundary: projects persisted attempt envelopes into public status report DTOs.

use super::model::SnsNeuronsRefreshAttempt;
use crate::{
    snapshot_cache::{
        SnapshotRefreshAttemptReadError, read_snapshot_refresh_attempt,
        read_snapshot_refresh_attempt_strict,
    },
    sns::report::{SnsHostError, SnsNeuronsRefreshAttemptStatus},
};
use std::path::Path;

pub(in crate::sns::report::neurons_cache::attempt) fn read_sns_neurons_attempt(
    path: &Path,
) -> Option<SnsNeuronsRefreshAttempt> {
    read_snapshot_refresh_attempt(path)
}

pub(in crate::sns::report::neurons_cache) fn read_sns_neurons_attempt_status(
    path: &Path,
) -> Option<SnsNeuronsRefreshAttemptStatus> {
    read_sns_neurons_attempt(path).map(SnsNeuronsRefreshAttemptStatus::from)
}

pub(in crate::sns::report::neurons_cache) fn read_sns_neurons_attempt_status_strict(
    path: &Path,
) -> Result<Option<SnsNeuronsRefreshAttemptStatus>, SnsHostError> {
    read_snapshot_refresh_attempt_strict::<SnsNeuronsRefreshAttempt>(path)
        .map(|attempt| attempt.map(SnsNeuronsRefreshAttemptStatus::from))
        .map_err(|err| match err {
            SnapshotRefreshAttemptReadError::Read { path, source } => {
                SnsHostError::ReadCache { path, source }
            }
            SnapshotRefreshAttemptReadError::Parse { path, source } => {
                SnsHostError::ParseCache { path, source }
            }
        })
}
