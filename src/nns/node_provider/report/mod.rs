use crate::ic_registry::{
    DEFAULT_MAINNET_ENDPOINT, MainnetNodeProviderList, MainnetRegistryFetchRequest,
    fetch_mainnet_node_provider_list,
};
use crate::subnet_catalog::canonical_principal_text;
use crate::{
    cache_file::{
        CachedJsonReport, LoadJsonCacheRequest, RefreshCacheWriteRequest, announce_cache_refresh,
        load_json_cache, write_json_refresh_cache,
    },
    nns::leaf::{NnsLeafCachePaths, nns_leaf_cache_path},
    subnet_catalog::format_utc_timestamp_secs,
};
use std::path::{Path, PathBuf};

mod model;
mod text;

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

pub fn load_cached_nns_node_provider_report(
    request: &NnsNodeProviderCacheRequest,
) -> Result<CachedJsonReport<NnsNodeProviderListReport>, NnsNodeProviderHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_node_provider_cache_path(&request.icp_root, &request.network);
    load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsNodeProviderCacheErrors,
    )
}

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

fn refresh_nns_node_provider_report_with_source(
    request: &NnsNodeProviderRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderRefreshReport, NnsNodeProviderHostError> {
    refresh_nns_node_provider_cache_with_source(request, source).map(|(_, report)| report)
}

fn refresh_nns_node_provider_cache_with_source(
    request: &NnsNodeProviderRefreshRequest,
    source: &dyn NnsNodeProviderSource,
) -> Result<(NnsNodeProviderListReport, NnsNodeProviderRefreshReport), NnsNodeProviderHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let paths = NnsLeafCachePaths::for_component(
        &request.cache.icp_root,
        NNS_NODE_PROVIDER_CACHE_DIR,
        &request.cache.network,
        NNS_NODE_PROVIDER_CACHE_FILE,
    );
    let report = fetch_nns_node_provider_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_json_refresh_cache(
        RefreshCacheWriteRequest {
            cache_path: &paths.cache_path,
            lock_path: &paths.lock_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
            dry_run: request.dry_run,
            output_path: request.output_path.as_deref(),
            report: &report,
        },
        node_provider_cache_error,
        |path, source| NnsNodeProviderHostError::SerializeCache { path, source },
    )?;
    let refresh_report = NnsNodeProviderRefreshReport {
        schema_version: NNS_NODE_PROVIDER_REFRESH_REPORT_SCHEMA_VERSION,
        network: report.network.clone(),
        cache_path: write_result.cache_path,
        refresh_lock_path: write_result.refresh_lock_path,
        output_path: write_result.output_path,
        governance_canister_id: report.governance_canister_id.clone(),
        registry_canister_id: report.registry_canister_id.clone(),
        registry_version: report.registry_version,
        fetched_at: report.fetched_at.clone(),
        source_endpoint: report.source_endpoint.clone(),
        fetched_by: report.fetched_by.clone(),
        dry_run: request.dry_run,
        wrote_cache: write_result.wrote_cache,
        replaced_existing_cache: write_result.replaced_existing_cache,
        node_provider_count: report.node_provider_count,
    };
    Ok((report, refresh_report))
}

fn fetch_nns_node_provider_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsNodeProviderSource,
) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
    enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_node_providers(&fetch_request)?;
    Ok(node_provider_report_from_list(list))
}

impl_nns_cache_error_mapper!(node_provider_cache_error, NnsNodeProviderHostError);

fn node_provider_report_from_list(list: MainnetNodeProviderList) -> NnsNodeProviderListReport {
    let node_providers = list
        .node_providers
        .into_iter()
        .map(|provider| NnsNodeProviderRow {
            node_provider_principal: provider.principal,
            name: None,
            node_count: provider.node_count,
            reward_account_hex: provider.reward_account_hex,
        })
        .collect::<Vec<_>>();
    NnsNodeProviderListReport {
        schema_version: NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        governance_canister_id: list.governance_canister_id,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        node_provider_count: node_providers.len(),
        node_providers,
    }
}

///
/// NnsNodeProviderSource
///
trait NnsNodeProviderSource {
    fn fetch_node_providers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeProviderList, NnsNodeProviderHostError>;
}

impl_nns_mainnet_network_enforcer!(NnsNodeProviderHostError);

///
/// LiveNnsNodeProviderSource
///
struct LiveNnsNodeProviderSource;

impl NnsNodeProviderSource for LiveNnsNodeProviderSource {
    fn fetch_node_providers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeProviderList, NnsNodeProviderHostError> {
        Ok(fetch_mainnet_node_provider_list(request)?)
    }
}

fn resolve_node_provider(
    report: &NnsNodeProviderListReport,
    input: &str,
) -> Result<(NnsNodeProviderRow, String), NnsNodeProviderHostError> {
    if let Ok(principal) = canonical_principal_text(input)
        && let Some(provider) = report
            .node_providers
            .iter()
            .find(|provider| provider.node_provider_principal == principal)
    {
        return Ok((provider.clone(), "node_provider_principal".to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsNodeProviderHostError::NodeProviderNotFound {
            input: input.to_string(),
        });
    }
    let matches = report
        .node_providers
        .iter()
        .filter(|provider| provider.node_provider_principal.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [provider] => Ok((
            provider.clone(),
            "node_provider_principal_prefix".to_string(),
        )),
        [] => Err(NnsNodeProviderHostError::NodeProviderNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsNodeProviderHostError::AmbiguousNodeProviderPrefix {
            prefix,
            matches: matches
                .into_iter()
                .map(|provider| provider.node_provider_principal)
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests;
