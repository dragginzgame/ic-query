//! Module: sns::cache
//!
//! Responsibility: render shared SNS cache list and status report DTOs.
//! Does not own: cache discovery, refresh execution, or JSON output shape.
//! Boundary: renders complete neuron snapshot and refresh-attempt evidence.

use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::sns::common::push_cache_error_lines;
use crate::text_value::yes_no;
use ic_query::sns::SnsCacheListReport;
use ic_query::sns::SnsCacheStatusReport;

/// Render an SNS neuron cache-list report as human-readable text.
#[must_use]
pub fn sns_neurons_cache_list_report_text(report: &SnsCacheListReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        network,
        cache_root,
        cache_count = report.caches.len(),
    ));
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(table(
            &["STATUS", "GOVERNANCE", "ROWS", "PAGES", "FETCHED_AT"],
            &report
                .caches
                .iter()
                .map(|cache| {
                    cache.valid().map_or_else(
                        || ["invalid", "-", "-", "-", "-"].map(str::to_string),
                        |data| {
                            [
                                "ok".to_string(),
                                data.governance_canister_id.clone(),
                                data.row_count.to_string(),
                                data.page_count.to_string(),
                                data.fetched_at.clone(),
                            ]
                        },
                    )
                })
                .collect::<Vec<_>>(),
            &[
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ],
        ));
    }
    push_cache_error_lines(&mut lines, &report.caches);
    lines.join("\n")
}

/// Render an SNS neuron cache-status report as human-readable text.
#[must_use]
pub fn sns_neurons_cache_status_report_text(report: &SnsCacheStatusReport) -> String {
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        network,
        input,
        cache_root,
        found = yes_no(report.found),
    ));
    if let Some(cache) = report.cache.as_ref() {
        lines.push(field("cache_status", &(cache.cache_status())));
        if let Some(data) = cache.valid() {
            lines.extend(crate::text_value::report_fields!(data;
                governance_canister_id,
                complete = "yes",
                row_count,
                page_count,
                page_size,
                fetched_at,
                source_endpoint,
            ));
        }
        lines.push(field("cache_path", &cache.cache_path));
        lines.push(field("refresh_attempt_path", &cache.refresh_attempt_path));
        if let Some(error) = cache.cache_error() {
            lines.push(field("cache_error", &(error)));
        }
    } else {
        lines.push(field("expected_cache_path", &report.expected_cache_path));
        lines.push(field("refresh_attempt_path", &report.refresh_attempt_path));
        lines.push(format!(
            "refresh_hint: icq sns neuron refresh {}",
            terminal_text(&report.input)
        ));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(String::new());
        lines.extend(crate::report::refresh_attempt_lines(&attempt.lifecycle));
    }
    lines.join("\n")
}
