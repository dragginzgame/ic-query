use super::{
    CachedNnsSubnetTopologyReport, LiveNnsSubnetTopologySource,
    NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetTopologyCacheRequest,
    NnsSubnetTopologyFreshness, NnsSubnetTopologyHostError, NnsSubnetTopologyRefreshRequest,
    NnsSubnetTopologyReport, NnsSubnetTopologySource, error::enforce_mainnet_network,
    source::source_request,
};
use crate::{
    cache_file::{
        LoadJsonCacheErrorMapper, LoadJsonCacheRequest, RefreshLockRequest,
        create_parent_directory, load_json_cache, with_refresh_lock, write_text_atomically,
    },
    subnet_catalog::{MAINNET_REGISTRY_CANISTER_ID, parse_utc_timestamp_secs},
};
use std::{
    io,
    path::{Path, PathBuf},
};

const CACHE_DIR: &str = "subnet-topology";
const CACHE_FILE: &str = "report.json";

/// Return the canonical joined Subnet topology cache path.
#[must_use]
pub fn nns_subnet_topology_cache_path(icp_root: &Path, network: &str) -> PathBuf {
    cache_dir(icp_root, network).join(CACHE_FILE)
}

/// Return the canonical joined Subnet topology refresh-lock path.
#[must_use]
pub fn nns_subnet_topology_refresh_lock_path(icp_root: &Path, network: &str) -> PathBuf {
    cache_dir(icp_root, network).join("refresh.lock")
}

fn cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    icp_root.join(".icq").join(CACHE_DIR).join(network)
}

/// Load and validate the joined cache without making a live network call.
pub fn load_cached_nns_subnet_topology(
    request: &NnsSubnetTopologyCacheRequest,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    enforce_mainnet_network(&request.network)?;
    let cached = load_json_cache(
        LoadJsonCacheRequest {
            path: nns_subnet_topology_cache_path(&request.icp_root, &request.network),
            network: &request.network,
            expected_schema_version: NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
        },
        SubnetTopologyLoadErrors,
    )?;
    validate_report_identity(&cached.report, &request.network, None)?;
    Ok(CachedNnsSubnetTopologyReport {
        path: cached.path,
        report: cached.report,
    })
}

/// Explicitly fetch one exact-version report and atomically replace its joined cache.
pub fn refresh_nns_subnet_topology(
    request: &NnsSubnetTopologyRefreshRequest,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    refresh_nns_subnet_topology_with_source(request, &LiveNnsSubnetTopologySource)
}

