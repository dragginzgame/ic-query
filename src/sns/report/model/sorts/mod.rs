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
pub use proposals::{
    SnsProposalSortDirection, SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort,
};
