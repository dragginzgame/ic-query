//! Module: sns::report::neurons_cache::collection
//!
//! Responsibility: collect complete SNS neuron pages for snapshot refresh.
//! Does not own: cache publishing, attempt sidecar storage, report construction, or CLI parsing.
//! Boundary: fetches and folds paged governance neuron responses into a complete collection.

mod fetch;

pub(super) use fetch::fetch_complete_sns_neurons;
