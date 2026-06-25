//! Module: sns::report::model::sorts
//!
//! Responsibility: group SNS report sort and filter model enums.
//! Does not own: clap value parsing, live request transport, or rendering.
//! Boundary: re-exports stable report-model selectors used by SNS reports.

mod list;
mod neurons;
mod proposals;

pub use list::SnsListSort;
pub use neurons::SnsNeuronsSort;
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(test)]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
pub use proposals::{
    SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
