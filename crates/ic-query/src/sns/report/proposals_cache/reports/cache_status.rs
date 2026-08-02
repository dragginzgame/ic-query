//! Module: sns::report::proposals_cache::reports::cache_status
//!
//! Responsibility: build proposal cache status reports.
//! Does not own: cache loading internals, refresh orchestration, or rendering.
//! Boundary: routes id/root status lookups into public status report DTOs.

use crate::sns::report::{
    SnsCacheStatusReport, SnsCacheStatusRequest, SnsHostError,
    cache_status::build_sns_cache_status_report,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION, paths::SnsProposalsCacheCollection,
    },
};

/// Build a local SNS proposal cache status report.
pub fn build_sns_proposals_cache_status_report(
    request: &SnsCacheStatusRequest,
) -> Result<SnsCacheStatusReport, SnsHostError> {
    build_sns_cache_status_report::<SnsProposalsCacheCollection>(
        request,
        SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
    )
}
