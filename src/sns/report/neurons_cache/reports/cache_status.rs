//! Module: sns::report::neurons_cache::reports::cache_status
//!
//! Responsibility: build SNS neuron cache status reports for one SNS input.
//! Does not own: cache refresh, storage implementation, text rendering, or CLI parsing.
//! Boundary: resolves id/root status views over cache snapshots and refresh-attempt sidecars.

use crate::sns::report::{
    SnsHostError, SnsNeuronsCacheStatusReport, SnsNeuronsCacheStatusRequest,
    SnsNeuronsCacheSummary, SnsNeuronsRefreshAttemptStatus,
    cache_status::{
        SnsCacheStatusFamily, SnsCacheStatusPaths, SnsCacheStatusSummaryView,
        build_sns_cache_status_lookup,
    },
    neurons_cache::{
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        attempt::read_sns_neurons_attempt_status,
        paths::{SnsNeuronsCachePaths, sns_network_cache_dir},
        storage::{
            find_sns_neurons_cache_by_id, load_sns_neurons_cache_at, sns_neurons_cache_summary,
        },
    },
};
use std::path::{Path, PathBuf};

pub fn build_sns_neurons_cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
) -> Result<SnsNeuronsCacheStatusReport, SnsHostError> {
    let lookup = build_sns_cache_status_lookup::<SnsNeuronsCacheStatusFamily>(
        &request.network,
        &request.icp_root,
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
    request: &SnsNeuronsCacheStatusRequest,
    cache_root: String,
    cache: Option<SnsNeuronsCacheSummary>,
    expected_cache_path: Option<String>,
    refresh_attempt_path: Option<String>,
    latest_attempt: Option<SnsNeuronsRefreshAttemptStatus>,
) -> SnsNeuronsCacheStatusReport {
    SnsNeuronsCacheStatusReport {
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
    type Attempt = SnsNeuronsRefreshAttemptStatus;
    type Summary = SnsNeuronsCacheSummary;

    fn network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
        sns_network_cache_dir(icp_root, network)
    }

    fn find_cache_by_id(
        icp_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<Self::Summary>, SnsHostError> {
        Ok(find_sns_neurons_cache_by_id(icp_root, network, id)?
            .map(|(path, cache)| sns_neurons_cache_summary(path, cache)))
    }

    fn root_cache_paths(
        icp_root: &Path,
        network: &str,
        root_canister_id: &str,
    ) -> SnsCacheStatusPaths {
        let paths = SnsNeuronsCachePaths::for_root(icp_root, network, root_canister_id);
        SnsCacheStatusPaths {
            cache_path: paths.cache_path,
            attempt_path: paths.attempt_path,
        }
    }

    fn load_root_cache_summary(
        cache_path: PathBuf,
        network: &str,
    ) -> Result<Self::Summary, SnsHostError> {
        Ok(sns_neurons_cache_summary(
            cache_path.clone(),
            load_sns_neurons_cache_at(cache_path, network)?,
        ))
    }

    fn read_attempt_status(attempt_path: &Path) -> Option<Self::Attempt> {
        read_sns_neurons_attempt_status(attempt_path)
    }
}

impl SnsCacheStatusSummaryView for SnsNeuronsCacheSummary {
    type Attempt = SnsNeuronsRefreshAttemptStatus;

    fn refresh_attempt_path(&self) -> &str {
        &self.refresh_attempt_path
    }

    fn latest_attempt(&self) -> Option<Self::Attempt> {
        self.latest_attempt.clone()
    }
}
