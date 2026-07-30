//! Module: sns::report::neurons_cache::reports::cache_status
//!
//! Responsibility: build SNS neuron cache status reports for one SNS input.
//! Does not own: cache refresh, storage implementation, text rendering, or CLI parsing.
//! Boundary: resolves id/root status views over cache snapshots and refresh-attempt sidecars.

use crate::sns::report::{
    SnsCacheStatusReport, SnsCacheStatusRequest, SnsCacheSummary, SnsHostError,
    SnsRefreshAttemptStatus,
    cache_attempt::read_sns_refresh_attempt_status_strict,
    cache_status::{SnsCacheStatusFamily, SnsCacheStatusPaths, build_sns_cache_status_lookup},
    find_sns_cache_summary_by_id,
    neurons_cache::{
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        paths::{SnsNeuronsCachePaths, sns_network_cache_dir},
        storage::{
            collect_sns_neurons_cache_paths, load_sns_neurons_cache_summary_at,
            read_sns_neurons_cache_header,
        },
    },
};
use std::path::{Path, PathBuf};

pub fn build_sns_neurons_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    let lookup = build_sns_cache_status_lookup::<SnsNeuronsCacheStatusFamily>(
        &request.network,
        &request.cache_root,
        &request.input,
    )?;
    Ok(cache_status_report(
        request,
        lookup.cache_root,
        lookup.cache,
        lookup.expected_cache_path,
        lookup.refresh_attempt_path,
        lookup.latest_attempt,
    ))
}

fn cache_status_report(
    request: &SnsCacheStatusRequest,
    cache_root: String,
    cache: Option<SnsCacheSummary>,
    expected_cache_path: Option<String>,
    refresh_attempt_path: Option<String>,
    latest_attempt: Option<SnsRefreshAttemptStatus>,
) -> SnsCacheStatusReport {
    SnsCacheStatusReport {
        schema_version: SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root,
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path,
        refresh_attempt_path,
        latest_attempt,
    }
}

struct SnsNeuronsCacheStatusFamily;

impl SnsCacheStatusFamily for SnsNeuronsCacheStatusFamily {
    type Attempt = SnsRefreshAttemptStatus;
    type Summary = SnsCacheSummary;

    const COLLECTION: &'static str = "neurons";

    fn network_cache_dir(cache_root: &Path, network: &str) -> PathBuf {
        sns_network_cache_dir(cache_root, network)
    }

    fn find_cache_by_id(
        cache_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<Self::Summary>, SnsHostError> {
        find_sns_cache_summary_by_id(
            collect_sns_neurons_cache_paths(cache_root, network)?,
            id,
            |path| read_sns_neurons_cache_header(path, network).map(|header| header.metadata.id),
            |path| load_sns_neurons_cache_summary_at(path, network),
        )
    }

    fn root_cache_paths(
        cache_root: &Path,
        network: &str,
        root_canister_id: &str,
    ) -> SnsCacheStatusPaths {
        let paths = SnsNeuronsCachePaths::for_root(cache_root, network, root_canister_id);
        SnsCacheStatusPaths {
            cache_path: paths.cache_path,
            attempt_path: paths.attempt_path,
        }
    }

    fn load_root_cache_summary(
        cache_path: PathBuf,
        network: &str,
    ) -> Result<Self::Summary, SnsHostError> {
        Ok(load_sns_neurons_cache_summary_at(cache_path, network))
    }

    fn read_attempt_status(
        attempt_path: &Path,
        network: &str,
    ) -> Result<Option<Self::Attempt>, SnsHostError> {
        read_sns_refresh_attempt_status_strict(attempt_path, network)
    }
}
