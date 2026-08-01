//! Module: sns::report::neurons_cache::refresh
//!
//! Responsibility: refresh complete SNS neuron cache snapshots.
//! Does not own: CLI parsing, text rendering, storage lookups, or live transport details.
//! Boundary: coordinates lookup, locking, collection, attempt tracking, and snapshot publishing.

mod publish;
mod run;

use crate::sns::report::{
    SnsNeuronsRefreshRequest, cache_refresh::SnsSnapshotRefreshContext,
    neurons_cache::paths::SnsNeuronsCacheCollection,
};

type SnsNeuronsRefreshContext<'a> =
    SnsSnapshotRefreshContext<'a, SnsNeuronsRefreshRequest, SnsNeuronsCacheCollection>;

pub use run::{
    DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS, refresh_sns_neurons_cache,
    refresh_sns_neurons_cache_with_progress, refresh_sns_neurons_cache_with_source,
};
