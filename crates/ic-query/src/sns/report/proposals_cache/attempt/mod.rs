//! Module: sns::report::proposals_cache::attempt
//!
//! Responsibility: group proposal refresh-attempt status helpers.
//! Does not own: cache publishing, page fetching, or cache status rendering.
//! Boundary: re-exports attempt context, progress, read, and write helpers.

mod model;
mod write;

pub(super) use model::{SnsProposalsAttemptContext, SnsProposalsAttemptProgress};
pub(super) use write::{
    write_complete_attempt, write_failed_attempt, write_running_attempt, write_starting_attempt,
};
