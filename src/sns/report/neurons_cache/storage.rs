use super::super::{
    SnsHostError, SnsNeuronRow, SnsNeuronsCacheSummary, SnsNeuronsSort, enforce_mainnet_network,
};
use super::{
    SNS_NEURONS_CACHE_SCHEMA_VERSION,
    attempt::read_sns_neurons_attempt_status,
    model::{SnsNeuronsCache, SnsNeuronsCacheHeader},
    paths::{
        sns_network_cache_dir, sns_neurons_attempt_path_for_cache_path, sns_neurons_cache_path,
    },
};
use crate::cache_file::{
    CachedJsonReport, LoadJsonCacheErrorHandlers, LoadJsonCacheRequest, load_json_cache,
};
use candid::Principal;
use std::{
    cmp::Reverse,
    fs,
    path::{Path, PathBuf},
};

pub(super) fn load_sns_neurons_cache_for_input(
    icp_root: &Path,
    network: &str,
    input: &str,
) -> Result<(PathBuf, SnsNeuronsCache), SnsHostError> {
    enforce_mainnet_network(network)?;
    if let Ok(id) = input.parse::<usize>() {
        return find_sns_neurons_cache_by_id(icp_root, network, id)?.ok_or_else(|| {
            SnsHostError::MissingNeuronsCacheForId {
                id,
                root: sns_network_cache_dir(icp_root, network),
            }
        });
    }

    let root_canister_id = Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })?
        .to_text();
    let path = sns_neurons_cache_path(icp_root, network, &root_canister_id);
    let cache = load_sns_neurons_cache(icp_root, network, &root_canister_id)?;
    Ok((path, cache))
}

pub(super) fn list_sns_neurons_cache_summaries(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<SnsNeuronsCacheSummary>, SnsHostError> {
    collect_sns_neurons_cache_paths(icp_root, network)?
        .into_iter()
        .map(|path| {
            let cache = load_sns_neurons_cache_at(path.clone(), network)?;
            Ok(sns_neurons_cache_summary(path, cache))
        })
        .collect()
}

pub(super) fn sns_neurons_cache_summary(
    cache_path: PathBuf,
    cache: SnsNeuronsCache,
) -> SnsNeuronsCacheSummary {
    let attempt_path = sns_neurons_attempt_path_for_cache_path(&cache_path);
    SnsNeuronsCacheSummary {
        id: cache.id,
        name: cache.name,
        root_canister_id: cache.root_canister_id,
        governance_canister_id: cache.governance_canister_id,
        complete: cache.completeness.status == "api_exhausted",
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: attempt_path.display().to_string(),
        latest_attempt: read_sns_neurons_attempt_status(&attempt_path),
    }
}

pub(super) fn find_sns_neurons_cache_by_id(
    icp_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsNeuronsCache)>, SnsHostError> {
    for path in collect_sns_neurons_cache_paths(icp_root, network)? {
        let header = read_sns_neurons_cache_header(&path, network)?;
        if header.id == id {
            let cache = load_sns_neurons_cache_at(path.clone(), network)?;
            return Ok(Some((path, cache)));
        }
    }
    Ok(None)
}

pub(super) fn load_sns_neurons_cache_at(
    path: PathBuf,
    network: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    let cached: CachedJsonReport<SnsNeuronsCache> = load_json_cache(
        LoadJsonCacheRequest {
            path,
            network,
            expected_schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        },
        LoadJsonCacheErrorHandlers {
            missing_cache: |path| SnsHostError::MissingNeuronsCache { path },
            read_cache: |path, source| SnsHostError::ReadCache { path, source },
            parse_cache: |path, source| SnsHostError::ParseCache { path, source },
            unsupported_schema: |version, expected| SnsHostError::UnsupportedCacheSchemaVersion {
                version,
                expected,
            },
            network_mismatch: |requested, actual| SnsHostError::CacheNetworkMismatch {
                requested,
                actual,
            },
        },
    )?;
    if cached.report.completeness.status != "api_exhausted" {
        return Err(SnsHostError::IncompleteRefresh {
            pages_fetched: cached.report.completeness.page_count,
            rows_fetched: cached.report.completeness.row_count,
            reason: "cached SNS neurons snapshot is not complete".to_string(),
        });
    }
    Ok(cached.report)
}

pub(super) fn sort_sns_neurons(neurons: &mut [SnsNeuronRow], sort: SnsNeuronsSort) {
    match sort {
        SnsNeuronsSort::Api => {}
        SnsNeuronsSort::Id => neurons.sort_by(|left, right| left.neuron_id.cmp(&right.neuron_id)),
        SnsNeuronsSort::Stake => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.cached_neuron_stake_e8s),
                neuron.neuron_id.clone(),
            )
        }),
        SnsNeuronsSort::Maturity => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.maturity_e8s_equivalent),
                neuron.neuron_id.clone(),
            )
        }),
        SnsNeuronsSort::Created => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.created_timestamp_seconds),
                neuron.neuron_id.clone(),
            )
        }),
    }
}

fn load_sns_neurons_cache(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    let path = sns_neurons_cache_path(icp_root, network, root_canister_id);
    load_sns_neurons_cache_at(path, network)
}

fn collect_sns_neurons_cache_paths(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError> {
    let root = sns_network_cache_dir(icp_root, network);
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => {
            return Err(SnsHostError::ReadCache { path: root, source });
        }
    };
    let mut cache_paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| SnsHostError::ReadCache {
            path: root.clone(),
            source,
        })?;
        let path = entry.path().join("neurons").join("full.json");
        if path.is_file() {
            cache_paths.push(path);
        }
    }
    cache_paths.sort();
    Ok(cache_paths)
}

fn read_sns_neurons_cache_header(
    path: &Path,
    network: &str,
) -> Result<SnsNeuronsCacheHeader, SnsHostError> {
    let data = fs::read(path).map_err(|source| SnsHostError::ReadCache {
        path: path.to_path_buf(),
        source,
    })?;
    let header: SnsNeuronsCacheHeader =
        serde_json::from_slice(&data).map_err(|source| SnsHostError::ParseCache {
            path: path.to_path_buf(),
            source,
        })?;
    if header.schema_version != SNS_NEURONS_CACHE_SCHEMA_VERSION {
        return Err(SnsHostError::UnsupportedCacheSchemaVersion {
            version: header.schema_version,
            expected: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        });
    }
    if header.network != network {
        return Err(SnsHostError::CacheNetworkMismatch {
            requested: network.to_string(),
            actual: header.network,
        });
    }
    Ok(header)
}
