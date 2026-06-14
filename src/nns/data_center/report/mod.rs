use crate::ic_registry::{
    DEFAULT_MAINNET_ENDPOINT, MainnetDataCenterList, MainnetRegistryFetchRequest,
    fetch_mainnet_data_center_list,
};
use crate::subnet_catalog::MAINNET_NETWORK;
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
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text, nns_data_center_refresh_report_text,
};

pub const DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;
pub const NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_DATA_CENTER_INFO_REPORT_SCHEMA_VERSION: u32 = 1;
pub const NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

#[must_use]
pub fn nns_data_center_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("data-center")
        .join(network)
        .join("data-centers.json")
}

#[must_use]
pub fn nns_data_center_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("data-center")
        .join(network)
        .join("refresh.lock")
}

struct NnsDataCenterCacheErrors;

impl LoadJsonCacheErrorMapper for NnsDataCenterCacheErrors {
    type Error = NnsDataCenterHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        NnsDataCenterHostError::MissingCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: std::io::Error) -> Self::Error {
        NnsDataCenterHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        NnsDataCenterHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        NnsDataCenterHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        NnsDataCenterHostError::NetworkMismatch { requested, actual }
    }
}

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

fn load_cached_nns_data_center_report(
    request: &NnsDataCenterCacheRequest,
) -> Result<CachedNnsDataCenterReport, NnsDataCenterHostError> {
    enforce_mainnet_network(&request.network)?;
    let path = nns_data_center_cache_path(&request.icp_root, &request.network);
    let cached = load_json_cache(
        LoadJsonCacheRequest {
            path,
            network: &request.network,
            expected_schema_version: NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
        },
        NnsDataCenterCacheErrors,
    )?;
    Ok(CachedNnsDataCenterReport {
        path: cached.path,
        report: cached.report,
    })
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

fn refresh_nns_data_center_report_with_source(
    request: &NnsDataCenterRefreshRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterRefreshReport, NnsDataCenterHostError> {
    refresh_nns_data_center_cache_with_source(request, source).map(|(_, report)| report)
}

fn refresh_nns_data_center_cache_with_source(
    request: &NnsDataCenterRefreshRequest,
    source: &dyn NnsDataCenterSource,
) -> Result<(NnsDataCenterListReport, NnsDataCenterRefreshReport), NnsDataCenterHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let cache_path = nns_data_center_cache_path(&request.cache.icp_root, &request.cache.network);
    let lock_path =
        nns_data_center_refresh_lock_path(&request.cache.icp_root, &request.cache.network);
    let report = fetch_nns_data_center_list_report_with_source(
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
        data_center_cache_error,
        |path, source| NnsDataCenterHostError::SerializeCache { path, source },
    )?;
    let refresh_report = NnsDataCenterRefreshReport {
        schema_version: NNS_DATA_CENTER_REFRESH_REPORT_SCHEMA_VERSION,
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
        data_center_count: report.data_center_count,
    };
    Ok((report, refresh_report))
}

fn fetch_nns_data_center_list_report_with_source(
    network: &str,
    source_endpoint: &str,
    now_unix_secs: u64,
    source: &dyn NnsDataCenterSource,
) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
    enforce_mainnet_network(network)?;
    let fetched_at = format_utc_timestamp_secs(now_unix_secs);
    let mut fetch_request = MainnetRegistryFetchRequest::new(fetched_at);
    fetch_request.endpoint = source_endpoint.to_string();
    let list = source.fetch_data_centers(&fetch_request)?;
    Ok(data_center_report_from_list(list))
}

impl_nns_cache_error_mapper!(data_center_cache_error, NnsDataCenterHostError);

fn data_center_report_from_list(list: MainnetDataCenterList) -> NnsDataCenterListReport {
    let data_centers = list
        .data_centers
        .into_iter()
        .map(|data_center| NnsDataCenterRow {
            data_center_id: data_center.id,
            region: data_center.region,
            owner: data_center.owner,
            latitude: data_center.latitude,
            longitude: data_center.longitude,
            node_operator_count: data_center.node_operator_count,
            node_provider_count: data_center.node_provider_count,
            node_count: data_center.node_count,
        })
        .collect::<Vec<_>>();
    NnsDataCenterListReport {
        schema_version: NNS_DATA_CENTER_LIST_REPORT_SCHEMA_VERSION,
        network: list.network,
        registry_canister_id: list.registry_canister_id,
        registry_version: list.registry_version,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        data_center_count: data_centers.len(),
        data_centers,
    }
}

///
/// NnsDataCenterSource
///
trait NnsDataCenterSource {
    fn fetch_data_centers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetDataCenterList, NnsDataCenterHostError>;
}

///
/// LiveNnsDataCenterSource
///
struct LiveNnsDataCenterSource;

impl NnsDataCenterSource for LiveNnsDataCenterSource {
    fn fetch_data_centers(
        &self,
        request: &MainnetRegistryFetchRequest,
    ) -> Result<MainnetDataCenterList, NnsDataCenterHostError> {
        Ok(fetch_mainnet_data_center_list(request)?)
    }
}

fn enforce_mainnet_network(network: &str) -> Result<(), NnsDataCenterHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(NnsDataCenterHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

fn resolve_data_center(
    report: &NnsDataCenterListReport,
    input: &str,
) -> Result<(NnsDataCenterRow, String), NnsDataCenterHostError> {
    let normalized = input.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err(NnsDataCenterHostError::DataCenterNotFound {
            input: input.to_string(),
        });
    }
    if let Some(data_center) = report
        .data_centers
        .iter()
        .find(|data_center| data_center.data_center_id == normalized)
    {
        return Ok((data_center.clone(), "data_center_id".to_string()));
    }
    let matches = report
        .data_centers
        .iter()
        .filter(|data_center| data_center.data_center_id.starts_with(&normalized))
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [data_center] => Ok((data_center.clone(), "data_center_id_prefix".to_string())),
        [] => Err(NnsDataCenterHostError::DataCenterNotFound {
            input: input.to_string(),
        }),
        _ => Err(NnsDataCenterHostError::AmbiguousDataCenterPrefix {
            prefix: normalized,
            matches: matches
                .into_iter()
                .map(|data_center| data_center.data_center_id)
                .collect(),
        }),
    }
}

#[cfg(test)]
mod tests;