/// Explicitly refresh with a caller-supplied source, primarily for deterministic collection.
pub fn refresh_nns_subnet_topology_with_source(
    request: &NnsSubnetTopologyRefreshRequest,
    source: &dyn NnsSubnetTopologySource,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let cache_path =
        nns_subnet_topology_cache_path(&request.cache.icp_root, &request.cache.network);
    let lock_path =
        nns_subnet_topology_refresh_lock_path(&request.cache.icp_root, &request.cache.network);
    create_parent_directory(&cache_path)?;
    with_refresh_lock(
        RefreshLockRequest {
            lock_path: &lock_path,
            target_path: &cache_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        NnsSubnetTopologyHostError::from,
        || {
            let source_request = source_request(
                &request.cache.network,
                &request.source_endpoint,
                request.now_unix_secs,
            );
            let report = source.fetch_subnet_topology_report(&source_request)?;
            validate_report_identity(
                &report,
                &request.cache.network,
                Some(&request.source_endpoint),
            )?;
            let report_json = serde_json::to_string_pretty(&report).map_err(|source| {
                NnsSubnetTopologyHostError::SerializeCache {
                    path: cache_path.clone(),
                    source,
                }
            })?;
            write_text_atomically(&cache_path, &report_json)?;
            Ok(CachedNnsSubnetTopologyReport {
                path: cache_path.clone(),
                report,
            })
        },
    )
}

/// Load the joined cache, refreshing only when it is missing.
pub fn load_or_refresh_missing_nns_subnet_topology(
    request: &NnsSubnetTopologyRefreshRequest,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    load_or_refresh_missing_nns_subnet_topology_with_source(request, &LiveNnsSubnetTopologySource)
}

/// Load the joined cache or use a caller-supplied source only when it is missing.
pub fn load_or_refresh_missing_nns_subnet_topology_with_source(
    request: &NnsSubnetTopologyRefreshRequest,
    source: &dyn NnsSubnetTopologySource,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    match load_cached_nns_subnet_topology(&request.cache) {
        Ok(cached) => Ok(cached),
        Err(NnsSubnetTopologyHostError::MissingCache { .. }) => {
            refresh_nns_subnet_topology_with_source(request, source)
        }
        Err(error) => Err(error),
    }
}

/// Load the joined cache, refreshing when it is missing or stale.
pub fn load_or_refresh_stale_nns_subnet_topology(
    request: &NnsSubnetTopologyRefreshRequest,
    stale_after_seconds: u64,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    load_or_refresh_stale_nns_subnet_topology_with_source(
        request,
        stale_after_seconds,
        &LiveNnsSubnetTopologySource,
    )
}

/// Load the joined cache or use a caller-supplied source when it is missing or stale.
pub fn load_or_refresh_stale_nns_subnet_topology_with_source(
    request: &NnsSubnetTopologyRefreshRequest,
    stale_after_seconds: u64,
    source: &dyn NnsSubnetTopologySource,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    match load_cached_nns_subnet_topology(&request.cache) {
        Ok(cached)
            if !nns_subnet_topology_freshness(
                &cached.report,
                request.now_unix_secs,
                stale_after_seconds,
            )
            .stale =>
        {
            Ok(cached)
        }
        Ok(_) | Err(NnsSubnetTopologyHostError::MissingCache { .. }) => {
            refresh_nns_subnet_topology_with_source(request, source)
        }
        Err(error) => Err(error),
    }
}

/// Derive caller-relative cache freshness without changing or refreshing state.
#[must_use]
pub fn nns_subnet_topology_freshness(
    report: &NnsSubnetTopologyReport,
    now_unix_secs: u64,
    stale_after_seconds: u64,
) -> NnsSubnetTopologyFreshness {
    let Some(fetched_at_unix_secs) = parse_utc_timestamp_secs(&report.fetched_at) else {
        return NnsSubnetTopologyFreshness {
            stale: true,
            reason: "fetched_at_unparseable".to_string(),
            stale_after_seconds,
            fetched_at_unix_secs: None,
            age_seconds: None,
        };
    };
    let Some(age_seconds) = now_unix_secs.checked_sub(fetched_at_unix_secs) else {
        return NnsSubnetTopologyFreshness {
            stale: true,
            reason: "fetched_at_in_future".to_string(),
            stale_after_seconds,
            fetched_at_unix_secs: Some(fetched_at_unix_secs),
            age_seconds: None,
        };
    };
    let stale = age_seconds > stale_after_seconds;
    NnsSubnetTopologyFreshness {
        stale,
        reason: if stale { "expired" } else { "fresh" }.to_string(),
        stale_after_seconds,
        fetched_at_unix_secs: Some(fetched_at_unix_secs),
        age_seconds: Some(age_seconds),
    }
}

fn validate_report_identity(
    report: &NnsSubnetTopologyReport,
    network: &str,
    endpoint: Option<&str>,
) -> Result<(), NnsSubnetTopologyHostError> {
    if report.network != network {
        return Err(NnsSubnetTopologyHostError::RefreshNetworkMismatch {
            requested: network.to_string(),
            actual: report.network.clone(),
        });
    }
    if report.registry_canister_id != MAINNET_REGISTRY_CANISTER_ID {
        return Err(NnsSubnetTopologyHostError::RegistryCanisterMismatch {
            expected: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            actual: report.registry_canister_id.clone(),
        });
    }
    if let Some(endpoint) = endpoint
        && report.source_endpoint != endpoint
    {
        return Err(NnsSubnetTopologyHostError::SourceEndpointMismatch {
            requested: endpoint.to_string(),
            actual: report.source_endpoint.clone(),
        });
    }
    report.validate()?;
    Ok(())
}

struct SubnetTopologyLoadErrors;

impl LoadJsonCacheErrorMapper for SubnetTopologyLoadErrors {
    type Error = NnsSubnetTopologyHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        NnsSubnetTopologyHostError::MissingCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error {
        NnsSubnetTopologyHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        NnsSubnetTopologyHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        NnsSubnetTopologyHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        NnsSubnetTopologyHostError::CacheNetworkMismatch { requested, actual }
    }
}
