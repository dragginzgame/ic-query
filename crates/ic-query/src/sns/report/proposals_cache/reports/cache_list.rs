//! Module: sns::report::proposals_cache::reports::cache_list
//!
//! Responsibility: build proposal cache list reports.
//! Does not own: cache scanning internals, refresh attempts, or rendering.
//! Boundary: shapes complete proposal cache summaries into public report DTOs.

use crate::sns::report::{
    SnsCacheListReport, SnsCacheListRequest, SnsHostError, build_sns_cache_list_report,
    load_sns_cache_summaries,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        storage::{collect_sns_proposals_cache_paths, load_sns_proposals_cache_at},
    },
};

/// Build a local SNS proposal cache list report.
pub fn build_sns_proposals_cache_list_report(
    request: &SnsCacheListRequest,
) -> Result<SnsCacheListReport, SnsHostError> {
    build_sns_cache_list_report(
        request,
        SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        |cache_root, network| {
            let paths = collect_sns_proposals_cache_paths(cache_root, network)?;
            Ok(load_sns_cache_summaries(
                paths,
                network,
                load_sns_proposals_cache_at,
            ))
        },
    )
}
