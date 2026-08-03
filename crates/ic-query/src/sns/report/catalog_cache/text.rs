//! Module: sns::report::catalog_cache::text
//!
//! Responsibility: render deployed-SNS catalog refresh results.
//! Does not own: refresh execution, cache IO, or JSON output.
//! Boundary: keeps replacement and metadata-gap evidence visible to operators.

use super::SnsCatalogRefreshReport;
use crate::text_value::{sanitize_text, yes_no};

/// Render a human-readable deployed-SNS catalog refresh report.
#[must_use]
pub fn sns_catalog_refresh_report_text(report: &SnsCatalogRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("sns_count: {}", report.sns_count),
        format!("metadata_errors: {}", report.metadata_error_count),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}
