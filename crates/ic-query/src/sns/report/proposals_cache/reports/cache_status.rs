//! Module: sns::report::proposals_cache::reports::cache_status
//!
//! Responsibility: build proposal cache status reports.
//! Does not own: cache loading internals, refresh orchestration, or rendering.
//! Boundary: routes id/root status lookups into public status report DTOs.

use crate::sns::report::{
    SnsHostError, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    SnsProposalsCacheSummary, SnsRefreshAttemptStatus,
    cache_status::{
        SnsCacheStatusFamily, SnsCacheStatusPaths, SnsCacheStatusSummaryView,
        build_sns_cache_status_lookup,
    },
    find_sns_cache_summary_by_id,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        attempt::read_sns_proposals_attempt_status_strict,
        paths::{SnsProposalsCachePaths, sns_network_cache_dir},
        storage::{
            collect_sns_proposals_cache_paths, load_sns_proposals_cache_summary_at,
            read_sns_proposals_cache_header,
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
    latest_attempt: Option<SnsRefreshAttemptStatus>,
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
    type Attempt = SnsRefreshAttemptStatus;
    type Summary = SnsProposalsCacheSummary;

    const COLLECTION: &'static str = "proposals";

    fn network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
        sns_network_cache_dir(icp_root, network)
    }

    fn find_cache_by_id(
        icp_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<Self::Summary>, SnsHostError> {
        find_sns_cache_summary_by_id(
            collect_sns_proposals_cache_paths(icp_root, network)?,
            id,
            |path| read_sns_proposals_cache_header(path, network).map(|header| header.metadata.id),
            |path| load_sns_proposals_cache_summary_at(path, network),
        )
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
        Ok(load_sns_proposals_cache_summary_at(cache_path, network))
    }

    fn read_attempt_status(
        attempt_path: &Path,
        network: &str,
    ) -> Result<Option<Self::Attempt>, SnsHostError> {
        read_sns_proposals_attempt_status_strict(attempt_path, network)
    }
}

impl SnsCacheStatusSummaryView for SnsProposalsCacheSummary {
    fn refresh_attempt_path(&self) -> &str {
        &self.refresh_attempt_path
    }
}
