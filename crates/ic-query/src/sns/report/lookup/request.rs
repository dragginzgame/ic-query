//! Module: sns::report::lookup::request
//!
//! Responsibility: build SNS lookup and fetch request values.
//! Does not own: command parsing, source fetching, or report assembly.
//! Boundary: normalizes shared request fields before lookup/source calls.

use crate::report::CollectionSourceRequest;
use crate::sns::report::{
    SNS_REFRESH_MAX_PAGE_SIZE, SnsHostError, SnsListRequest, enforce_mainnet_network,
};
use ic_host_text::timestamp::format_utc_timestamp_secs;

use ic_platform::candid::contract::Principal;

/// Parse and normalize an SNS root canister principal input.
pub(in crate::sns::report) fn parse_sns_root_canister_input(
    input: &str,
) -> Result<String, SnsHostError> {
    Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })
        .map(|principal| principal.to_text())
}

pub(in crate::sns::report) fn validate_sns_refresh_page_size(
    page_size: u32,
) -> Result<(), SnsHostError> {
    if (1..=SNS_REFRESH_MAX_PAGE_SIZE).contains(&page_size) {
        return Ok(());
    }
    Err(SnsHostError::InvalidRefreshPageSize {
        page_size,
        max_page_size: SNS_REFRESH_MAX_PAGE_SIZE,
    })
}

/// Build a live fetch request for an SNS list command.
pub(in crate::sns::report) fn sns_list_fetch_request(
    request: &SnsListRequest,
) -> Result<CollectionSourceRequest, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    Ok(fetch_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    ))
}

/// Build a live fetch request from already-validated source fields.
pub(super) fn fetch_request_from_parts(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    fetched_by: String,
) -> CollectionSourceRequest {
    CollectionSourceRequest::new(
        network,
        source_endpoint,
        format_utc_timestamp_secs(now_unix_secs),
        fetched_by,
    )
}
