mod cache_list;
mod cache_status;
mod detail;
mod list;
mod refresh;
mod single;

pub use cache_list::sns_proposals_cache_list_report_text;
pub use cache_status::sns_proposals_cache_status_report_text;
pub use list::sns_proposals_report_text;
pub use refresh::sns_proposals_refresh_report_text;
pub use single::sns_proposal_report_text;

pub(super) const SNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;
pub(super) const SNS_PROPOSAL_TITLE_TEXT_LIMIT: usize = 96;
