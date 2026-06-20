//! Module: sns::report::proposals_cache::storage
//!
//! Responsibility: group proposal snapshot cache loading, lookup, and summaries.
//! Does not own: refresh orchestration, report status assembly, or text rendering.
//! Boundary: re-exports storage helpers used by proposal cache reports.

mod load;
mod lookup;
mod scan;
mod summary;

pub(super) use lookup::load_sns_proposals_cache_for_input_with_path;
pub(super) use summary::{list_sns_proposals_cache_summaries, load_sns_proposals_cache_summary_at};
