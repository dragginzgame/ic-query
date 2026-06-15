use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::{cache_file::announce_cache_refresh, nns::leaf::nns_leaf_cache_path};
use std::path::{Path, PathBuf};

mod cache;
mod filters;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

use cache::load_cached_nns_node_report;
use filters::filter_node_list_report;
use refresh::{refresh_nns_node_cache_with_source, refresh_nns_node_report_with_source};
use resolve::resolve_node;
use source::{LiveNnsNodeSource, NnsNodeSource};

#[cfg(test)]
use crate::ic_registry::{MainnetNodeList, MainnetRegistryFetchRequest};

pub use model::*;
pub use text::{
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
    nns_node_refresh_report_text,
};

pub const DEFAULT_NNS_NODE_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_SUBNET_KIND_APPLICATION: &str = "application";
pub const NNS_NODE_SUBNET_KIND_CLOUD_ENGINE: &str = "cloud_engine";
pub const NNS_NODE_SUBNET_KIND_SYSTEM: &str = "system";
pub const NNS_NODE_SUBNET_KIND_UNKNOWN: &str = "unknown";
const NNS_NODE_CACHE_DIR: &str = "node";
const NNS_NODE_CACHE_FILE: &str = "nodes.json";

#[must_use]
pub fn nns_node_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(icp_root, NNS_NODE_CACHE_DIR, network, NNS_NODE_CACHE_FILE)
}

impl_nns_load_json_cache_error_mapper!(NnsNodeCacheErrors, NnsNodeHostError);
impl_nns_cache_error_mapper!(node_cache_error, NnsNodeHostError);
impl_nns_mainnet_network_enforcer!(NnsNodeHostError);

pub fn build_nns_node_list_report(
    request: &NnsNodeListRequest,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    build_nns_node_list_report_with_source(request, &LiveNnsNodeSource)
}

pub fn build_nns_node_info_report(
    request: &NnsNodeInfoRequest,
) -> Result<NnsNodeInfoReport, NnsNodeHostError> {
    build_nns_node_info_report_with_source(request, &LiveNnsNodeSource)
}

pub fn refresh_nns_node_report(
    request: &NnsNodeRefreshRequest,
) -> Result<NnsNodeRefreshReport, NnsNodeHostError> {
    refresh_nns_node_report_with_source(request, &LiveNnsNodeSource)
}

fn build_nns_node_list_report_with_source(
    request: &NnsNodeListRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    let report = match load_cached_nns_node_report(&request.cache) {
        Ok(cached) => cached.report,
        Err(NnsNodeHostError::MissingCache { path }) => {
            announce_cache_refresh("node", &path, &request.source_endpoint);
            let refresh_request = NnsNodeRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            let (report, _) = refresh_nns_node_cache_with_source(&refresh_request, source)?;
            report
        }
        Err(err) => return Err(err),
    };
    Ok(filter_node_list_report(report, &request.filters))
}

fn build_nns_node_info_report_with_source(
    request: &NnsNodeInfoRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeInfoReport, NnsNodeHostError> {
    let list_request = NnsNodeListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
        filters: NnsNodeListFilters::default(),
    };
    let report = build_nns_node_list_report_with_source(&list_request, source)?;
    let (node, resolved_from) = resolve_node(&report, &request.input)?;
    Ok(NnsNodeInfoReport {
        schema_version: NNS_NODE_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        node_principal: node.node_principal,
        node_operator_principal: node.node_operator_principal,
        node_provider_principal: node.node_provider_principal,
        subnet_principal: node.subnet_principal,
        subnet_kind: node.subnet_kind,
        data_center_id: node.data_center_id,
    })
}

#[cfg(test)]
mod tests;
