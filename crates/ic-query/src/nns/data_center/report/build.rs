use super::{
    DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
    NnsDataCenterHostError, NnsDataCenterInfoReport, NnsDataCenterListReport,
    NnsInventoryInfoRequest, NnsInventoryListRequest, NnsInventoryRefreshRequest,
    cache::load_cached_nns_data_center_report, refresh::refresh_nns_data_center_cache_with_source,
    resolve::resolve_data_center, source::NnsDataCenterSource,
};
use crate::{HostCacheError, cache_file::load_or_refresh_missing_cache, nns::LiveNnsSource};

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

pub fn build_nns_data_center_list_report_with_source(
    request: &NnsInventoryListRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    load_or_refresh_missing_cache(
        || load_cached_nns_data_center_report(&request.cache).map(|cached| cached.report),
        |err| match err {
            NnsDataCenterHostError::Cache(HostCacheError::MissingCache { path, .. }) => Ok(path),
            err => Err(err),
        },
        |_| {
            let refresh_request = NnsInventoryRefreshRequest::new(
                request.cache.clone(),
                request.source_endpoint.clone(),
                request.now_unix_secs,
                DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
            );
            refresh_nns_data_center_cache_with_source(&refresh_request, source).map(|_| ())
        },
    )
}

pub fn build_nns_data_center_info_report_with_source(
    request: &NnsInventoryInfoRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    let list_request = NnsInventoryListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_data_center_list_report_with_source(&list_request, source)?;
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
