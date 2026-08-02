//! Module: sns::report::proposals_cache::reports::cache_status
//!
//! Responsibility: build proposal cache status reports.
//! Does not own: cache loading internals, refresh orchestration, or rendering.
//! Boundary: routes id/root status lookups into public status report DTOs.

use crate::sns::report::{
    SnsCacheStatusReport, SnsCacheStatusRequest, SnsCacheSummary, SnsHostError,
    SnsRefreshAttemptStatus,
    cache_attempt::read_sns_refresh_attempt_status_strict,
    cache_status::{SnsCacheStatusFamily, SnsCacheStatusPaths, build_sns_cache_status_report},
    find_sns_cache_summary_by_id,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        paths::{SnsProposalsCacheCollection, SnsProposalsCachePaths},
        storage::{
            collect_sns_proposals_cache_paths, load_sns_proposals_cache_summary_at,
            read_sns_proposals_cache_header,
        },
    },
};
use std::path::{Path, PathBuf};

/// Build a local SNS proposal cache status report.
pub fn build_sns_proposals_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    build_sns_cache_status_report::<SnsProposalsCacheStatusFamily>(
        request,
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    )
}

struct SnsProposalsCacheStatusFamily;

impl SnsCacheStatusFamily for SnsProposalsCacheStatusFamily {
    type Collection = SnsProposalsCacheCollection;

    fn find_cache_by_id(
        cache_root: &Path,
        network: &str,
        id: usize,
    ) -> Result<Option<SnsCacheSummary>, SnsHostError> {
        find_sns_cache_summary_by_id(
            collect_sns_proposals_cache_paths(cache_root, network)?,
            id,
            |path| read_sns_proposals_cache_header(path, network).map(|header| header.metadata.id),
            |path| load_sns_proposals_cache_summary_at(path, network),
        )
    }

    fn root_cache_paths(
        cache_root: &Path,
        network: &str,
        root_canister_id: &str,
    ) -> SnsCacheStatusPaths {
        let paths = SnsProposalsCachePaths::for_root(cache_root, network, root_canister_id);
        SnsCacheStatusPaths {
            cache_path: paths.cache_path,
            attempt_path: paths.attempt_path,
        }
    }

    fn load_root_cache_summary(
        cache_path: PathBuf,
        network: &str,
    ) -> Result<SnsCacheSummary, SnsHostError> {
        Ok(load_sns_proposals_cache_summary_at(cache_path, network))
    }

    fn read_attempt_status(
        attempt_path: &Path,
        network: &str,
    ) -> Result<Option<SnsRefreshAttemptStatus>, SnsHostError> {
        read_sns_refresh_attempt_status_strict(attempt_path, network)
    }
}
