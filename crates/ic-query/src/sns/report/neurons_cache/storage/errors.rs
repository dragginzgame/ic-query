//! Module: sns::report::neurons_cache::storage::errors
//!
//! Responsibility: build SNS neuron cache lookup failures.
//! Does not own: shared cache parsing, refresh attempts, or rendering.
//! Boundary: retains neuron-specific lookup context not shared by cache families.

use crate::sns::report::SnsHostError;
use std::path::PathBuf;

pub(super) const fn missing_id_error(id: usize, root: PathBuf) -> SnsHostError {
    SnsHostError::MissingNeuronsCacheForId { id, root }
}

pub(super) fn invalid_lookup_error(input: &str) -> SnsHostError {
    SnsHostError::InvalidLookup {
        input: input.to_string(),
    }
}
