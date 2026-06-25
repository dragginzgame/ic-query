//! Module: sns::report::cache_attempt
//!
//! Responsibility: shared SNS cache refresh-attempt helper models.
//! Does not own: attempt sidecar IO, cache publication, or text rendering.
//! Boundary: carries in-progress page counters for neuron and proposal refresh attempts.

use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// SnsRefreshAttemptMetadata
///
/// Snapshot refresh-attempt metadata shared by SNS neuron and proposal cache
/// refresh sidecars.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report) struct SnsRefreshAttemptMetadata {
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

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
