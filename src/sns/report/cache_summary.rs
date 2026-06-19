//! Module: sns::report::cache_summary
//!
//! Responsibility: share cache-summary view helpers across SNS cache reports.
//! Does not own: cache storage, refresh attempts, or text rendering.
//! Boundary: keeps common cache-summary ordering deterministic.

use crate::sns::report::{SnsHostError, enforce_mainnet_network};
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
