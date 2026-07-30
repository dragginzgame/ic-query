//! Module: sns::report::neurons_cache::reports::cache_list
//!
//! Responsibility: build SNS neuron cache list reports.
//! Does not own: snapshot scanning details, text rendering, refresh, or CLI parsing.
//! Boundary: projects cache summaries into stable id-ordered report output.

use crate::sns::report::{
    SnsCacheListFamily, SnsCacheListReport, SnsCacheListRequest, SnsCacheSummary, SnsHostError,
    build_sns_cache_list_lookup,
    neurons_cache::{
        SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION, paths::sns_network_cache_dir,
        storage::list_sns_neurons_cache_summaries,
    },
};
use std::path::{Path, PathBuf};

pub fn build_sns_neurons_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    let lookup = build_sns_cache_list_lookup::<SnsNeuronsCacheListFamily>(
        &request.network,
        &request.cache_root,
    )?;
    Ok(SnsCacheListReport {
        schema_version: SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: lookup.cache_root,
        cache_count: lookup.caches.len(),
        caches: lookup.caches,
    })
}

struct SnsNeuronsCacheListFamily;

impl SnsCacheListFamily for SnsNeuronsCacheListFamily {
    type Summary = SnsCacheSummary;

    fn network_cache_dir(cache_root: &Path, network: &str) -> PathBuf {
        sns_network_cache_dir(cache_root, network)
    }

    fn list_cache_summaries(
        cache_root: &Path,
        network: &str,
    ) -> Result<Vec<Self::Summary>, SnsHostError> {
        list_sns_neurons_cache_summaries(cache_root, network)
    }
}
