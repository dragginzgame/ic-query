//! Module: sns::report::model::reports::attempt
//!
//! Responsibility: shared SNS snapshot refresh-attempt status DTO.
//! Does not own: attempt sidecar loading, refresh lifecycle, or text rendering.
//! Boundary: preserves the JSON fields used by neuron and proposal cache status reports.

use crate::snapshot_cache::SnapshotRefreshAttempt;
use serde::Serialize;

///
/// SnsRefreshAttemptStatus
///
/// Serializable status for the latest SNS snapshot refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsRefreshAttemptStatus {
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}

impl<Metadata> From<SnapshotRefreshAttempt<Metadata>> for SnsRefreshAttemptStatus {
    fn from(attempt: SnapshotRefreshAttempt<Metadata>) -> Self {
        Self {
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
}
