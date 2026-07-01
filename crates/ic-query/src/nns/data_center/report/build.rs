use super::{
    DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION,
    NnsDataCenterHostError, NnsDataCenterInfoReport, NnsDataCenterInfoRequest,
    NnsDataCenterListReport, NnsDataCenterListRequest, NnsDataCenterRefreshRequest,
    cache::load_cached_nns_data_center_report,
    refresh::refresh_nns_data_center_cache_with_source,
    resolve::resolve_data_center,
    source::{LiveNnsDataCenterSource, NnsDataCenterSource},
};
use crate::{cache_file::load_or_refresh_missing_cache, nns::leaf::NnsLeafHostCacheError};

pub fn build_nns_data_center_list_report(
    request: &NnsDataCenterListRequest,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    build_nns_data_center_list_report_with_source(request, &LiveNnsDataCenterSource)
}

pub fn build_nns_data_center_info_report(
    request: &NnsDataCenterInfoRequest,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    build_nns_data_center_info_report_with_source(request, &LiveNnsDataCenterSource)
}

pub fn build_nns_data_center_list_report_with_source(
    request: &NnsDataCenterListRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    load_or_refresh_missing_cache(
        "data-center",
        &request.source_endpoint,
        || load_cached_nns_data_center_report(&request.cache).map(|cached| cached.report),
        || {
            let refresh_request = NnsDataCenterRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            refresh_nns_data_center_cache_with_source(&refresh_request, source).map(|_| ())
        },
        |err| match err {
            NnsDataCenterHostError::Cache(NnsLeafHostCacheError::MissingCache { path, .. }) => {
                Ok(path)
            }
            err => Err(err),
        },
    )
}

pub fn build_nns_data_center_info_report_with_source(
    request: &NnsDataCenterInfoRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterInfoReport, NnsDataCenterHostError> {
    let list_request = NnsDataCenterListRequest {
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
