//! Module: sns::report::model::sorts
//!
//! Responsibility: group SNS report sort and filter model enums.
//! Does not own: clap value parsing, live request transport, or rendering.
//! Boundary: re-exports stable report-model selectors used by SNS reports.

mod list;
#[cfg(feature = "host")]
mod neurons;
#[cfg(feature = "host")]
mod proposals;

pub use list::SnsListSort;
#[cfg(feature = "host")]
pub use neurons::SnsNeuronsSort;
#[cfg(feature = "host")]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_STATUS_ADOPTED_CODE, SNS_PROPOSAL_STATUS_REJECTED_CODE,
};
#[cfg(all(test, feature = "host"))]
pub(in crate::sns::report) use proposals::{
    SNS_PROPOSAL_STATUS_EXECUTED_CODE, SNS_PROPOSAL_STATUS_OPEN_CODE,
};
#[cfg(feature = "host")]
pub use proposals::{
    SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
