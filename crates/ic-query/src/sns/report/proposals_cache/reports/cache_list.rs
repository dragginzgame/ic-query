//! Module: sns::report::proposals_cache::reports::cache_list
//!
//! Responsibility: build proposal cache list reports.
//! Does not own: cache scanning internals, refresh attempts, or rendering.
//! Boundary: shapes complete proposal cache summaries into public report DTOs.

use crate::sns::report::{
    SnsCacheListFamily, SnsCacheListReport, SnsCacheListRequest, SnsCacheSummary, SnsHostError,
    build_sns_cache_list_lookup,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION, paths::sns_network_cache_dir,
        storage::list_sns_proposals_cache_summaries,
    },
};
use std::path::{Path, PathBuf};

/// Build a local SNS proposal cache list report.
pub fn build_sns_proposals_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    let lookup = build_sns_cache_list_lookup::<SnsProposalsCacheListFamily>(
        &request.network,
        &request.cache_root,
    )?;
    Ok(SnsCacheListReport {
        schema_version: SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: lookup.cache_root,
        cache_count: lookup.caches.len(),
        caches: lookup.caches,
    })
}

struct SnsProposalsCacheListFamily;

impl SnsCacheListFamily for SnsProposalsCacheListFamily {
    type Summary = SnsCacheSummary;

    fn network_cache_dir(cache_root: &Path, network: &str) -> PathBuf {
        sns_network_cache_dir(cache_root, network)
    }

    fn list_cache_summaries(
        cache_root: &Path,
        network: &str,
    ) -> Result<Vec<Self::Summary>, SnsHostError> {
        list_sns_proposals_cache_summaries(cache_root, network)
    }
}
