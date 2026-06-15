use crate::ic_registry::DEFAULT_MAINNET_ENDPOINT;
use crate::{cache_file::announce_cache_refresh, nns::leaf::nns_leaf_cache_path};
use std::path::{Path, PathBuf};

mod cache;
mod model;
mod refresh;
mod resolve;
mod source;
mod text;

use cache::load_cached_nns_node_provider_report;
use refresh::{
    refresh_nns_node_provider_cache_with_source, refresh_nns_node_provider_report_with_source,
};
use resolve::resolve_node_provider;
use source::{LiveNnsNodeProviderSource, NnsNodeProviderSource};

#[cfg(test)]
use crate::ic_registry::{MainnetNodeProviderList, MainnetRegistryFetchRequest};

pub use model::*;
pub use text::{
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text, nns_node_provider_refresh_report_text,
};

pub const DEFAULT_NNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_NODE_PROVIDER_CACHE_DIR: &str = "node-provider";
const NNS_NODE_PROVIDER_CACHE_FILE: &str = "providers.json";

#[must_use]
pub fn nns_node_provider_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    nns_leaf_cache_path(
        icp_root,
        NNS_NODE_PROVIDER_CACHE_DIR,
        network,
        NNS_NODE_PROVIDER_CACHE_FILE,
    )
}

impl_nns_load_json_cache_error_mapper!(NnsNodeProviderCacheErrors, NnsNodeProviderHostError);
impl_nns_cache_error_mapper!(node_provider_cache_error, NnsNodeProviderHostError);
impl_nns_mainnet_network_enforcer!(NnsNodeProviderHostError);

pub fn build_nns_node_provider_list_report(
    request: &NnsNodeProviderListRequest,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    build_nns_node_provider_list_report_with_source(request, &LiveNnsNodeProviderSource)
}

pub fn build_nns_node_provider_info_report(
    request: &NnsNodeProviderInfoRequest,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    build_nns_node_provider_info_report_with_source(request, &LiveNnsNodeProviderSource)
}

pub fn refresh_nns_node_provider_report(
    request: &NnsNodeProviderRefreshRequest,
) -> Result<NnsNodeProviderRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_report_with_source(request, &LiveNnsNodeProviderSource)
}

fn build_nns_node_provider_list_report_with_source(
    request: &NnsNodeProviderListRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    match load_cached_nns_node_provider_report(&request.cache) {
        Ok(cached) => Ok(cached.report),
        Err(NnsNodeProviderHostError::MissingCache { path }) => {
            announce_cache_refresh("node-provider", &path, &request.source_endpoint);
            let refresh_request = NnsNodeProviderRefreshRequest {
                cache: request.cache.clone(),
                source_endpoint: request.source_endpoint.clone(),
                now_unix_secs: request.now_unix_secs,
                lock_stale_after_seconds: DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
                dry_run: false,
                output_path: None,
            };
            let (report, _) =
                refresh_nns_node_provider_cache_with_source(&refresh_request, source)?;
            Ok(report)
        }
        Err(err) => Err(err),
    }
}

fn build_nns_node_provider_info_report_with_source(
    request: &NnsNodeProviderInfoRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderInfoReport, NnsNodeProviderHostError> {
    let list_request = NnsNodeProviderListRequest {
        cache: request.cache.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
    };
    let report = build_nns_node_provider_list_report_with_source(&list_request, source)?;
    let (provider, resolved_from) = resolve_node_provider(&report, &request.input)?;
    Ok(NnsNodeProviderInfoReport {
        schema_version: NNS_NODE_PROVIDER_INFO_REPORT_SCHEMA_VERSION,
        input: request.input.clone(),
        resolved_from,
        network: report.network,
        governance_canister_id: report.governance_canister_id,
        registry_canister_id: report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        node_provider_principal: provider.node_provider_principal,
        name: provider.name,
        node_count: provider.node_count,
        reward_account_hex: provider.reward_account_hex,
    })
}

#[cfg(test)]
mod tests;
