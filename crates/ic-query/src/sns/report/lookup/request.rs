//! Module: sns::report::lookup::request
//!
//! Responsibility: build SNS lookup and fetch request values.
//! Does not own: command parsing, source fetching, or report assembly.
//! Boundary: normalizes shared request fields before lookup/source calls.

use crate::sns::report::lookup::network::enforce_mainnet_network;
use crate::sns::report::{
    SnsHostError, SnsListRequest, SnsLookupRequest,
    source::{SnsFetchRequest, SnsSourceRequest},
};
use crate::subnet_catalog::format_utc_timestamp_secs;

/// Build a shared SNS lookup request from command runtime fields.
pub(in crate::sns::report) fn lookup_request_from_parts(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    input: &str,
) -> SnsLookupRequest {
    SnsLookupRequest {
        network: network.to_string(),
        source_endpoint: source_endpoint.to_string(),
        now_unix_secs,
        input: input.to_string(),
    }
}

/// Build a live fetch request for an SNS list command.
pub(in crate::sns::report) fn sns_list_fetch_request(
    request: &SnsListRequest,
) -> Result<SnsFetchRequest, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    Ok(fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    ))
}

/// Build a live fetch request from already-validated source fields.
pub(super) fn fetch_request_from_parts(
    source_endpoint: &str,
    now_unix_secs: u64,
    fetched_by: String,
) -> SnsFetchRequest {
    SnsSourceRequest::new(
        source_endpoint,
        format_utc_timestamp_secs(now_unix_secs),
        fetched_by,
    )
}
