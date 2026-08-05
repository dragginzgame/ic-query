//! Module: sns::report::model::sorts
//!
//! Responsibility: group SNS report sort and filter model enums.
//! Does not own: clap value parsing, live request transport, or rendering.
//! Boundary: re-exports stable report-model selectors used by SNS reports.

mod list;
#[cfg(feature = "sns-host")]
mod neurons;
mod proposals;

pub use list::SnsListSort;
#[cfg(feature = "sns-host")]
pub use neurons::SnsNeuronsSort;
#[cfg(feature = "sns-host")]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "sns-host"))]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
pub use proposals::{
    SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
