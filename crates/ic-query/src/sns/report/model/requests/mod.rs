//! Module: sns::report::model::requests
//!
//! Responsibility: group SNS report request DTOs.
//! Does not own: CLI parsing, live source calls, cache IO, or rendering.
//! Boundary: re-exports request contracts accepted by SNS report builders.

mod list;
mod lookup;
mod neurons;
mod proposals;

pub use list::SnsListRequest;
pub use lookup::{SnsInfoRequest, SnsLookupRequest, SnsParamsRequest, SnsTokenRequest};
pub use neurons::{
    SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest,
    SnsNeuronsRequest,
};
pub use proposals::{
    SnsProposalRequest, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
    SnsProposalsRefreshRequest, SnsProposalsRequest,
};
