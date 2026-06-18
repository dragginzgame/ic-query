//! Module: sns::report::source
//!
//! Responsibility: group SNS source models and source traits.
//! Does not own: live transport implementations, cache IO, report assembly, or rendering.
//! Boundary: exposes source-layer contracts used by report builders and tests.

mod model;
mod traits;

pub(in crate::sns::report) use model::{
    MainnetSns, MainnetSnsCanisters, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons,
    MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals, MainnetSnsToken,
    SnsFetchRequest, SnsNeuronId,
};
pub(in crate::sns::report) use traits::{
    SnsListSource, SnsNeuronsSource, SnsParamsSource, SnsProposalSource, SnsProposalsSource,
    SnsTokenSource,
};
