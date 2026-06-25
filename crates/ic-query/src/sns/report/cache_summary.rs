//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary view helpers across SNS cache reports.
//! Does not own: cache storage, refresh attempts, or text rendering.
//! Boundary: keeps common cache-summary ordering deterministic.

use crate::{
    snapshot_cache::SNAPSHOT_CACHE_STATUS_INVALID,
    sns::report::{SnsHostError, enforce_mainnet_network},
};
use candid::Principal;
use std::path::{Path, PathBuf};

///
/// SnsCacheListLookup
///
/// Shared lookup result used to assemble SNS cache-list reports.
///

pub(in crate::sns::report) struct SnsCacheListLookup<Summary> {
    pub(in crate::sns::report) cache_root: String,
    pub(in crate::sns::report) caches: Vec<Summary>,
}

///
/// SnsCacheSummarySortKey
///
/// Stable ordering key implemented by SNS cache summary report rows.
///

pub(in crate::sns::report) trait SnsCacheSummarySortKey {
    fn id(&self) -> usize;
    fn root_canister_id(&self) -> &str;
    fn cache_path(&self) -> &str;
    fn cache_error(&self) -> Option<&str>;
}

///
/// SnsInvalidCacheSummaryFields
///
/// Shared invalid-cache fields reused by SNS cache summary DTOs.
///

pub(in crate::sns::report) struct SnsInvalidCacheSummaryFields {
    pub(in crate::sns::report) root_canister_id: String,
    pub(in crate::sns::report) cache_status: String,
    pub(in crate::sns::report) cache_error: Option<String>,
    pub(in crate::sns::report) complete: bool,
    pub(in crate::sns::report) row_count: usize,
    pub(in crate::sns::report) page_count: u32,
    pub(in crate::sns::report) page_size: u32,
    pub(in crate::sns::report) fetched_at: String,
    pub(in crate::sns::report) source_endpoint: String,
    pub(in crate::sns::report) cache_path: String,
    pub(in crate::sns::report) refresh_attempt_path: String,
}

///
/// SnsCacheListFamily
///
/// Family-specific hooks required by the shared SNS cache-list report flow.
///

pub(in crate::sns::report) trait SnsCacheListFamily {
    type Summary: SnsCacheSummarySortKey;

    fn network_cache_dir(icp_root: &Path, network: &str) -> PathBuf;
    fn list_cache_summaries(
        icp_root: &Path,
        network: &str,
    ) -> Result<Vec<Self::Summary>, SnsHostError>;
}

/// Build a deterministic cache-list lookup for one SNS cache family.
pub(in crate::sns::report) fn build_sns_cache_list_lookup<Family>(
    network: &str,
    icp_root: &Path,
) -> Result<SnsCacheListLookup<Family::Summary>, SnsHostError>
where
    Family: SnsCacheListFamily,
{
    enforce_mainnet_network(network)?;
    let cache_root = Family::network_cache_dir(icp_root, network)
        .display()
        .to_string();
    let mut caches = Family::list_cache_summaries(icp_root, network)?;
    sort_sns_cache_summaries(&mut caches);
    Ok(SnsCacheListLookup { cache_root, caches })
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

/// Sort SNS cache summaries by stable list id and root principal.
pub(in crate::sns::report) fn sort_sns_cache_summaries<T>(caches: &mut [T])
where
    T: SnsCacheSummarySortKey,
{
    caches.sort_by(|left, right| {
        left.id()
            .cmp(&right.id())
            .then_with(|| left.root_canister_id().cmp(right.root_canister_id()))
    });
}

/// Find a valid SNS cache summary by stable SNS list id.
pub(in crate::sns::report) fn find_valid_sns_cache_summary_by_id<T>(
    caches: impl IntoIterator<Item = T>,
    id: usize,
) -> Option<T>
where
    T: SnsCacheSummarySortKey,
{
    caches
        .into_iter()
        .find(|cache| cache.id() == id && cache.cache_error().is_none())
}

/// Build shared invalid-cache summary fields from a failed local cache read.
pub(in crate::sns::report) fn invalid_sns_cache_summary_fields(
    cache_path: &Path,
    refresh_attempt_path: &Path,
    error: &SnsHostError,
) -> SnsInvalidCacheSummaryFields {
    SnsInvalidCacheSummaryFields {
        root_canister_id: root_from_cache_path(cache_path),
        cache_status: SNAPSHOT_CACHE_STATUS_INVALID.to_string(),
        cache_error: Some(error.to_string()),
        complete: false,
        row_count: 0,
        page_count: 0,
        page_size: 0,
        fetched_at: "-".to_string(),
        source_endpoint: "-".to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_attempt_path: refresh_attempt_path.display().to_string(),
    }
}

fn root_from_cache_path(cache_path: &Path) -> String {
    cache_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map_or_else(
            || "-".to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
}
