use super::super::{SnsHostError, SnsListRequest, SnsLookupRequest, source::SnsFetchRequest};
use super::network::enforce_mainnet_network;
use crate::subnet_catalog::format_utc_timestamp_secs;

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

pub(super) fn fetch_request_from_parts(
    source_endpoint: &str,
    now_unix_secs: u64,
    fetched_by: String,
) -> SnsFetchRequest {
    SnsFetchRequest {
        endpoint: source_endpoint.to_string(),
        fetched_at: format_utc_timestamp_secs(now_unix_secs),
        fetched_by,
    }
}
