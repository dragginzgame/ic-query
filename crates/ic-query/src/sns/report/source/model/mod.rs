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
pub use fetch::SnsSourceRequest;
pub(in crate::sns::report) use list::MainnetSnsCanisters;
pub use list::{MainnetSns, MainnetSnsList};
pub use neurons::{MainnetSnsNeuronPage, MainnetSnsNeurons, SnsNeuronId};
pub use proposals::{MainnetSnsProposal, MainnetSnsProposalPage, MainnetSnsProposals};
pub use token::MainnetSnsToken;
