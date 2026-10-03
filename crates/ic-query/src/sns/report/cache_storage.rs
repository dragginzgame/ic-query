//! Module: sns::report::cache_storage
//!
//! Responsibility: shared SNS snapshot storage and collection-family contract.
//! Does not own: cache payload models, refresh collection, or publication policy.
//! Boundary: centralizes discovery, strict id/root lookup, validation, loading, and error mapping.

use crate::{
    HostCacheError,
    cache::validate_cache_collection_completeness,
    cache_file::{
        CacheFileError, CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest, ManagedReadBudget,
        load_json_cache_strict,
    },
    snapshot_cache::{
        SnapshotEnvelope, SnapshotIdentityMismatch, collect_full_collection_snapshot_paths,
        load_complete_snapshot_for_key,
    },
    sns::report::{
        SNS_CACHE_COMPONENT, SnsHostError,
        cache_paths::{
            SnsCacheCollection, SnsSnapshotCachePaths, sns_snapshot_key_for_cache_path,
            sns_snapshot_network_cache_dir,
        },
        enforce_mainnet_network,
    },
};
use candid::Principal;
use serde::{Deserialize as SerdeDeserialize, Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

pub(in crate::sns::report) const SNS_CACHE_SCAN_MAXIMUM_BYTES: u64 = 1024 * 1024 * 1024;

pub(in crate::sns::report) const fn sns_cache_scan_limit_exceeded(error: &SnsHostError) -> bool {
    matches!(
        error,
        SnsHostError::Cache(HostCacheError::Operation {
            source: CacheFileError::ScanLimitExceeded { .. },
            ..
        })
    )
}

///
/// SnsCacheStorageFamily
///
/// Schema and missing-cache contract owned by one SNS snapshot collection.
///

pub(in crate::sns::report) trait SnsCacheStorageFamily:
    SnsCacheCollection
{
    type Data: DeserializeOwned;

    const CACHE_SCHEMA_VERSION: u32;
    const CACHE_FIELDS: &'static [&'static str];
    const CACHE_ITEM_NAME: &'static str;

    fn missing_cache_error(path: PathBuf) -> SnsHostError;
    fn missing_cache_for_id(_id: usize, root: PathBuf) -> SnsHostError {
        Self::missing_cache_error(root)
    }
    fn row_count(data: &Self::Data) -> usize;
    fn validate_rows(data: &Self::Data) -> Result<(), String>;
}

///
/// SnsStoredCache
///
/// Complete stored snapshot type associated with one SNS cache family.
///

pub(in crate::sns::report) type SnsStoredCache<Family> =
    SnapshotEnvelope<SnsCacheMetadata, <Family as SnsCacheStorageFamily>::Data>;

///
/// SnsStoredCacheWithPath
///
/// Loaded SNS snapshot paired with its concrete complete-cache path.
///

pub(in crate::sns::report) type SnsStoredCacheWithPath<Family> = (PathBuf, SnsStoredCache<Family>);

///
/// SnsCacheMetadata
///
/// Shared persisted identity metadata for a complete SNS collection cache.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(in crate::sns::report) struct SnsCacheMetadata {
    pub(in crate::sns::report) sns_wasm_canister_id: String,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) name: String,
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) governance_canister_id: String,
}

///
/// SnsCacheLookupHeader
///
/// Required snapshot identity fields loaded while locating an SNS cache by id.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
pub(in crate::sns::report) struct SnsCacheLookupHeader {
    pub(in crate::sns::report) schema_version: u32,
    pub(in crate::sns::report) network: String,
    pub(in crate::sns::report) domain: String,
    pub(in crate::sns::report) entity: String,
    pub(in crate::sns::report) collection: String,
    pub(in crate::sns::report) scope: String,
    pub(in crate::sns::report) id: usize,
}

impl JsonCacheReport for SnsCacheLookupHeader {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

/// Collect complete SNS snapshot paths for one cache collection.
pub(in crate::sns::report) fn collect_sns_cache_paths<Family>(
    cache_root: &Path,
    network: &str,
) -> Result<Vec<PathBuf>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let root = sns_snapshot_network_cache_dir(cache_root, network);
    collect_full_collection_snapshot_paths(cache_root, &root, Family::COLLECTION).map_err(
        |source| SnsHostError::from(HostCacheError::operation(SNS_CACHE_COMPONENT, source)),
    )
}

