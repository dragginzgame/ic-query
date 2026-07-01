//! Module: sns::report::source
//!
//! Responsibility: group SNS source models and source traits.
//! Does not own: live transport implementations, cache IO, report assembly, or rendering.
//! Boundary: exposes source-layer contracts used by report builders and tests.

mod model;
mod traits;

pub use model::{MainnetSns, MainnetSnsList, MainnetSnsToken, SnsSourceRequest};
pub(in crate::sns::report) use model::{
    MainnetSnsCanisters, MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal,
    MainnetSnsProposalPage, MainnetSnsProposals, SnsFetchRequest, SnsNeuronId,
};
pub use traits::{SnsListSource, SnsParamsSource, SnsTokenSource};
pub(in crate::sns::report) use traits::{SnsNeuronsSource, SnsProposalSource, SnsProposalsSource};
