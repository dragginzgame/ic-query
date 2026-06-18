//! Module: sns::report::neurons_cache::attempt
//!
//! Responsibility: persist and read SNS neuron refresh-attempt sidecars.
//! Does not own: cache snapshots, refresh fetching, report rendering, or CLI parsing.
//! Boundary: tracks refresh lifecycle status and progress for cache status reports.

mod model;
mod read;
mod timestamp;
mod write;

pub(super) use model::{SnsNeuronsAttemptContext, SnsNeuronsAttemptProgress};
pub(super) use read::read_sns_neurons_attempt_status;
pub(super) use write::{
    write_complete_sns_neurons_attempt, write_failed_sns_neurons_attempt,
    write_running_sns_neurons_attempt, write_starting_sns_neurons_attempt,
};
