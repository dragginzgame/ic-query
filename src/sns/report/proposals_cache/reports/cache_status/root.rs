//! Module: sns::report::proposals_cache::reports::cache_status::root
//!
//! Responsibility: build proposal cache status reports for SNS root principals.
//! Does not own: numeric id lookup, text rendering, or cache list reports.
//! Boundary: resolves root-principal paths and latest attempt metadata.

use super::report::cache_status_report;
use crate::sns::report::{
    SnsHostError, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    proposals_cache::{
        attempt::read_sns_proposals_attempt_status,
        paths::SnsProposalsCachePaths,
        storage::{load_sns_proposals_cache_at, sns_proposals_cache_summary},
    },
};
use candid::Principal;

pub(super) fn build_root_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    let root_canister_id = Principal::from_text(&request.input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: request.input.clone(),
        })?
        .to_text();
    let paths =
        SnsProposalsCachePaths::for_root(&request.icp_root, &request.network, &root_canister_id);
    let cache = if paths.cache_path.is_file() {
        Some(sns_proposals_cache_summary(
            paths.cache_path.clone(),
            load_sns_proposals_cache_at(paths.cache_path.clone(), &request.network)?,
        ))
    } else {
        None
    };
    let latest_attempt = cache.as_ref().map_or_else(
        || read_sns_proposals_attempt_status(&paths.attempt_path),
        |cache| cache.latest_attempt.clone(),
    );
    Ok(cache_status_report(
        request,
        cache_root,
        cache,
        Some(paths.cache_path.display().to_string()),
        Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    ))
}
