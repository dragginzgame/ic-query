//! Module: sns::report::proposals_cache::reports::cache_list
//!
//! Responsibility: build proposal cache list reports.
//! Does not own: cache scanning internals, refresh attempts, or rendering.
//! Boundary: shapes complete proposal cache summaries into public report DTOs.

use super::super::{
    SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION, paths::sns_network_cache_dir,
    storage::list_sns_proposals_cache_summaries,
};
use crate::sns::report::{
    SnsHostError, SnsProposalsCacheListReport, SnsProposalsCacheListRequest,
    enforce_mainnet_network,
};

/// Build a local SNS proposal cache list report.
pub fn build_sns_proposals_cache_list_report(
    request: &SnsProposalsCacheListRequest,
) -> Result<SnsProposalsCacheListReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    let mut caches = list_sns_proposals_cache_summaries(&request.icp_root, &request.network)?;
    caches.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.root_canister_id.cmp(&right.root_canister_id))
    });
    Ok(SnsProposalsCacheListReport {
        schema_version: SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: cache_root.display().to_string(),
        cache_count: caches.len(),
        caches,
    })
}
