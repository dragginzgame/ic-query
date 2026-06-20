//! Module: sns::report::text::proposals::cache_list
//!
//! Responsibility: render SNS proposal cache list reports as text.
//! Does not own: cache discovery, refresh execution, or JSON output shape.
//! Boundary: formats discovered proposal snapshots for humans.

use crate::{
    sns::report::{SnsProposalsCacheListReport, text::common::push_cache_error_lines},
    table::{ColumnAlign, render_table},
};

pub fn sns_proposals_cache_list_report_text(report: &SnsProposalsCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("cache_root: {}", report.cache_root),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "STATUS",
                "ID",
                "NAME",
                "ROOT",
                "ROWS",
                "PAGES",
                "FETCHED_AT",
            ],
            &report
                .caches
                .iter()
                .map(|cache| {
                    [
                        cache.cache_status.clone(),
                        cache.id.to_string(),
                        cache.name.clone(),
                        cache.root_canister_id.clone(),
                        cache.row_count.to_string(),
                        cache.page_count.to_string(),
                        cache.fetched_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
        push_cache_error_lines(&mut lines, &report.caches);
    }
    lines.join("\n")
}
