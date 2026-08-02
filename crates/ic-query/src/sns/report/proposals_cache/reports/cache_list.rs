//! Module: sns::report::proposals_cache::reports::cache_list
//!
//! Responsibility: build proposal cache list reports.
//! Does not own: cache scanning internals, refresh attempts, or rendering.
//! Boundary: shapes complete proposal cache summaries into public report DTOs.

use crate::sns::report::{
    SnsCacheListReport, SnsCacheListRequest, SnsHostError, build_sns_cache_list_report,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION, storage::list_sns_proposals_cache_summaries,
    },
};

/// Build a local SNS proposal cache list report.
pub fn build_sns_proposals_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    build_sns_cache_list_report(
        request,
        SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        list_sns_proposals_cache_summaries,
    )
}
