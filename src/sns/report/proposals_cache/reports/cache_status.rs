//! Module: sns::report::proposals_cache::reports::cache_status
//!
//! Responsibility: build proposal cache status reports.
//! Does not own: cache loading internals, refresh orchestration, or rendering.
//! Boundary: routes id/root status lookups into public status report DTOs.

use crate::sns::report::{
    SnsHostError, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus,
    cache_status::{
        SnsCacheStatusFamily, SnsCacheStatusPaths, SnsCacheStatusSummaryView,
        build_sns_cache_status_lookup,
    },
    proposals_cache::{
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        attempt::read_sns_proposals_attempt_status,
        paths::{SnsProposalsCachePaths, sns_network_cache_dir},
        storage::{
            invalid_sns_proposals_cache_summary, list_sns_proposals_cache_summaries,
            load_sns_proposals_cache_at, sns_proposals_cache_summary,
        },
    },
};
use std::path::{Path, PathBuf};

/// Build a local SNS proposal cache status report.
pub fn build_sns_proposals_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    let lookup = build_sns_cache_status_lookup::<SnsProposalsCacheStatusFamily>(
        &request.network,
        &request.icp_root,
        &request.input,
    )?;
    Ok(cache_status_report(
        request,
        lookup.cache_root,
        lookup.cache,
        lookup.expected_cache_path,
        lookup.refresh_attempt_path,
        lookup.latest_attempt,
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

struct SnsProposalsCacheStatusFamily;

impl SnsCacheStatusFamily for SnsProposalsCacheStatusFamily {
    type Attempt = SnsProposalsRefreshAttemptStatus;
    type Summary = SnsProposalsCacheSummary;

    fn network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
        sns_network_cache_dir(icp_root, network)
    }

    fn find_cache_by_id(
        icp_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<Self::Summary>, SnsHostError> {
        Ok(list_sns_proposals_cache_summaries(icp_root, network)?
            .into_iter()
            .find(|cache| cache.id == id && cache.cache_error.is_none()))
    }

    fn root_cache_paths(
        icp_root: &Path,
        network: &str,
        root_canister_id: &str,
    ) -> SnsCacheStatusPaths {
        let paths = SnsProposalsCachePaths::for_root(icp_root, network, root_canister_id);
        SnsCacheStatusPaths {
            cache_path: paths.cache_path,
            attempt_path: paths.attempt_path,
        }
    }

    fn load_root_cache_summary(
        cache_path: PathBuf,
        network: &str,
    ) -> Result<Self::Summary, SnsHostError> {
        Ok(
            match load_sns_proposals_cache_at(cache_path.clone(), network) {
                Ok(cache) => sns_proposals_cache_summary(cache_path, cache),
                Err(error) => invalid_sns_proposals_cache_summary(cache_path, error),
            },
        )
    }

    fn read_attempt_status(attempt_path: &Path) -> Option<Self::Attempt> {
        read_sns_proposals_attempt_status(attempt_path)
    }
}

impl SnsCacheStatusSummaryView for SnsProposalsCacheSummary {
    type Attempt = SnsProposalsRefreshAttemptStatus;

    fn refresh_attempt_path(&self) -> &str {
        &self.refresh_attempt_path
    }

    fn latest_attempt(&self) -> Option<Self::Attempt> {
        self.latest_attempt.clone()
    }
}
