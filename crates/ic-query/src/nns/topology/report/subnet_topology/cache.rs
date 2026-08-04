use super::{
    CachedNnsSubnetTopologyReport, NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
    NnsSubnetTopologyCacheRequest, NnsSubnetTopologyFreshness, NnsSubnetTopologyHostError,
    NnsSubnetTopologyRefreshRequest, NnsSubnetTopologyReport, NnsSubnetTopologySource,
    error::enforce_mainnet_network, source::source_request,
};
use crate::{
    cache_file::{
        CacheRefreshReason, HostCacheError, HostJsonCacheErrorMapper, LoadJsonCacheRequest,
        RefreshLockRequest, create_parent_directory, host_cache_refresh_reason, load_json_cache,
        load_or_refresh_cache_with_error_policy, load_or_refresh_stale_cache_with_error_policy,
        with_refresh_lock, write_text_atomically,
    },
    freshness::freshness_facts,
    nns::LiveNnsSource,
    subnet_catalog::{MAINNET_REGISTRY_CANISTER_ID, parse_utc_timestamp_secs},
};
use std::path::{Path, PathBuf};

const CACHE_DIR: &str = "subnet-topology";
const CACHE_FILE: &str = "report.json";
const CACHE_COMPONENT: &str = "Subnet topology";

/// Return the canonical joined Subnet topology cache path.
#[must_use]
pub fn nns_subnet_topology_cache_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_dir(cache_root, network).join(CACHE_FILE)
}

/// Return the canonical joined Subnet topology refresh-lock path.
#[must_use]
pub fn nns_subnet_topology_refresh_lock_path(cache_root: &Path, network: &str) -> PathBuf {
    cache_dir(cache_root, network).join("refresh.lock")
}

fn cache_dir(cache_root: &Path, network: &str) -> PathBuf {
    cache_root.join("nns").join(network).join(CACHE_DIR)
}

/// Load and validate the joined cache without making a live network call.
pub fn load_cached_nns_subnet_topology(
    request: &NnsSubnetTopologyCacheRequest,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    enforce_mainnet_network(&request.network)?;
    let cached = load_json_cache(
        LoadJsonCacheRequest {
            path: nns_subnet_topology_cache_path(&request.cache_root, &request.network),
            network: &request.network,
            expected_schema_version: NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
        },
        HostJsonCacheErrorMapper::new(CACHE_COMPONENT),
    )
    .map_err(NnsSubnetTopologyHostError::from)?;
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
    refresh_nns_subnet_topology_with_source(request, &LiveNnsSource)
}

/// Explicitly refresh with a caller-supplied source, primarily for deterministic collection.
pub fn refresh_nns_subnet_topology_with_source(
    request: &NnsSubnetTopologyRefreshRequest,
    source: &dyn NnsSubnetTopologySource,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    enforce_mainnet_network(&request.cache.network)?;
    let cache_path =
        nns_subnet_topology_cache_path(&request.cache.cache_root, &request.cache.network);
    let lock_path =
        nns_subnet_topology_refresh_lock_path(&request.cache.cache_root, &request.cache.network);
    create_parent_directory(&cache_path)
        .map_err(|error| HostCacheError::operation(CACHE_COMPONENT, error))?;
    with_refresh_lock(
        RefreshLockRequest {
            lock_path: &lock_path,
            target_path: &cache_path,
            network: &request.cache.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: request.lock_stale_after_seconds,
        },
        |error| HostCacheError::operation(CACHE_COMPONENT, error).into(),
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
                HostCacheError::serialize_cache(CACHE_COMPONENT, cache_path.clone(), source)
            })?;
            write_text_atomically(&cache_path, &report_json)
                .map_err(|error| HostCacheError::operation(CACHE_COMPONENT, error))?;
            Ok(CachedNnsSubnetTopologyReport {
                path: cache_path.clone(),
                report,
            })
        },
    )
}

/// Load the joined cache, refreshing when it is missing or invalid.
pub fn load_or_refresh_missing_nns_subnet_topology(
    request: &NnsSubnetTopologyRefreshRequest,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    load_or_refresh_missing_nns_subnet_topology_with_source(request, &LiveNnsSource)
}

/// Load the joined cache or replace missing or invalid content with a caller-supplied source.
pub fn load_or_refresh_missing_nns_subnet_topology_with_source(
    request: &NnsSubnetTopologyRefreshRequest,
    source: &dyn NnsSubnetTopologySource,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    let expected_path =
        nns_subnet_topology_cache_path(&request.cache.cache_root, &request.cache.network);
    load_or_refresh_cache_with_error_policy(
        || load_cached_nns_subnet_topology(&request.cache),
        |error| subnet_topology_cache_refresh_reason(error, &expected_path),
        |_| {
            refresh_nns_subnet_topology_with_source(request, source)?;
            Ok(())
        },
    )
}

/// Load the joined cache, refreshing when it is missing, invalid, or stale.
pub fn load_or_refresh_stale_nns_subnet_topology(
    request: &NnsSubnetTopologyRefreshRequest,
    stale_after_seconds: u64,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    load_or_refresh_stale_nns_subnet_topology_with_source(
        request,
        stale_after_seconds,
        &LiveNnsSource,
    )
}

/// Load the joined cache or use a caller-supplied source when it is unusable or stale.
pub fn load_or_refresh_stale_nns_subnet_topology_with_source(
    request: &NnsSubnetTopologyRefreshRequest,
    stale_after_seconds: u64,
    source: &dyn NnsSubnetTopologySource,
) -> Result<CachedNnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
    let expected_path =
        nns_subnet_topology_cache_path(&request.cache.cache_root, &request.cache.network);
    load_or_refresh_stale_cache_with_error_policy(
        || load_cached_nns_subnet_topology(&request.cache),
        |cached| {
            nns_subnet_topology_freshness(
                &cached.report,
                request.now_unix_secs,
                stale_after_seconds,
            )
            .stale
        },
        |error| subnet_topology_cache_refresh_reason(error, &expected_path),
        |_| {
            refresh_nns_subnet_topology_with_source(request, source)?;
            Ok(())
        },
    )
}

/// Derive caller-relative cache freshness without changing or refreshing state.
#[must_use]
pub fn nns_subnet_topology_freshness(
    report: &NnsSubnetTopologyReport,
    now_unix_secs: u64,
    stale_after_seconds: u64,
) -> NnsSubnetTopologyFreshness {
    let freshness = freshness_facts(
        parse_utc_timestamp_secs(&report.fetched_at),
        now_unix_secs,
        stale_after_seconds,
    );
    NnsSubnetTopologyFreshness {
        stale: freshness.stale,
        reason: freshness.reason.to_string(),
        stale_after_seconds: freshness.stale_after_seconds,
        fetched_at_unix_secs: freshness.fetched_at_unix_secs,
        age_seconds: freshness.age_seconds,
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

fn subnet_topology_cache_refresh_reason(
    error: NnsSubnetTopologyHostError,
    expected_path: &Path,
) -> Result<CacheRefreshReason, NnsSubnetTopologyHostError> {
    match error {
        NnsSubnetTopologyHostError::Cache(error) => host_cache_refresh_reason(error, expected_path)
            .map_err(NnsSubnetTopologyHostError::Cache),
        NnsSubnetTopologyHostError::RefreshNetworkMismatch { .. }
        | NnsSubnetTopologyHostError::RegistryCanisterMismatch { .. }
        | NnsSubnetTopologyHostError::Validation(_) => {
            Ok(CacheRefreshReason::Invalid(expected_path.to_path_buf()))
        }
        error => Err(error),
    }
}
