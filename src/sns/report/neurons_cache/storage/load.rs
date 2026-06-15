use super::errors::{SnsNeuronsCacheErrors, incomplete_cache_error};
use crate::{
    cache_file::{CachedJsonReport, LoadJsonCacheRequest, load_json_cache},
    sns::report::SnsHostError,
};
use std::path::PathBuf;

use super::super::{SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SnsNeuronsCache};

pub(in crate::sns::report::neurons_cache) fn load_sns_neurons_cache_at(
    path: PathBuf,
    network: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    let cached: CachedJsonReport<SnsNeuronsCache> = load_json_cache(
        LoadJsonCacheRequest {
            path,
            network,
            expected_schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        },
        SnsNeuronsCacheErrors,
    )?;
    if !cached.report.completeness.is_api_exhausted() {
        return Err(incomplete_cache_error(
            cached.report.completeness.page_count,
            cached.report.completeness.row_count,
        ));
    }
    Ok(cached.report)
}
