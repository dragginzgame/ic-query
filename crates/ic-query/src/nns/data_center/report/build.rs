use super::{
    DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
    NnsDataCenterHostError, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsInventoryInfoRequest, NnsInventoryListRequest,
    cache::{load_cached_nns_data_center_report, nns_data_center_cache_path},
    refresh::refresh_nns_data_center_cache_with_source,
    resolve::resolve_data_center,
    source::NnsDataCenterSource,
};
use crate::nns::{LiveNnsSource, inventory::load_or_refresh_nns_inventory_report};
use crate::{QueryProgress, progress::IgnoreQueryProgress};

pub fn build_nns_data_center_list_report(
    request: &NnsInventoryListRequest,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    build_nns_data_center_list_report_with_source(request, &LiveNnsSource)
}

pub fn build_nns_data_center_info_report(
    request: &NnsInventoryInfoRequest,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    build_nns_data_center_info_report_with_source(request, &LiveNnsSource)
}

/// Build the report while reporting authorized cache refreshes.
pub fn build_nns_data_center_list_report_with_progress(
    request: &NnsInventoryListRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    build_nns_data_center_list_report_with_source_and_progress(request, &LiveNnsSource, progress)
}

pub fn build_nns_data_center_list_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    build_nns_data_center_list_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
}

/// Build the report with a caller-owned source and progress sink.
pub fn build_nns_data_center_list_report_with_source_and_progress(
    request: &NnsInventoryListRequest,
    source: &dyn NnsDataCenterSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    load_or_refresh_nns_inventory_report(
        request,
        nns_data_center_cache_path(&request.cache.cache_root, &request.cache.network),
        DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
        progress,
        |cache| load_cached_nns_data_center_report(cache).map(|cached| cached.report),
        |refresh_request| {
            refresh_nns_data_center_cache_with_source(refresh_request, source).map(|_| ())
        },
    )
}

/// Build the report while reporting authorized cache refreshes.
pub fn build_nns_data_center_info_report_with_progress(
    request: &NnsInventoryInfoRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    build_nns_data_center_info_report_with_source_and_progress(request, &LiveNnsSource, progress)
}

pub fn build_nns_data_center_info_report_with_source(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    build_nns_data_center_info_report_with_source_and_progress(
        request,
        source,
        &mut IgnoreQueryProgress,
    )
}

/// Build the report with a caller-owned source and progress sink.
pub fn build_nns_data_center_info_report_with_source_and_progress(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsDataCenterSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    let list_request = NnsInventoryListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_data_center_list_report_with_source_and_progress(
        &list_request,
        source,
        progress,
    )?;
    let (data_center, resolved_from) = resolve_data_center(&report, &request.input)?;
    Ok(NnsDataCenterInfoReport {
        schema_version: NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        data_center_id: data_center.data_center_id,
        region: data_center.region,
        owner: data_center.owner,
        latitude: data_center.latitude,
        longitude: data_center.longitude,
        node_operator_count: data_center.node_operator_count,
        node_provider_count: data_center.node_provider_count,
        node_count: data_center.node_count,
    })
}
