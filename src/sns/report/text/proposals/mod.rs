mod detail;
mod list;
mod single;

pub use list::sns_proposals_report_text;
pub use single::sns_proposal_report_text;

pub(super) const SNS_PROPOSAL_DETAIL_TEXT_LIMIT: usize = 240;
pub(super) const SNS_PROPOSAL_TITLE_TEXT_LIMIT: usize = 96;
