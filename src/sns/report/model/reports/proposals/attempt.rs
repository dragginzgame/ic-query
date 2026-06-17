//! Module: sns::report::model::reports::proposals::attempt
//!
//! Responsibility: define SNS proposal refresh-attempt status report DTOs.
//! Does not own: attempt file storage, refresh orchestration, or rendering.
//! Boundary: carries attempt metadata for cache status text and JSON output.

use serde::Serialize;

///
/// SnsProposalsRefreshAttemptStatus
///
/// Serializable status for the latest SNS proposal snapshot refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsRefreshAttemptStatus {
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}
