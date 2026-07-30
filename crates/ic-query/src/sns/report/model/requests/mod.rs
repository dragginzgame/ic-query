//! Module: sns::report::model::requests
//!
//! Responsibility: group SNS report request DTOs.
//! Does not own: CLI parsing, live source calls, cache IO, or rendering.
//! Boundary: re-exports request contracts accepted by SNS report builders.

#[cfg(feature = "host")]
mod cache;
mod list;
mod lookup;
#[cfg(feature = "host")]
mod neurons;
mod proposals;

#[cfg(feature = "host")]
pub use cache::{SnsCacheListRequest, SnsCacheStatusRequest};
pub use list::SnsListRequest;
pub use lookup::SnsLookupRequest;
#[cfg(feature = "host")]
pub use neurons::{SnsNeuronsRefreshRequest, SnsNeuronsRequest};
#[cfg(feature = "host")]
pub use proposals::SnsProposalsRefreshRequest;
pub use proposals::{SnsProposalRequest, SnsProposalsRequest};
