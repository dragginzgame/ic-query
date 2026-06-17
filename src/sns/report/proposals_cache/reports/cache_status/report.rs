//! Module: sns::report::proposals_cache::reports::cache_status::report
//!
//! Responsibility: assemble proposal cache status report DTOs.
//! Does not own: cache lookup, root parsing, or text rendering.
//! Boundary: keeps status report field mapping centralized.

use super::super::super::SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION;
use crate::sns::report::{
    SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest, SnsProposalsCacheSummary,
    SnsProposalsRefreshAttemptStatus,
};

pub(super) fn cache_status_report(
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
