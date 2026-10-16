//! Module: sns::report::model::requests::reward
//!
//! Responsibility: request contract for SNS reward checkpoint collection.
//! Does not own: live source calls, strict pagination, or report rendering.
//! Boundary: carries one target, collection start, endpoint, and optional diagnostic page cap.

use super::lookup::SnsLookupRequest;
///
/// SnsRewardCheckpointRequest
///
/// Request accepted by the live SNS reward checkpoint builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsRewardCheckpointRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
    /// Optional diagnostic page cap that may be stricter than the protocol ceiling.
    pub max_pages: Option<u32>,
}

impl SnsRewardCheckpointRequest {
    /// Construct one live SNS reward checkpoint request.
    #[must_use]
    pub const fn new(lookup: SnsLookupRequest) -> Self {
        Self {
            lookup,
            max_pages: None,
        }
    }

    /// Apply an optional diagnostic page cap.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self
    }
}
