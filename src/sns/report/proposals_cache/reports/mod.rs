//! Module: sns::report::proposals_cache::reports
//!
//! Responsibility: group proposal cache report builders.
//! Does not own: refresh locking, page fetching, cache loading internals, or rendering.
//! Boundary: exposes cache-backed proposal list, cache list, and status reports.

mod cache_list;
mod cache_status;
mod cached_report;

pub use cache_list::build_sns_proposals_cache_list_report;
pub use cache_status::build_sns_proposals_cache_status_report;
pub(in crate::sns::report) use cached_report::build_sns_proposals_report_from_cache_or_refresh;
