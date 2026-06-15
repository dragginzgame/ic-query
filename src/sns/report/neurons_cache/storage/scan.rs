use crate::{
    cache_file::LoadJsonCacheRequest,
    snapshot_cache::{collect_full_collection_snapshot_paths, load_snapshot_header},
    sns::report::SnsHostError,
};
use std::path::{Path, PathBuf};

use super::super::{
    SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCacheHeader, paths::sns_network_cache_dir,
};
use super::errors::SnsNeuronsCacheErrors;

pub(super) fn collect_sns_neurons_cache_paths(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    let root = sns_network_cache_dir(icp_root, network);
    collect_full_collection_snapshot_paths(&root, "neurons")
        .map_err(|source| SnsHostError::ReadCache { path: root, source })
}

pub(super) fn read_sns_neurons_cache_header(
    path: &Path,
    network: &str,
) -> Result<SnsNeuronsCacheHeader, SnsHostError> {
    load_snapshot_header(
        LoadJsonCacheRequest {
            path: path.to_path_buf(),
            network,
            expected_schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        },
        SnsNeuronsCacheErrors,
    )
}
