//! Module: sns::report::neurons_cache::reports
//!
//! Responsibility: build SNS neuron cache reports from stored snapshots.
//! Does not own: cache refresh, cache file IO details, text rendering, or CLI parsing.
//! Boundary: exposes cache list/status reports and cache-backed neuron list reports.

mod cached_report;

use super::{
    SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    paths::SnsNeuronsCacheCollection,
};
use crate::sns::report::{
    SnsCacheListReport, SnsCacheListRequest, SnsCacheStatusReport, SnsCacheStatusRequest,
    SnsHostError, build_sns_cache_list_report, cache_status::build_sns_cache_status_report,
};

pub(in crate::sns::report) use cached_report::build_sns_neurons_report_from_cache;

/// Build a local SNS neuron cache list report.
pub fn build_sns_neurons_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    build_sns_cache_list_report::<SnsNeuronsCacheCollection>(
        request,
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION,
    )
}

/// Build an SNS neuron cache status report for one SNS input.
pub fn build_sns_neurons_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    build_sns_cache_status_report::<SnsNeuronsCacheCollection>(
        request,
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    )
}
