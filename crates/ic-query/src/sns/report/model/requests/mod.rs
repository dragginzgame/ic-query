//! Module: sns::report::model::requests
//!
//! Responsibility: group SNS report request DTOs.
//! Does not own: CLI parsing, live source calls, cache IO, or rendering.
//! Boundary: re-exports request contracts accepted by SNS report builders.

mod list;
#[cfg(feature = "host")]
mod lookup;
#[cfg(feature = "host")]
mod neurons;
#[cfg(feature = "host")]
mod proposals;

pub use list::SnsListRequest;
#[cfg(feature = "host")]
pub use lookup::{SnsInfoRequest, SnsLookupRequest, SnsParamsRequest, SnsTokenRequest};
#[cfg(feature = "host")]
pub use neurons::{
    SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest, SnsNeuronsRefreshRequest,
    SnsNeuronsRequest,
};
#[cfg(feature = "host")]
pub use proposals::{
    SnsProposalRequest, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
    SnsProposalsRefreshRequest, SnsProposalsRequest,
};
