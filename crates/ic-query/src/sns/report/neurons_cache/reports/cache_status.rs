//! Module: sns::report::neurons_cache::reports::cache_status
//!
//! Responsibility: build SNS neuron cache status reports for one SNS input.
//! Does not own: cache refresh, storage implementation, text rendering, or CLI parsing.
//! Boundary: resolves id/root status views over cache snapshots and refresh-attempt sidecars.

use crate::sns::report::{
    SnsCacheStatusReport, SnsCacheStatusRequest, SnsHostError,
    cache_status::build_sns_cache_status_report,
    neurons_cache::{
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION, paths::SnsNeuronsCacheCollection,
    },
};

pub fn build_sns_neurons_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    build_sns_cache_status_report::<SnsNeuronsCacheCollection>(
        request,
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    )
}
