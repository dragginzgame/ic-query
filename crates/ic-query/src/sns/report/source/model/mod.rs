//! Module: sns::report::source::model
//!
//! Responsibility: group SNS source result models.
//! Does not own: live transport, report DTOs, cache IO, or rendering.
//! Boundary: re-exports source-layer data passed from fetchers to builders.

mod fetch;
mod list;
mod neurons;
mod proposals;
mod token;

pub(in crate::sns::report) use fetch::SnsFetchRequest;
pub(in crate::sns::report) use list::{MainnetSns, MainnetSnsCanisters, MainnetSnsList};
pub(in crate::sns::report) use neurons::{MainnetSnsNeuronPage, MainnetSnsNeurons, SnsNeuronId};
pub(in crate::sns::report) use proposals::{
    MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals,
};
pub(in crate::sns::report) use token::MainnetSnsToken;
