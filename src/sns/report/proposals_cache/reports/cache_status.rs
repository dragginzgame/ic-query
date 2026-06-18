//! Module: sns::report::proposals_cache::reports::cache_status
//!
//! Responsibility: build proposal cache status reports.
//! Does not own: cache loading internals, refresh orchestration, or rendering.
//! Boundary: routes id/root status lookups into public status report DTOs.

use crate::sns::report::{
    SnsHostError, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, enforce_mainnet_network,
    parse_sns_root_canister_input,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        attempt::read_sns_proposals_attempt_status,
        paths::{SnsProposalsCachePaths, sns_network_cache_dir},
        storage::{
            find_sns_proposals_cache_by_id, load_sns_proposals_cache_at,
            sns_proposals_cache_summary,
        },
    },
};

/// Build a local SNS proposal cache status report.
pub fn build_sns_proposals_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    if let Ok(id) = request.input.parse::<usize>() {
        return build_id_cache_status_report(request, cache_root.display().to_string(), id);
    }
    build_root_cache_status_report(request, cache_root.display().to_string())
}

fn build_id_cache_status_report(
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

fn build_root_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    let root_canister_id = parse_sns_root_canister_input(&request.input)?;
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

fn cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
    cache: Option<SnsProposalsCacheSummary>,
    expected_cache_path: Option<String>,
    refresh_attempt_path: Option<String>,
    latest_attempt: Option<SnsProposalsRefreshAttemptStatus>,
) -> SnsProposalsCacheStatusReport {
    SnsProposalsCacheStatusReport {
        schema_version: SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root,
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path,
        refresh_attempt_path,
        latest_attempt,
    }
}
