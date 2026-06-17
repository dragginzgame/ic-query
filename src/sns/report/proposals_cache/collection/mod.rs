//! Module: sns::report::proposals_cache::collection
//!
//! Responsibility: group complete paged SNS proposal collection fetching.
//! Does not own: cache publishing, command parsing, or report rendering.
//! Boundary: re-exports the complete proposal collection fetcher.

mod fetch;
mod progress;

pub(super) use fetch::fetch_complete_sns_proposals;
