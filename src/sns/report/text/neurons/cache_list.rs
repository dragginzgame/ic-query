use super::super::super::{SnsNeuronsCacheListReport, short_principal};
use crate::{
    nns::render::yes_no,
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
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}
