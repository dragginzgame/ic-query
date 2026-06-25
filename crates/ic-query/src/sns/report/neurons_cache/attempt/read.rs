//! Module: sns::report::neurons_cache::attempt::read
//!
//! Responsibility: read SNS neuron refresh-attempt sidecars.
//! Does not own: attempt writes, cache snapshots, refresh execution, or text rendering.
//! Boundary: projects persisted attempt envelopes into public status report DTOs.

use super::model::SnsNeuronsRefreshAttempt;
use crate::{
    snapshot_cache::read_snapshot_refresh_attempt, sns::report::SnsNeuronsRefreshAttemptStatus,
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
