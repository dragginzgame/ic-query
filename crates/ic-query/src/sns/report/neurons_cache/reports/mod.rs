//! Module: sns::report::neurons_cache::reports
//!
//! Responsibility: build SNS neuron cache reports from stored snapshots.
//! Does not own: cache refresh, cache file IO details, text rendering, or CLI parsing.
//! Boundary: exposes cache list/status reports and cache-backed neuron list reports.

mod cached_report;

pub use crate::sns::report::{
    cache_status::build_sns_neurons_cache_status_report,
    cache_summary::build_sns_neurons_cache_list_report,
};

pub(in crate::sns::report) use cached_report::build_sns_neurons_report_from_cache;
