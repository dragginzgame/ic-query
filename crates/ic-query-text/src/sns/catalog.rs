//! Module: sns::catalog
//!
//! Responsibility: render deployed-SNS catalog refresh results.
//! Does not own: refresh execution, cache IO, or JSON output.
//! Boundary: keeps replacement and enrichment-gap evidence visible to operators.

use crate::text_value::field;

use crate::text_value::yes_no;
use ic_query::sns::SnsCatalogRefreshReport;

/// Render a human-readable deployed-SNS catalog refresh report.
#[must_use]
pub fn sns_catalog_refresh_report_text(report: &SnsCatalogRefreshReport) -> String {
    [
        field("network", &report.network),
        field("fetched_at", &report.fetched_at),
        field("source_endpoint", &report.source_endpoint),
        field("sns_count", &(report.sns_count)),
        field("metadata_errors", &(report.metadata_error_count)),
        field("lifecycle_errors", &(report.lifecycle_error_count)),
        field(
            "replaced_existing_cache",
            &(yes_no(report.replaced_existing_cache)),
        ),
        field("cache_path", &report.cache_path),
        field("refresh_lock_path", &report.refresh_lock_path),
    ]
    .join("\n")
}
