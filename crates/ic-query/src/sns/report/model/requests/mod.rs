//! Module: sns::report::model::requests
//!
//! Responsibility: group SNS report request DTOs.
//! Does not own: CLI parsing, live source calls, cache IO, or rendering.
//! Boundary: re-exports request contracts accepted by SNS report builders.

#[cfg(feature = "host")]
mod cache;
mod list;
mod lookup;
mod metrics;
#[cfg(feature = "host")]
mod neurons;
mod proposals;
#[cfg(feature = "host")]
mod reward;

#[cfg(feature = "host")]
pub use cache::{SnsCacheListRequest, SnsCacheStatusRequest};
pub use list::SnsListRequest;
pub use lookup::SnsLookupRequest;
pub use metrics::{
    DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, MAX_SNS_METRICS_TIME_WINDOW_SECONDS, SnsMetricsRequest,
};
#[cfg(feature = "host")]
pub(in crate::sns::report) use metrics::{
    sns_metrics_lookup_request, validate_sns_metrics_request, validate_sns_metrics_time_window,
};
#[cfg(feature = "host")]
pub use neurons::{SnsNeuronRequest, SnsNeuronsRefreshRequest, SnsNeuronsRequest};
#[cfg(feature = "host")]
pub use proposals::SnsProposalsRefreshRequest;
pub use proposals::{SnsProposalRequest, SnsProposalsRequest};
#[cfg(feature = "host")]
pub use reward::SnsRewardCheckpointRequest;
