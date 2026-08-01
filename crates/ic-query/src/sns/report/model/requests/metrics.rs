//! Module: sns::report::model::requests::metrics
//!
//! Responsibility: bounded SNS Governance metrics request contract.
//! Does not own: CLI parsing, discovery, live calls, or rendering.
//! Boundary: validates the proposal-count window before any source access.

#[cfg(feature = "host")]
use crate::sns::report::{SnsHostError, SnsLookupRequest};

/// Default recent-proposal window used by SNS metrics reports.
pub const DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS: u64 = 30 * 24 * 60 * 60;

/// Largest recent-proposal window accepted by SNS metrics reports.
pub const MAX_SNS_METRICS_TIME_WINDOW_SECONDS: u64 = 365 * 24 * 60 * 60;

///
/// SnsMetricsRequest
///
/// Request for one bounded SNS Governance metrics report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsMetricsRequest {
    /// Requested IC network identity.
    pub network: String,
    /// IC API endpoint used for discovery and Governance queries.
    pub source_endpoint: String,
    /// Caller-supplied collection time in Unix seconds.
    pub now_unix_secs: u64,
    /// SNS list id or Root principal.
    pub input: String,
    /// Window used for recent submitted/executed proposal counts.
    pub time_window_seconds: u64,
}

impl SnsMetricsRequest {
    /// Construct a request with the default 30-day proposal-count window.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input: input.into(),
            time_window_seconds: DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS,
        }
    }

    /// Replace the proposal-count window.
    #[must_use]
    pub const fn with_time_window_seconds(mut self, time_window_seconds: u64) -> Self {
        self.time_window_seconds = time_window_seconds;
        self
    }
}

#[cfg(feature = "host")]
pub(in crate::sns::report) fn validate_sns_metrics_request(
    request: &SnsMetricsRequest,
) -> Result<(), SnsHostError> {
    validate_sns_metrics_time_window(request.time_window_seconds)
}

#[cfg(feature = "host")]
pub(in crate::sns::report) fn validate_sns_metrics_time_window(
    time_window_seconds: u64,
) -> Result<(), SnsHostError> {
    if !(1..=MAX_SNS_METRICS_TIME_WINDOW_SECONDS).contains(&time_window_seconds) {
        return Err(SnsHostError::InvalidMetricsTimeWindow {
            seconds: time_window_seconds,
            max_seconds: MAX_SNS_METRICS_TIME_WINDOW_SECONDS,
        });
    }
    Ok(())
}

#[cfg(feature = "host")]
pub(in crate::sns::report) fn sns_metrics_lookup_request(
    request: &SnsMetricsRequest,
) -> SnsLookupRequest {
    SnsLookupRequest {
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
        input: request.input.clone(),
    }
}
