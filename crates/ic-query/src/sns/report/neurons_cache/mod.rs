//! Module: sns::report::neurons_cache
//!
//! Responsibility: manage complete SNS neuron snapshot cache behavior.
//! Does not own: CLI parsing, live SNS transport, generic snapshot mechanics, or text rendering.
//! Boundary: coordinates neuron cache refresh, lookup, storage, and cache-backed reports.

mod attempt;
mod collection;
mod model;
mod paths;
mod refresh;
mod reports;
mod storage;

pub use paths::{
    sns_neurons_cache_path, sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
#[cfg(test)]
pub(in crate::sns::report) use refresh::refresh_sns_neurons_cache_with_source;
pub use refresh::{DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS, refresh_sns_neurons_cache};
pub(in crate::sns::report) use reports::build_sns_neurons_report_from_cache;
pub use reports::{build_sns_neurons_cache_list_report, build_sns_neurons_cache_status_report};

pub(super) const SNS_NEURONS_CACHE_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
