//! Module: sns::report::neurons_cache::reports::cache_list
//!
//! Responsibility: build SNS neuron cache list reports.
//! Does not own: snapshot scanning details, text rendering, refresh, or CLI parsing.
//! Boundary: projects cache summaries into stable id-ordered report output.

use crate::sns::report::{
    SnsCacheListReport, SnsCacheListRequest, SnsHostError, build_sns_cache_list_report,
    neurons_cache::{
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, storage::list_sns_neurons_cache_summaries,
    },
};

pub fn build_sns_neurons_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    build_sns_cache_list_report(
        request,
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        list_sns_neurons_cache_summaries,
    )
}
