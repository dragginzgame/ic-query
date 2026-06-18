//! Module: sns::report::neurons_cache::storage
//!
//! Responsibility: load, locate, and summarize complete SNS neuron cache snapshots.
//! Does not own: refresh collection, attempt persistence, report rendering, or CLI parsing.
//! Boundary: centralizes cache-file storage reads for neuron cache reports.

mod errors;
mod load;
mod lookup;
mod scan;
mod summary;

pub(super) use load::load_sns_neurons_cache_at;
pub(super) use lookup::{find_sns_neurons_cache_by_id, load_sns_neurons_cache_for_input};
pub(super) use summary::{list_sns_neurons_cache_summaries, sns_neurons_cache_summary};
