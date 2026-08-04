//! Module: ic::model::error
//!
//! Responsibility: typed host errors for Dashboard builders and live sources.
//! Does not own: requests, reports, source data, transport, or validation.
//! Boundary: preserves deterministic request, source, endpoint, and transport failures.

use crate::runtime::RuntimeError;
use thiserror::Error as ThisError;

///
/// IcHostError
///
/// Typed error returned by IC Dashboard report builders and live sources.
///

#[derive(Debug, ThisError)]
pub enum IcHostError {
    /// The synchronous adapter could not create its local async runtime.
    #[error("failed to run IC Dashboard query: {0}")]
    Runtime(#[from] RuntimeError),

    /// A request supplied an invalid canister principal.
    #[error("invalid {field}: {reason}")]
    InvalidPrincipal {
        /// Principal field being validated.
        field: &'static str,
        /// Principal parser diagnostic.
        reason: String,
    },

    /// A request violates the bounded Dashboard query contract.
    #[error("invalid {field}: {reason}")]
    InvalidRequest {
        /// Request field being validated.
        field: &'static str,
        /// Deterministic validation diagnostic.
        reason: String,
    },

    /// The Dashboard API base endpoint is malformed or unsupported.
    #[error("invalid IC Dashboard endpoint {endpoint}: {reason}")]
    InvalidEndpoint {
        /// Rejected endpoint.
        endpoint: String,
        /// URL validation diagnostic.
        reason: String,
    },

    /// The HTTP client could not be constructed.
    #[error("failed to build IC Dashboard HTTP client: {reason}")]
    HttpClientBuild {
        /// HTTP client construction diagnostic.
        reason: String,
    },

    /// The live Dashboard request failed before a response was received.
    #[error("IC Dashboard request to {url} failed: {reason}")]
    HttpRequest {
        /// Fully resolved request URL.
        url: String,
        /// HTTP transport error.
        reason: String,
    },

    /// The Dashboard returned a non-success HTTP status.
    #[error("IC Dashboard request to {url} returned HTTP status {status}")]
    HttpStatus {
        /// Fully resolved request URL.
        url: String,
        /// Numeric HTTP status.
        status: u16,
    },

    /// The Dashboard response body could not be read completely.
    #[error("failed to read IC Dashboard response body from {url}: {reason}")]
    HttpResponseBody {
        /// Fully resolved request URL.
        url: String,
        /// HTTP response-body transport error.
        reason: String,
    },

    /// The Dashboard response body exceeded the shared transport ceiling.
    #[error(
        "IC Dashboard response from {url} exceeded the {max_bytes}-byte limit (declared or received size at rejection: {observed_bytes} bytes)"
    )]
    HttpResponseTooLarge {
        /// Fully resolved request URL.
        url: String,
        /// Maximum response bytes accepted by the transport.
        max_bytes: u64,
        /// Declared or consumed bytes observed when the response was rejected.
        observed_bytes: u64,
    },

    /// The Dashboard response did not match the expected JSON shape.
    #[error("failed to decode IC Dashboard response from {url}: {reason}")]
    JsonDecode {
        /// Fully resolved request URL.
        url: String,
        /// JSON response decoding error.
        reason: String,
    },

    /// A source capability returned data that violates its public result contract.
    #[error("invalid IC Dashboard source data: {reason}")]
    InvalidSourceData {
        /// Deterministic invariant failure.
        reason: String,
    },
}
