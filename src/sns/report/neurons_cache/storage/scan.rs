//! Module: sns::report::neurons_cache::storage::scan
//!
//! Responsibility: scan SNS neuron cache directories and read snapshot headers.
//! Does not own: full cache loading, lookup policy, refresh, or rendering.
//! Boundary: exposes only complete snapshot paths and validated cache headers.

use crate::sns::report::{
    SnsHostError,
    cache_storage::{collect_sns_cache_paths, read_sns_cache_header},
    neurons_cache::{
        SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCacheHeader,
        paths::SnsNeuronsCacheCollection, storage::errors::SnsNeuronsCacheErrors,
    },
};
use std::path::{Path, PathBuf};

pub(super) fn collect_sns_neurons_cache_paths(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    collect_sns_cache_paths::<SnsNeuronsCacheCollection>(icp_root, network)
}

pub(super) fn read_sns_neurons_cache_header(
    path: &Path,
    network: &str,
) -> Result<SnsNeuronsCacheHeader, SnsHostError> {
    read_sns_cache_header(
        path,
        network,
        SNS_NEURONS_CACHE_SCHEMA_VERSION,
        SnsNeuronsCacheErrors,
    )
}
