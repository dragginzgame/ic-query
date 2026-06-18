//! Module: sns::report::neurons_cache::storage::load
//!
//! Responsibility: load one complete SNS neuron cache snapshot from disk.
//! Does not own: lookup resolution, cache path construction, refresh, or rendering.
//! Boundary: validates schema, network, and completeness before returning a cache model.

use super::errors::{SnsNeuronsCacheErrors, incomplete_cache_error};
use crate::{
    cache_file::LoadJsonCacheRequest,
    snapshot_cache::load_complete_snapshot,
    sns::report::{
        SnsHostError,
        neurons_cache::{SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCache},
    },
};
use std::path::PathBuf;

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
