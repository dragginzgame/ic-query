use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::{cache_file::announce_cache_refresh, nns::leaf::nns_leaf_cache_path};
use std::path::{Path, PathBuf};

mod cache;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

use cache::load_cached_nns_node_operator_report;
use refresh::{
    refresh_nns_node_operator_cache_with_source, refresh_nns_node_operator_report_with_source,
};
use resolve::resolve_node_operator;
use source::{LiveNnsNodeOperatorSource, NnsNodeOperatorSource};

#[cfg(test)]
use crate::ic_registry::{MainnetNodeOperatorList, MainnetRegistryFetchRequest};

pub use model::*;
pub use text::{
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text, nns_node_operator_refresh_report_text,
};

pub const DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NODE_OPERATOR_CACHE_DIR: &str = "node-operator";
const NNS_NODE_OPERATOR_CACHE_FILE: &str = "operators.json";

#[must_use]
pub fn nns_node_operator_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(
        icp_root,
        NNS_NODE_OPERATOR_CACHE_DIR,
        network,
        NNS_NODE_OPERATOR_CACHE_FILE,
    )
}

impl_nns_load_json_cache_error_mapper!(NnsNodeOperatorCacheErrors, NnsNodeOperatorHostError);
impl_nns_cache_error_mapper!(node_operator_cache_error, NnsNodeOperatorHostError);
impl_nns_mainnet_network_enforcer!(NnsNodeOperatorHostError);

pub fn build_nns_node_operator_list_report(
    request: &NnsNodeOperatorListRequest,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_list_report_with_source(request, &LiveNnsNodeOperatorSource)
}

pub fn build_nns_node_operator_info_report(
    request: &NnsNodeOperatorInfoRequest,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    build_nns_node_operator_info_report_with_source(request, &LiveNnsNodeOperatorSource)
}

pub fn refresh_nns_node_operator_report(
    request: &NnsNodeOperatorRefreshRequest,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_report_with_source(request, &LiveNnsNodeOperatorSource)
}

fn build_nns_node_operator_list_report_with_source(
    request: &NnsNodeOperatorListRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    match load_cached_nns_node_operator_report(&request.cache) {
        Ok(cached) => Ok(cached.report),
        Err(NnsNodeOperatorHostError::MissingCache { path }) => {
            announce_cache_refresh("node-operator", &path, &request.source_endpoint);
            let refresh_request = NnsNodeOperatorRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            let (report, _) =
                refresh_nns_node_operator_cache_with_source(&refresh_request, source)?;
            Ok(report)
        }
        Err(err) => Err(err),
    }
}

fn build_nns_node_operator_info_report_with_source(
    request: &NnsNodeOperatorInfoRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorInfoReport, NnsNodeOperatorHostError> {
    let list_request = NnsNodeOperatorListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_node_operator_list_report_with_source(&list_request, source)?;
    let (operator, resolved_from) = resolve_node_operator(&report, &request.input)?;
    Ok(NnsNodeOperatorInfoReport {
        schema_version: NNS_NODE_OPERATOR_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        node_operator_principal: operator.node_operator_principal,
        node_provider_principal: operator.node_provider_principal,
        node_allowance: operator.node_allowance,
        data_center_id: operator.data_center_id,
        node_count: operator.node_count,
    })
}

#[cfg(test)]
mod tests;
