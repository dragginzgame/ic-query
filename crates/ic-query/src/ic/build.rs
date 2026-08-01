//! Module: ic::build
//!
//! Responsibility: build IC Dashboard reports through focused source capabilities.
//! Does not own: HTTP transport, source result validation, command parsing, or rendering.
//! Boundary: validates request identity before any live source call.

use crate::{
    ic::{
        IcBoundaryNodeDataCentersReport, IcBoundaryNodeDataCentersRequest,
        IcCanisterCollectionSource, IcCanisterCountReport, IcCanisterCountRequest,
        IcCanisterPageReport, IcCanisterPageRequest, IcCanisterReport, IcCanisterRequest,
        IcCanisterSource, IcHostError, IcMetricReport, IcMetricRequest, IcMetricSource,
        IcNetworkSource, IcSourceRequest, LiveIcSource,
        source::{
            boundary_node_data_centers_report_from_source, canonical_canister_id,
            canonical_page_cursor, count_report_from_source, metric_report_from_source,
            normalized_filters, page_report_from_source, report_from_source,
            validate_metric_request, validate_page_limit,
        },
    },
    subnet_catalog::format_utc_timestamp_secs,
};

/// Build one live boundary-node data-center report from the official Dashboard API.
pub fn build_ic_boundary_node_data_centers_report(
    request: &IcBoundaryNodeDataCentersRequest,
) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> {
    build_ic_boundary_node_data_centers_report_with_source(request, &LiveIcSource)
}

/// Build one boundary-node data-center report through a custom Dashboard source.
pub fn build_ic_boundary_node_data_centers_report_with_source(
    request: &IcBoundaryNodeDataCentersRequest,
    source: &dyn IcNetworkSource,
) -> Result<IcBoundaryNodeDataCentersReport, IcHostError> {
    let source_request = source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_boundary_node_data_centers(&source_request)?;
    boundary_node_data_centers_report_from_source(&source_request, source_data)
}

/// Build one live, bounded metric report from the official Dashboard Metrics API.
pub fn build_ic_metric_report(request: &IcMetricRequest) -> Result<IcMetricReport, IcHostError> {
    build_ic_metric_report_with_source(request, &LiveIcSource)
}

/// Build one bounded metric report through a custom Dashboard source capability.
pub fn build_ic_metric_report_with_source(
    request: &IcMetricRequest,
    source: &dyn IcMetricSource,
) -> Result<IcMetricReport, IcHostError> {
    validate_metric_request(request.now_unix_secs, &request.query)?;
    let source_request = source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_metric(&source_request, &request.query)?;
    metric_report_from_source(&source_request, &request.query, source_data)
}

/// Build one live canister report from the official IC Dashboard API.
pub fn build_ic_canister_report(
    request: &IcCanisterRequest,
) -> Result<IcCanisterReport, IcHostError> {
    build_ic_canister_report_with_source(request, &LiveIcSource)
}

/// Build one canister report through a custom Dashboard source capability.
pub fn build_ic_canister_report_with_source(
    request: &IcCanisterRequest,
    source: &dyn IcCanisterSource,
) -> Result<IcCanisterReport, IcHostError> {
    let canister_id = canonical_canister_id(&request.canister_id)?;
    let source_request = source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_canister(&source_request, &canister_id)?;
    report_from_source(&source_request, &canister_id, source_data)
}

/// Build one live filtered canister count from the official IC Dashboard API.
pub fn build_ic_canister_count_report(
    request: &IcCanisterCountRequest,
) -> Result<IcCanisterCountReport, IcHostError> {
    build_ic_canister_count_report_with_source(request, &LiveIcSource)
}

/// Build one filtered canister count through a custom Dashboard source capability.
pub fn build_ic_canister_count_report_with_source(
    request: &IcCanisterCountRequest,
    source: &dyn IcCanisterCollectionSource,
) -> Result<IcCanisterCountReport, IcHostError> {
    let filters = normalized_filters(&request.filters)?;
    let source_request = source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_canister_count(&source_request, &filters)?;
    count_report_from_source(&source_request, &filters, source_data)
}

/// Build one live, bounded canister page from the official IC Dashboard API.
pub fn build_ic_canister_page_report(
    request: &IcCanisterPageRequest,
) -> Result<IcCanisterPageReport, IcHostError> {
    build_ic_canister_page_report_with_source(request, &LiveIcSource)
}

/// Build one bounded canister page through a custom Dashboard source capability.
pub fn build_ic_canister_page_report_with_source(
    request: &IcCanisterPageRequest,
    source: &dyn IcCanisterCollectionSource,
) -> Result<IcCanisterPageReport, IcHostError> {
    validate_page_limit(request.limit)?;
    if request.after.is_some() && request.before.is_some() {
        return Err(IcHostError::InvalidRequest {
            field: "pagination",
            reason: "after and before are mutually exclusive".to_string(),
        });
    }

    let filters = normalized_filters(&request.filters)?;
    let after = canonical_page_cursor("after", request.after.as_deref())?;
    let before = canonical_page_cursor("before", request.before.as_deref())?;
    let source_request = source_request(&request.source_endpoint, request.now_unix_secs);
    let source_data = source.fetch_canister_page(
        &source_request,
        &filters,
        request.limit,
        after.as_deref(),
        before.as_deref(),
    )?;
    page_report_from_source(
        &source_request,
        &filters,
        request.limit,
        after.as_deref(),
        before.as_deref(),
        source_data,
    )
}

fn source_request(source_endpoint: &str, now_unix_secs: u64) -> IcSourceRequest {
    IcSourceRequest::new(
        source_endpoint,
        format_utc_timestamp_secs(now_unix_secs),
        "ic-query",
    )
}
