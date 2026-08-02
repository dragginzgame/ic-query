//! Module: sns::report::neurons_cache::storage
//!
//! Responsibility: group neuron cache loading, lookup, and storage identity.
//! Does not own: refresh collection, attempt persistence, report rendering, or CLI parsing.
//! Boundary: centralizes cache-file storage reads for neuron cache reports.

use crate::sns::report::{
    SnsHostError,
    cache_storage::SnsCacheStorageFamily,
    neurons_cache::{
        SNS_NEURONS_CACHE_SCHEMA_VERSION, model::SNS_NEURONS_CACHE_FIELDS,
        paths::SnsNeuronsCacheCollection,
    },
};
use std::path::PathBuf;

mod errors;
mod load;
mod lookup;

pub(super) use load::load_sns_neurons_cache_at;
pub(super) use lookup::load_sns_neurons_cache_for_input;

impl SnsCacheStorageFamily for SnsNeuronsCacheCollection {
    const CACHE_SCHEMA_VERSION: u32 = SNS_NEURONS_CACHE_SCHEMA_VERSION;
    const CACHE_FIELDS: &'static [&'static str] = SNS_NEURONS_CACHE_FIELDS;

    fn missing_cache_error(path: PathBuf) -> SnsHostError {
        SnsHostError::MissingNeuronsCache { path }
    }
}