/// Read and validate one SNS snapshot cache header.
pub(in crate::sns::report) fn read_sns_cache_header<Family>(
    cache_root: &Path,
    path: &Path,
    network: &str,
    budget: &mut ManagedReadBudget,
) -> Result<SnsCacheLookupHeader, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let cached: CachedJsonReport<SnsCacheLookupHeader> = load_json_cache_strict(
        LoadJsonCacheRequest {
            component: SNS_CACHE_COMPONENT,
            cache_root,
            path: path.to_path_buf(),
            network,
            expected_schema_version: Family::CACHE_SCHEMA_VERSION,
            maximum_bytes: 512 * 1024 * 1024,
        },
        Family::CACHE_FIELDS,
        Some(budget),
    )
    .map_err(|error| match error {
        HostCacheError::MissingCache { path, .. } => Family::missing_cache_error(path),
        error => error.into(),
    })?;
    Ok(cached.report)
}

/// Load the unique SNS snapshot and bind its final contents to the requested id.
fn load_unique_sns_cache_by_id<Family>(
    paths: Vec<PathBuf>,
    id: usize,
    budget: &mut ManagedReadBudget,
    mut read_id: impl FnMut(&Path, &mut ManagedReadBudget) -> Result<usize, SnsHostError>,
    mut load_cache: impl FnMut(
        PathBuf,
        &mut ManagedReadBudget,
    ) -> Result<SnsStoredCache<Family>, SnsHostError>,
) -> Result<Option<SnsStoredCacheWithPath<Family>>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let mut matching = None;
    for path in paths {
        if read_id(&path, budget)? != id {
            continue;
        }
        if matching.replace(path).is_some() {
            return Err(SnsHostError::AmbiguousCacheId { id });
        }
    }
    let Some(path) = matching else {
        return Ok(None);
    };
    let cache = load_cache(path.clone(), budget)?;
    // Publication can replace the file after header discovery.
    if cache.metadata.id != id {
        return Err(SnsHostError::CacheIdentityMismatch {
            path,
            field: "id",
            expected: id.to_string(),
            actual: cache.metadata.id.to_string(),
        });
    }
    Ok(Some((path, cache)))
}

/// Load the unique complete SNS cache whose validated header claims an id.
pub(in crate::sns::report) fn load_sns_cache_by_id<Family>(
    cache_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<SnsStoredCacheWithPath<Family>>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let mut budget = ManagedReadBudget::new(SNS_CACHE_SCAN_MAXIMUM_BYTES);
    load_unique_sns_cache_by_id::<Family>(
        collect_sns_cache_paths::<Family>(cache_root, network)?,
        id,
        &mut budget,
        |path, budget| {
            read_sns_cache_header::<Family>(cache_root, path, network, budget)
                .map(|header| header.id)
        },
        |path, budget| load_sns_cache_at::<Family>(cache_root, path, network, Some(budget)),
    )
}

/// Load one complete SNS cache by root canister principal.
pub(in crate::sns::report) fn load_sns_cache_for_root<Family>(
    cache_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> Result<SnsStoredCacheWithPath<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let path =
        SnsSnapshotCachePaths::<Family>::for_root(cache_root, network, root_canister_id).cache_path;
    let cache = load_sns_cache_at::<Family>(cache_root, path.clone(), network, None)?;
    Ok((path, cache))
}

/// Resolve an SNS id or root principal to one complete family cache.
pub(in crate::sns::report) fn load_sns_cache_for_input<Family>(
    cache_root: &Path,
    network: &str,
    input: &str,
) -> Result<SnsStoredCacheWithPath<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    enforce_mainnet_network(network)?;
    if let Ok(id) = input.parse::<usize>() {
        let network_root = sns_snapshot_network_cache_dir(cache_root, network);
        return load_sns_cache_by_id::<Family>(cache_root, network, id)?
            .ok_or_else(|| Family::missing_cache_for_id(id, network_root));
    }

    let root_canister_id = parse_sns_root_canister_input(input)?;
    load_sns_cache_for_root::<Family>(cache_root, network, &root_canister_id)
}

