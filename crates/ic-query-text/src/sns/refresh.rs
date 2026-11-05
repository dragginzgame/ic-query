//! Module: sns::refresh
//!
//! Responsibility: render SNS snapshot refresh reports as text.
//! Does not own: refresh execution, cache writes, report construction, or JSON output.
//! Boundary: formats complete SNS snapshot refresh results for humans.

use crate::sns::common::sns_identity_lines;

use crate::text_value::yes_no;
use ic_query::sns::SnsSnapshotRefreshReport;

/// Render one complete SNS snapshot refresh result, including publication warnings.
#[must_use]
pub fn sns_snapshot_refresh_report_text(report: &SnsSnapshotRefreshReport) -> String {
    let mut lines = sns_identity_lines(&report.context);
    lines.extend(crate::text_value::report_fields!(report;
        page_size,
        page_count,
        collection,
        row_count,
        complete = yes_no(report.complete),
    ));
    lines.extend(crate::sns::common::sns_collection_lines(&report.context));
    lines.extend(crate::text_value::refresh_outcome_lines(
        report.wrote_cache,
        report.replaced_existing_cache,
        report.attempt_finalization_error.as_deref(),
    ));
    lines.extend(crate::text_value::refresh_path_lines(
        &report.cache_path,
        &report.refresh_lock_path,
        &report.refresh_attempt_path,
    ));
    lines.join("\n")
}
