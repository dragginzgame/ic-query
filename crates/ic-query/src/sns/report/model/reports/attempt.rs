//! Module: sns::report::model::reports::attempt
//!
//! Responsibility: shared SNS snapshot refresh-attempt status DTO.
//! Does not own: attempt sidecar loading, refresh lifecycle, or text rendering.
//! Boundary: preserves the JSON fields used by neuron cache status reports.

use crate::{
    snapshot_cache::SnapshotRefreshAttempt, sns::report::cache_attempt::SnsRefreshAttemptMetadata,
};
use serde::Serialize;

///
/// SnsRefreshAttemptStatus
///
/// Serializable status for the latest SNS snapshot refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsRefreshAttemptStatus {
    pub network: String,
    pub source_endpoint: String,

    pub governance_canister_id: String,
    /// Validated shared lifecycle evidence.
    #[serde(flatten)]
    pub lifecycle: crate::cache::SnapshotRefreshAttemptStatus,
}

impl SnsRefreshAttemptStatus {
    pub(in crate::sns::report) fn from_validated(
        attempt: SnapshotRefreshAttempt<SnsRefreshAttemptMetadata>,
    ) -> Self {
        Self {
            network: attempt.network.clone(),
            source_endpoint: attempt.source_endpoint.clone(),

            governance_canister_id: attempt.metadata.governance_canister_id.clone(),
            lifecycle: attempt.into(),
        }
    }
}
