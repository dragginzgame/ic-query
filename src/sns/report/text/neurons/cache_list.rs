//! Module: sns::report::text::neurons::cache_list
//!
//! Responsibility: render SNS neuron cache list reports as text.
//! Does not own: cache discovery, cache refresh, report construction, or JSON output.
//! Boundary: formats discovered neuron snapshots into a human-readable table.

use crate::{
    nns::render::yes_no,
    sns::report::{SnsNeuronsCacheListReport, short_principal},
    table::{ColumnAlign, render_table},
};

#[must_use]
pub fn sns_neurons_cache_list_report_text(report: &SnsNeuronsCacheListReport) -> String {
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
                "COMPLETE",
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
                        short_principal(&cache.root_canister_id),
                        yes_no(cache.complete).to_string(),
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
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
        for cache in &report.caches {
            if let Some(error) = cache.cache_error.as_ref() {
                lines.push(format!("cache_error: {}: {error}", cache.cache_path));
            }
        }
    }
    lines.join("\n")
}
