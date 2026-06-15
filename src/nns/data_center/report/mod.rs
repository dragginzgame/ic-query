use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::{cache_file::announce_cache_refresh, nns::leaf::nns_leaf_cache_path};
use std::path::{Path, PathBuf};

mod cache;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

use cache::load_cached_nns_data_center_report;
use refresh::{
    refresh_nns_data_center_cache_with_source, refresh_nns_data_center_report_with_source,
};
use resolve::resolve_data_center;
use source::{LiveNnsDataCenterSource, NnsDataCenterSource};

#[cfg(test)]
use crate::ic_registry::{MainnetDataCenterList, MainnetRegistryFetchRequest};

pub use model::*;
pub use text::{
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text, nns_data_center_refresh_report_text,
};

pub const DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_DATA_CENTER_CACHE_DIR: &str = "data-center";
const NNS_DATA_CENTER_CACHE_FILE: &str = "data-centers.json";

#[must_use]
pub fn nns_data_center_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(
        icp_root,
        NNS_DATA_CENTER_CACHE_DIR,
        network,
        NNS_DATA_CENTER_CACHE_FILE,
    )
}

impl_nns_load_json_cache_error_mapper!(NnsDataCenterCacheErrors, NnsDataCenterHostError);
impl_nns_cache_error_mapper!(data_center_cache_error, NnsDataCenterHostError);
impl_nns_mainnet_network_enforcer!(NnsDataCenterHostError);

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

pub fn refresh_nns_data_center_report(
    request: &NnsDataCenterRefreshRequest,
) -> Result<NnsDataCenterRefreshReport, NnsDataCenterHostError> {
    refresh_nns_data_center_report_with_source(request, &LiveNnsDataCenterSource)
}

fn build_nns_data_center_list_report_with_source(
    request: &NnsDataCenterListRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    match load_cached_nns_data_center_report(&request.cache) {
        Ok(cached) => Ok(cached.report),
        Err(NnsDataCenterHostError::MissingCache { path }) => {
            announce_cache_refresh("data-center", &path, &request.source_endpoint);
            let refresh_request = NnsDataCenterRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            let (report, _) = refresh_nns_data_center_cache_with_source(&refresh_request, source)?;
            Ok(report)
        }
        Err(err) => Err(err),
    }
}

fn build_nns_data_center_info_report_with_source(
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

#[cfg(test)]
mod tests;