/// Parse and normalize an SNS root canister principal input.
pub(in crate::sns::report) fn parse_sns_root_canister_input(
    input: &str,
) -> Result<String, SnsHostError> {
    Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })
        .map(|principal| principal.to_text())
}

/// Load and validate one complete SNS snapshot cache.
pub(in crate::sns::report) fn load_sns_cache_at<Family>(
    cache_root: &Path,
    path: PathBuf,
    network: &str,
    budget: Option<&mut ManagedReadBudget>,
) -> Result<SnsStoredCache<Family>, SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    let key = sns_snapshot_key_for_cache_path::<Family>(network, &path);
    let cache = load_complete_snapshot_for_key(
        LoadJsonCacheRequest {
            component: SNS_CACHE_COMPONENT,
            cache_root,
            path: path.clone(),
            network,
            expected_schema_version: Family::CACHE_SCHEMA_VERSION,
            maximum_bytes: 512 * 1024 * 1024,
        },
        budget,
        &key,
        Family::CACHE_FIELDS,
        Family::missing_cache_error,
        |completeness| SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: format!("cached SNS {} snapshot is not complete", Family::COLLECTION),
        },
        |mismatch| sns_identity_mismatch_error(path.clone(), mismatch),
    )?;
    validate_sns_cache::<Family>(&path, &cache)?;
    Ok(cache)
}

fn validate_sns_cache<Family>(
    path: &Path,
    cache: &SnsStoredCache<Family>,
) -> Result<(), SnsHostError>
where
    Family: SnsCacheStorageFamily,
{
    validate_cache_collection_completeness(&cache.completeness, Family::row_count(&cache.data))
        .map_err(|reason| invalid_sns_cache_error(path, reason))?;
    if cache.completeness.point_in_time_guaranteed {
        return Err(invalid_sns_cache_error(
            path,
            format!(
                "SNS Governance {} pagination cannot claim a point-in-time guarantee",
                Family::CACHE_ITEM_NAME
            ),
        ));
    }
    validate_sns_cache_metadata(path, &cache.metadata, &cache.entity)?;
    Family::validate_rows(&cache.data).map_err(|reason| invalid_sns_cache_error(path, reason))
}

