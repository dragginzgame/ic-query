use crate::ic_registry::{
    DEFAULT_MAINNET_ENDPOINT, MainnetNodeList, MainnetRegistryFetchRequest, fetch_mainnet_node_list,
};
use crate::subnet_catalog::{MAINNET_NETWORK, canonical_principal_text};
use crate::{
    cache_file::{
        LoadJsonCacheErrorMapper, LoadJsonCacheRequest, RefreshCacheWriteRequest,
        announce_cache_refresh, load_json_cache, write_json_refresh_cache,
    },
    subnet_catalog::format_utc_timestamp_secs,
};
use std::path::{Path, PathBuf};

mod model;
mod text;

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

#[must_use]
pub fn nns_node_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("node")
        .join(network)
        .join("nodes.json")
}

#[must_use]
pub fn nns_node_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("node")
        .join(network)
        .join("refresh.lock")
}

struct NnsNodeCacheErrors;

impl LoadJsonCacheErrorMapper for NnsNodeCacheErrors {
    type Error = NnsNodeHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        NnsNodeHostError::MissingCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: std::io::Error) -> Self::Error {
        NnsNodeHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        NnsNodeHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        NnsNodeHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        NnsNodeHostError::NetworkMismatch { requested, actual }
    }
}

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

fn load_cached_nns_node_report(
    request: &NnsNodeCacheRequest,
) -> Result<CachedNnsNodeReport, NnsNodeHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_node_cache_path(&request.icp_root, &request.network);
    let cached = load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsNodeCacheErrors,
    )?;
    Ok(CachedNnsNodeReport {
        path: cached.path,
        report: cached.report,
    })
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

fn refresh_nns_node_report_with_source(
    request: &NnsNodeRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeRefreshReport, NnsNodeHostError> {
    refresh_nns_node_cache_with_source(request, source).map(|(_, report)| report)
}

fn refresh_nns_node_cache_with_source(
    request: &NnsNodeRefreshRequest,
    source: &dyn NnsNodeSource,
) -> Result<(NnsNodeListReport, NnsNodeRefreshReport), NnsNodeHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let cache_path = nns_node_cache_path(&request.cache.icp_root, &request.cache.network);
    let lock_path = nns_node_refresh_lock_path(&request.cache.icp_root, &request.cache.network);
    let report = fetch_nns_node_list_report_with_source(
        &request.cache.network,
        &request.source_endpoint,
        request.now_unix_secs,
        source,
    )?;
    let write_result = write_json_refresh_cache(
        RefreshCacheWriteRequest {
            cache_path: &cache_path,
            lock_path: &lock_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
            dry_run: request.dry_run,
            output_path: request.output_path.as_deref(),
            report: &report,
        },
        node_cache_error,
        |path, source| NnsNodeHostError::SerializeCache { path, source },
    )?;
    let refresh_report = NnsNodeRefreshReport {
        schema_version: NNS_NODE_REFRESH_REPORT_SCHEMA_VERSION,
        network: report.network.clone(),
        cache_path: write_result.cache_path,
        refresh_lock_path: write_result.refresh_lock_path,
        output_path: write_result.output_path,
        registry_canister_id: report.registry_canister_id.clone(),
        registry_version: report.registry_version,
        fetched_at: report.fetched_at.clone(),
        source_endpoint: report.source_endpoint.clone(),
        fetched_by: report.fetched_by.clone(),
        dry_run: request.dry_run,
        wrote_cache: write_result.wrote_cache,
        replaced_existing_cache: write_result.replaced_existing_cache,
        node_count: report.node_count,
    };
    Ok((report, refresh_report))
}

fn fetch_nns_node_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsNodeSource,
) -> Result<NnsNodeListReport, NnsNodeHostError> {
    enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_nodes(&fetch_request)?;
    Ok(node_report_from_list(list))
}

impl_nns_cache_error_mapper!(node_cache_error, NnsNodeHostError);

