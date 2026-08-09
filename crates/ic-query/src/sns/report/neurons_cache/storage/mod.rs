//! Module: sns::report::neurons_cache::storage
//!
//! Responsibility: group neuron cache loading, lookup, and storage identity.
//! Does not own: refresh collection, attempt persistence, report rendering, or CLI parsing.
//! Boundary: centralizes cache-file storage reads for neuron cache reports.

use crate::sns::report::{
    SnsHostError,
    cache_storage::SnsCacheStorageFamily,
    neurons_cache::{
        SNS_NEURONS_CACHE_SCHEMA_VERSION,
        model::{SNS_NEURONS_CACHE_FIELDS, SnsNeuronsCacheRows},
        paths::SnsNeuronsCacheCollection,
    },
    source::validate_sns_neuron_rows,
};
use std::path::PathBuf;

impl SnsCacheStorageFamily for SnsNeuronsCacheCollection {
    type Data = SnsNeuronsCacheRows;

    const CACHE_SCHEMA_VERSION: u32 = SNS_NEURONS_CACHE_SCHEMA_VERSION;
    const CACHE_FIELDS: &'static [&'static str] = SNS_NEURONS_CACHE_FIELDS;
    const CACHE_ITEM_NAME: &'static str = "neuron";

    fn missing_cache_error(path: PathBuf) -> SnsHostError {
        SnsHostError::MissingNeuronsCache { path }
    }

    fn missing_cache_for_id(id: usize, root: PathBuf) -> SnsHostError {
        SnsHostError::MissingNeuronsCacheForId { id, root }
    }

    fn row_count(data: &Self::Data) -> usize {
        data.neurons.len()
    }

    fn validate_rows(data: &Self::Data) -> Result<(), String> {
        validate_sns_neuron_rows(&data.neurons)
    }
}
