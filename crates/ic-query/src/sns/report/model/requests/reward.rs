//! Module: sns::report::model::requests::reward
//!
//! Responsibility: request contract for SNS reward checkpoint collection.
//! Does not own: live source calls, strict pagination, or report rendering.
//! Boundary: carries one target, collection start, endpoint, and optional diagnostic page cap.

///
/// SnsRewardCheckpointRequest
///
/// Request accepted by the live SNS reward checkpoint builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsRewardCheckpointRequest {
    /// Requested IC network identity.
    pub network: String,
    /// Explicit IC API endpoint used for live calls.
    pub source_endpoint: String,
    /// Collection start timestamp supplied immediately before dispatch.
    pub now_unix_secs: u64,
    /// SNS list id or Root canister principal.
    pub input: String,
    /// Optional diagnostic page cap that may be stricter than the protocol ceiling.
    pub max_pages: Option<u32>,
}

impl SnsRewardCheckpointRequest {
    /// Construct one live SNS reward checkpoint request.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input: input.into(),
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
