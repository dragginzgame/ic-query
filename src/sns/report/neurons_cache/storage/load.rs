use super::errors::{SnsNeuronsCacheErrors, incomplete_cache_error};
use crate::{
    cache_file::LoadJsonCacheRequest, snapshot_cache::load_complete_snapshot,
    sns::report::SnsHostError,
};
use std::path::PathBuf;

use super::super::{SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCache};

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_at(
    path: PathBuf,
    network: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    load_complete_snapshot(
        LoadJsonCacheRequest {
            path,
            network,
            expected_schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        },
        SnsNeuronsCacheErrors,
        |completeness| incomplete_cache_error(completeness.page_count, completeness.row_count),
    )
}