fn validate_sns_cache_metadata(
    path: &Path,
    metadata: &SnsCacheMetadata,
    entity: &str,
) -> Result<(), SnsHostError> {
    if metadata.id == 0 {
        return Err(invalid_sns_cache_error(
            path,
            "SNS list id must be greater than zero".to_string(),
        ));
    }
    if metadata.root_canister_id != entity {
        return Err(invalid_sns_cache_error(
            path,
            format!(
                "root_canister_id is {}, expected {entity}",
                metadata.root_canister_id
            ),
        ));
    }
    if metadata.governance_canister_id.is_empty() {
        return Err(invalid_sns_cache_error(
            path,
            "governance_canister_id must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn invalid_sns_cache_error(path: &Path, reason: String) -> SnsHostError {
    SnsHostError::InvalidCache {
        path: path.to_path_buf(),
        reason,
    }
}

fn sns_identity_mismatch_error(path: PathBuf, mismatch: SnapshotIdentityMismatch) -> SnsHostError {
    SnsHostError::CacheIdentityMismatch {
        path,
        field: mismatch.field,
        expected: mismatch.expected,
        actual: mismatch.actual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::CacheCollectionCompleteness, cache_file::write_managed_text_atomically,
        sns::report::proposals_cache::SnsProposalsCacheCollection, test_support::temp_dir,
    };
    use std::{cell::Cell, fs};

    fn cache(id: usize) -> SnsStoredCache<SnsProposalsCacheCollection> {
        serde_json::from_value(serde_json::json!({
            "schema_version": 1, "network": "ic", "domain": "sns", "entity": "aaaaa-aa",
            "collection": "proposals", "scope": "full", "id": id, "name": "Fixture",
            "sns_wasm_canister_id": "qaa6y-5yaaa-aaaaa-aaafa-cai", "root_canister_id": "aaaaa-aa",
            "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "source_endpoint": "https://icp-api.io", "fetched_at": "2026-10-03T00:00:00Z",
            "fetched_by": "fixture", "completeness": CacheCollectionCompleteness::api_exhausted(100, 1, 0, false),
            "proposals": [],
        }))
        .unwrap()
    }

    #[test]
    fn cache_id_lookup_loads_only_the_unique_matching_snapshot() {
        let loads = Cell::new(0);
        let (path, snapshot) = load_unique_sns_cache_by_id::<SnsProposalsCacheCollection>(
            vec![PathBuf::from("1"), PathBuf::from("2"), PathBuf::from("3")],
            2,
            &mut ManagedReadBudget::new(1024),
            |path, _| {
                path.to_string_lossy()
                    .parse::<usize>()
                    .map_err(|_| SnsHostError::InvalidLookup {
                        input: path.display().to_string(),
                    })
            },
            |path, _| {
                assert_eq!(path, PathBuf::from("2"));
                loads.set(loads.get() + 1);
                Ok(cache(2))
            },
        )
        .expect("lookup succeeds")
        .expect("matching snapshot");

        assert_eq!(path, PathBuf::from("2"));
        assert_eq!(snapshot.metadata.id, 2);
        assert_eq!(loads.get(), 1);
    }

    #[test]
    fn cache_id_lookup_rejects_duplicate_headers_before_loading_rows() {
        let result = load_unique_sns_cache_by_id::<SnsProposalsCacheCollection>(
            vec![PathBuf::from("a"), PathBuf::from("b")],
            7,
            &mut ManagedReadBudget::new(1024),
            |_, _| Ok(7),
            |_, _| panic!("ambiguous headers must not load rows"),
        );

        assert!(matches!(
            result,
            Err(SnsHostError::AmbiguousCacheId { id: 7 })
        ));
    }

    #[test]
    fn cache_id_lookup_preserves_unrelated_header_errors() {
        let result = load_unique_sns_cache_by_id::<SnsProposalsCacheCollection>(
            vec![PathBuf::from("matching"), PathBuf::from("invalid")],
            7,
            &mut ManagedReadBudget::new(1024),
            |path, _| {
                if path == Path::new("matching") {
                    Ok(7)
                } else {
                    Err(SnsHostError::InvalidCache {
                        path: path.to_path_buf(),
                        reason: "fixture".to_string(),
                    })
                }
            },
            |_, _| panic!("a header error must terminate lookup before loading rows"),
        );
        assert!(
            matches!(result, Err(SnsHostError::InvalidCache { path, .. }) if path == Path::new("invalid"))
        );
    }

    #[test]
    fn cache_id_lookup_binds_the_atomically_replaced_snapshot_to_requested_id() {
        let root = temp_dir("ic-query-sns-id-replacement");
        let path = root.join("sns/ic/aaaaa-aa/proposals/full.json");
        for replacement_id in [7, 8] {
            write_managed_text_atomically(&root, &path, &serde_json::to_string(&cache(7)).unwrap())
                .unwrap();
            let result = load_unique_sns_cache_by_id::<SnsProposalsCacheCollection>(
                vec![path.clone()],
                7,
                &mut ManagedReadBudget::new(1024 * 1024),
                |path, budget| {
                    read_sns_cache_header::<SnsProposalsCacheCollection>(&root, path, "ic", budget)
                        .map(|header| header.id)
                },
                |path, budget| {
                    let mut replacement = cache(replacement_id);
                    replacement.metadata.name = "Updated".to_string();
                    write_managed_text_atomically(
                        &root,
                        &path,
                        &serde_json::to_string(&replacement).unwrap(),
                    )
                    .unwrap();
                    load_sns_cache_at::<SnsProposalsCacheCollection>(
                        &root,
                        path,
                        "ic",
                        Some(budget),
                    )
                },
            );
            if replacement_id == 7 {
                let (_, snapshot) = result.unwrap().unwrap();
                assert_eq!(snapshot.metadata.id, 7);
                assert_eq!(snapshot.metadata.name, "Updated");
            } else {
                assert!(matches!(
                    result,
                    Err(SnsHostError::CacheIdentityMismatch { path: actual_path, field: "id", expected, actual })
                        if actual_path == path && expected == "7" && actual == "8"
                ));
            }
        }
        fs::remove_dir_all(root).unwrap();
    }
}
