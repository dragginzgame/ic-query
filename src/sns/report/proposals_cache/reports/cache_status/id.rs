//! Module: sns::report::proposals_cache::reports::cache_status::id
//!
//! Responsibility: build proposal cache status reports for numeric SNS ids.
//! Does not own: root-principal lookup, text rendering, or cache file parsing.
//! Boundary: resolves id lookups through proposal cache storage.

use super::report::cache_status_report;
use crate::sns::report::{
    SnsHostError, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    proposals_cache::storage::{find_sns_proposals_cache_by_id, sns_proposals_cache_summary},
};

pub(super) fn build_id_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
    id: usize,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    let cache = find_sns_proposals_cache_by_id(&request.icp_root, &request.network, id)?
        .map(|(path, cache)| sns_proposals_cache_summary(path, cache));
    let refresh_attempt_path = cache
        .as_ref()
        .map(|cache| cache.refresh_attempt_path.clone());
    let latest_attempt = cache
        .as_ref()
        .and_then(|cache| cache.latest_attempt.clone());
    Ok(cache_status_report(
        request,
        cache_root,
        cache,
        None,
        refresh_attempt_path,
        latest_attempt,
    ))
}