fn node_report_from_list(list: MainnetNodeList) -> NnsNodeListReport {
    let nodes = list
        .nodes
        .into_iter()
        .map(|node| NnsNodeRow {
            node_principal: node.principal,
            node_operator_principal: node.node_operator_principal,
            node_provider_principal: node.node_provider_principal,
            subnet_principal: node.subnet_principal,
            subnet_kind: node.subnet_kind,
            data_center_id: node.data_center_id,
        })
        .collect::<Vec<_>>();
    NnsNodeListReport {
        schema_version: NNS_NODE_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        node_count: nodes.len(),
        nodes,
    }
}

fn filter_node_list_report(
    mut report: NnsNodeListReport,
    filters: &NnsNodeListFilters,
) -> NnsNodeListReport {
    if filters.is_empty() {
        return report;
    }
    report
        .nodes
        .retain(|node| node_matches_filters(node, filters));
    report.node_count = report.nodes.len();
    report
}

fn node_matches_filters(node: &NnsNodeRow, filters: &NnsNodeListFilters) -> bool {
    filters
        .subnet
        .as_deref()
        .is_none_or(|filter| principal_filter_matches(&node.subnet_principal, filter))
        && filters
            .subnet_kind
            .as_deref()
            .is_none_or(|filter| text_filter_equals(&node.subnet_kind, filter))
        && filters
            .data_center
            .as_deref()
            .is_none_or(|filter| text_filter_starts_with(&node.data_center_id, filter))
        && filters
            .node_provider
            .as_deref()
            .is_none_or(|filter| principal_filter_matches(&node.node_provider_principal, filter))
        && filters
            .node_operator
            .as_deref()
            .is_none_or(|filter| principal_filter_matches(&node.node_operator_principal, filter))
}

fn principal_filter_matches(value: &str, filter: &str) -> bool {
    let Some(filter) = non_empty_filter(filter) else {
        return false;
    };
    if let Ok(principal) = canonical_principal_text(filter) {
        value == principal
    } else {
        value.starts_with(&filter.to_ascii_lowercase())
    }
}

fn text_filter_starts_with(value: &str, filter: &str) -> bool {
    let Some(filter) = non_empty_filter(filter) else {
        return false;
    };
    value
        .to_ascii_lowercase()
        .starts_with(&filter.to_ascii_lowercase())
}

fn text_filter_equals(value: &str, filter: &str) -> bool {
    let Some(filter) = non_empty_filter(filter) else {
        return false;
    };
    value.eq_ignore_ascii_case(filter)
}

fn non_empty_filter(filter: &str) -> Option<&str> {
    let filter = filter.trim();
    (!filter.is_empty()).then_some(filter)
}

///
/// NnsNodeSource
///
trait NnsNodeSource {
    fn fetch_nodes(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeList, NnsNodeHostError>;
}

///
/// LiveNnsNodeSource
///
struct LiveNnsNodeSource;

impl NnsNodeSource for LiveNnsNodeSource {
    fn fetch_nodes(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeList, NnsNodeHostError> {
        Ok(fetch_mainnet_node_list(request)?)
    }
}

fn enforce_mainnet_network(network: &str) -> Result<(), NnsNodeHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(NnsNodeHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

fn resolve_node(
    report: &NnsNodeListReport,
    input: &str,
) -> Result<(NnsNodeRow, String), NnsNodeHostError> {
    if let Ok(principal) = canonical_principal_text(input)
        && let Some(node) = report
            .nodes
            .iter()
            .find(|node| node.node_principal == principal)
    {
        return Ok((node.clone(), "node_principal".to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsNodeHostError::NodeNotFound {
            input: input.to_string(),
        });
    }
    let matches = report
        .nodes
        .iter()
        .filter(|node| node.node_principal.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node] => Ok((node.clone(), "node_principal_prefix".to_string())),
        [] => Err(NnsNodeHostError::NodeNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsNodeHostError::AmbiguousNodePrefix {
            prefix,
            matches: matches
                .into_iter()
                .map(|node| node.node_principal)
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests;
