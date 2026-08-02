//! Module: sns::report::neurons_cache::reports::cache_list
//!
//! Responsibility: build SNS neuron cache list reports.
//! Does not own: snapshot scanning details, text rendering, refresh, or CLI parsing.
//! Boundary: projects cache summaries into stable id-ordered report output.

use crate::sns::report::{
    SnsCacheListReport, SnsCacheListRequest, SnsHostError, build_sns_cache_list_report,
    cache_storage::collect_sns_cache_paths,
    load_sns_cache_summaries,
    neurons_cache::{
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, paths::SnsNeuronsCacheCollection,
        storage::load_sns_neurons_cache_at,
    },
};

pub fn build_sns_neurons_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    build_sns_cache_list_report(
        request,
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        |cache_root, network| {
            let paths = collect_sns_cache_paths::<SnsNeuronsCacheCollection>(cache_root, network)?;
            Ok(load_sns_cache_summaries(
                paths,
                network,
                load_sns_neurons_cache_at,
            ))
        },
    )
}
