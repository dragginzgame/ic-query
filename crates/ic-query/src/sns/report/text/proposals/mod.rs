//! Module: sns::report::text::proposals
//!
//! Responsibility: group SNS proposal text report renderers.
//! Does not own: proposal fetching, cache loading, report construction, or JSON output.
//! Boundary: renders proposal list, detail, refresh, and cache DTOs for humans.

#[cfg(feature = "host")]
mod cache_list;
#[cfg(feature = "host")]
mod cache_status;
mod detail;
mod list;
#[cfg(feature = "host")]
mod refresh;
mod single;

#[cfg(feature = "host")]
pub use cache_list::sns_proposals_cache_list_report_text;
#[cfg(feature = "host")]
pub use cache_status::sns_proposals_cache_status_report_text;
pub use list::sns_proposals_report_text;
#[cfg(feature = "host")]
pub use refresh::sns_proposals_refresh_report_text;
pub use single::sns_proposal_report_text;

pub(super) const SNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;
pub(super) const SNS_PROPOSAL_TITLE_TEXT_LIMIT: usize = 96;
