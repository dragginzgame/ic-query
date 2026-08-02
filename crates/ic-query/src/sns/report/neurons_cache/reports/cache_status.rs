//! Module: sns::report::neurons_cache::reports::cache_status
//!
//! Responsibility: build SNS neuron cache status reports for one SNS input.
//! Does not own: cache refresh, storage implementation, text rendering, or CLI parsing.
//! Boundary: resolves id/root status views over cache snapshots and refresh-attempt sidecars.

use crate::sns::report::{
    SnsCacheStatusReport, SnsCacheStatusRequest, SnsCacheSummary, SnsHostError,
    SnsRefreshAttemptStatus,
    cache_attempt::read_sns_refresh_attempt_status_strict,
    cache_status::{SnsCacheStatusFamily, SnsCacheStatusPaths, build_sns_cache_status_report},
    find_sns_cache_summary_by_id, load_sns_cache_summary_at,
    neurons_cache::{
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        paths::{SnsNeuronsCacheCollection, SnsNeuronsCachePaths},
        storage::{
            collect_sns_neurons_cache_paths, load_sns_neurons_cache_at,
            read_sns_neurons_cache_header,
        },
    },
};
use std::path::{Path, PathBuf};

pub fn build_sns_neurons_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    build_sns_cache_status_report::<SnsNeuronsCacheStatusFamily>(
        request,
        SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    )
}

struct SnsNeuronsCacheStatusFamily;

impl SnsCacheStatusFamily for SnsNeuronsCacheStatusFamily {
    type Collection = SnsNeuronsCacheCollection;

    fn find_cache_by_id(
        cache_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<SnsCacheSummary>, SnsHostError> {
        find_sns_cache_summary_by_id(
            collect_sns_neurons_cache_paths(cache_root, network)?,
            id,
            |path| read_sns_neurons_cache_header(path, network).map(|header| header.metadata.id),
            |path| load_sns_cache_summary_at(path, network, load_sns_neurons_cache_at),
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
    ) -> Result<SnsCacheSummary, SnsHostError> {
        Ok(load_sns_cache_summary_at(
            cache_path,
            network,
            load_sns_neurons_cache_at,
        ))
    }

    fn read_attempt_status(
        attempt_path: &Path,
        network: &str,
    ) -> Result<Option<SnsRefreshAttemptStatus>, SnsHostError> {
        read_sns_refresh_attempt_status_strict(attempt_path, network)
    }
}
