//! Module: sns::report::model::requests::lookup
//!
//! Responsibility: shared request DTO for direct SNS lookup reports.
//! Does not own: command option parsing, source resolution, or rendering.
//! Boundary: carries validated lookup inputs into one SNS report builder.

///
/// SnsLookupRequest
///
/// Shared request accepted by direct SNS info, token, and params builders.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsLookupRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
}

/// Request accepted by the SNS info report builder.
pub type SnsInfoRequest = SnsLookupRequest;

/// Request accepted by the SNS governance-parameters report builder.
pub type SnsParamsRequest = SnsLookupRequest;

/// Request accepted by the SNS token report builder.
pub type SnsTokenRequest = SnsLookupRequest;
