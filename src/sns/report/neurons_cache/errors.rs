//! Module: sns::report::neurons_cache::errors
//!
//! Responsibility: re-export shared SNS cache-file error mapping for neuron caches.
//! Does not own: host error definitions, cache IO, refresh orchestration, or rendering.
//! Boundary: keeps neuron cache modules using the shared SNS cache error adapter.

pub(super) use crate::sns::report::cache_error::sns_cache_file_error;
