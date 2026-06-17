//! Module: sns::report::model::reports::neurons::attempt
//!
//! Responsibility: SNS neuron refresh-attempt status DTO.
//! Does not own: attempt-file loading, refresh lifecycle, or rendering.
//! Boundary: preserves latest refresh-attempt metadata for cache status output.

use serde::Serialize;

///
/// SnsNeuronsRefreshAttemptStatus
///
/// Serializable status for the latest SNS neuron snapshot refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsRefreshAttemptStatus {
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}
