use crate::ic_registry::{
    DEFAULT_MAINNET_ENDPOINT, MainnetNodeOperatorList, MainnetRegistryFetchRequest,
    fetch_mainnet_node_operator_list,
};
use crate::subnet_catalog::{MAINNET_NETWORK, canonical_principal_text};
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

pub fn load_cached_nns_node_operator_report(
    request: &NnsNodeOperatorCacheRequest,
) -> Result<CachedJsonReport<NnsNodeOperatorListReport>, NnsNodeOperatorHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_node_operator_cache_path(&request.icp_root, &request.network);
    load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsNodeOperatorCacheErrors,
    )
}

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

fn refresh_nns_node_operator_report_with_source(
    request: &NnsNodeOperatorRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorRefreshReport, NnsNodeOperatorHostError> {
    refresh_nns_node_operator_cache_with_source(request, source).map(|(_, report)| report)
}

fn refresh_nns_node_operator_cache_with_source(
    request: &NnsNodeOperatorRefreshRequest,
    source: &dyn NnsNodeOperatorSource,
) -> Result<(NnsNodeOperatorListReport, NnsNodeOperatorRefreshReport), NnsNodeOperatorHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let paths = NnsLeafCachePaths::for_component(
        &request.cache.icp_root,
        NNS_NODE_OPERATOR_CACHE_DIR,
        &request.cache.network,
        NNS_NODE_OPERATOR_CACHE_FILE,
    );
    let report = fetch_nns_node_operator_list_report_with_source(
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
        node_operator_cache_error,
        |path, source| NnsNodeOperatorHostError::SerializeCache { path, source },
    )?;
    let refresh_report = NnsNodeOperatorRefreshReport {
        schema_version: NNS_NODE_OPERATOR_REFRESH_REPORT_SCHEMA_VERSION,
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
        node_operator_count: report.node_operator_count,
    };
    Ok((report, refresh_report))
}

fn fetch_nns_node_operator_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsNodeOperatorSource,
) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
    enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_node_operators(&fetch_request)?;
    Ok(node_operator_report_from_list(list))
}

impl_nns_cache_error_mapper!(node_operator_cache_error, NnsNodeOperatorHostError);

fn node_operator_report_from_list(list: MainnetNodeOperatorList) -> NnsNodeOperatorListReport {
    let node_operators = list
        .node_operators
        .into_iter()
        .map(|operator| NnsNodeOperatorRow {
            node_operator_principal: operator.principal,
            node_provider_principal: operator.node_provider_principal,
            node_allowance: operator.node_allowance,
            data_center_id: operator.data_center_id,
            node_count: operator.node_count,
        })
        .collect::<Vec<_>>();
    NnsNodeOperatorListReport {
        schema_version: NNS_NODE_OPERATOR_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        node_operator_count: node_operators.len(),
        node_operators,
    }
}

///
/// NnsNodeOperatorSource
///
trait NnsNodeOperatorSource {
    fn fetch_node_operators(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeOperatorList, NnsNodeOperatorHostError>;
}

fn enforce_mainnet_network(network: &str) -> Result<(), NnsNodeOperatorHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(NnsNodeOperatorHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

///
/// LiveNnsNodeOperatorSource
///
struct LiveNnsNodeOperatorSource;

impl NnsNodeOperatorSource for LiveNnsNodeOperatorSource {
    fn fetch_node_operators(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetNodeOperatorList, NnsNodeOperatorHostError> {
        Ok(fetch_mainnet_node_operator_list(request)?)
    }
}

fn resolve_node_operator(
    report: &NnsNodeOperatorListReport,
    input: &str,
) -> Result<(NnsNodeOperatorRow, String), NnsNodeOperatorHostError> {
    if let Ok(principal) = canonical_principal_text(input)
        && let Some(operator) = report
            .node_operators
            .iter()
            .find(|operator| operator.node_operator_principal == principal)
    {
        return Ok((operator.clone(), "node_operator_principal".to_string()));
    }

    let prefix = input.trim().to_ascii_lowercase();
    if prefix.is_empty() {
        return Err(NnsNodeOperatorHostError::NodeOperatorNotFound {
            input: input.to_string(),
        });
    }
    let matches = report
        .node_operators
        .iter()
        .filter(|operator| operator.node_operator_principal.starts_with(&prefix))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [operator] => Ok((
            operator.clone(),
            "node_operator_principal_prefix".to_string(),
        )),
        [] => Err(NnsNodeOperatorHostError::NodeOperatorNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsNodeOperatorHostError::AmbiguousNodeOperatorPrefix {
            prefix,
            matches: matches
                .into_iter()
                .map(|operator| operator.node_operator_principal)
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests;
