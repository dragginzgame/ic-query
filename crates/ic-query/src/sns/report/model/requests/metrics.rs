//! Module: sns::report::model::requests::metrics
//!
//! Responsibility: bounded SNS Governance metrics request contract.
//! Does not own: CLI parsing, discovery, live calls, or rendering.
//! Boundary: validates the proposal-count window before any source access.

use super::lookup::SnsLookupRequest;
#[cfg(feature = "sns-host")]
use crate::sns::report::SnsHostError;

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
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
    /// Window used for recent submitted/executed proposal counts.
    pub time_window_seconds: u64,
}

impl SnsMetricsRequest {
    /// Construct a request with the default 30-day proposal-count window.
    #[must_use]
    pub const fn new(lookup: SnsLookupRequest) -> Self {
        Self {
            lookup,
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

#[cfg(feature = "sns-host")]
pub(in crate::sns::report) fn validate_sns_metrics_request(
    request: &SnsMetricsRequest,
) -> Result<(), SnsHostError> {
    validate_sns_metrics_time_window(request.time_window_seconds)
}

#[cfg(feature = "sns-host")]
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
