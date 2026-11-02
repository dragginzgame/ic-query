//! Module: sns::cache
//!
//! Responsibility: render shared SNS cache list and status report DTOs.
//! Does not own: cache discovery, refresh execution, or JSON output shape.
//! Boundary: keeps neuron and proposal cache text aligned while varying refresh commands.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;
use ic_host_text::table::render;

use crate::sns::common::optional_text;
use crate::sns::common::push_cache_error_lines;
use crate::sns::short_principal;
use crate::text_value::yes_no;
use ic_query::sns::SnsCacheListReport;
use ic_query::sns::SnsCacheStatusReport;
use ic_query::sns::SnsRefreshAttemptStatus;

/// Render an SNS neuron cache-list report as human-readable text.
#[must_use]
pub fn sns_neurons_cache_list_report_text(report: &SnsCacheListReport) -> String {
    sns_cache_list_report_text(report)
}

/// Render an SNS proposal cache-list report as human-readable text.
#[must_use]
pub fn sns_proposals_cache_list_report_text(report: &SnsCacheListReport) -> String {
    sns_cache_list_report_text(report)
}

/// Render an SNS neuron cache-status report as human-readable text.
#[must_use]
pub fn sns_neurons_cache_status_report_text(report: &SnsCacheStatusReport) -> String {
    sns_cache_status_report_text(report, "neuron")
}

/// Render an SNS proposal cache-status report as human-readable text.
#[must_use]
pub fn sns_proposals_cache_status_report_text(report: &SnsCacheStatusReport) -> String {
    sns_cache_status_report_text(report, "proposal")
}

fn sns_cache_list_report_text(report: &SnsCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("cache_root: {}", terminal_text(&report.cache_root)),
        format!("cache_count: {}", report.caches.len()),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render(
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
                    cache.valid().map_or_else(
                        || ["invalid", "-", "-", "-", "-", "-", "-", "-"].map(str::to_string),
                        |data| {
                            [
                                "ok".to_string(),
                                data.id.to_string(),
                                data.name.clone(),
                                short_principal(&data.root_canister_id),
                                "yes".to_string(),
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
                Alignment::Right,
                Alignment::Left,
                Alignment::Left,
                Alignment::Left,
                Alignment::Right,
                Alignment::Right,
                Alignment::Left,
            ],
            REPORT_TABLE_OPTIONS,
        ));
        push_cache_error_lines(&mut lines, &report.caches);
    }
    lines.join("\n")
}

fn sns_cache_status_report_text(report: &SnsCacheStatusReport, refresh_family: &str) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("input: {}", terminal_text(&report.input)),
        format!("cache_root: {}", terminal_text(&report.cache_root)),
        format!("found: {}", yes_no(report.found)),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.push(format!("cache_status: {}", cache.cache_status()));
        if let Some(data) = cache.valid() {
            lines.extend([
                format!("sns_id: {}", data.id),
                format!("name: {}", terminal_text(&data.name)),
                format!("root_canister_id: {}", data.root_canister_id),
                format!("governance_canister_id: {}", data.governance_canister_id),
                "complete: yes".to_string(),
                format!("row_count: {}", data.row_count),
                format!("page_count: {}", data.page_count),
                format!("page_size: {}", data.page_size),
                format!("fetched_at: {}", terminal_text(&data.fetched_at)),
                format!("source_endpoint: {}", terminal_text(&data.source_endpoint)),
            ]);
        }
        lines.push(format!("cache_path: {}", terminal_text(&cache.cache_path)));
        lines.push(format!(
            "refresh_attempt_path: {}",
            terminal_text(&cache.refresh_attempt_path)
        ));
        if let Some(error) = cache.cache_error() {
            lines.push(format!("cache_error: {}", terminal_text(error)));
        }
    } else {
        lines.push(format!(
            "expected_cache_path: {}",
            terminal_text(&report.expected_cache_path)
        ));
        lines.push(format!(
            "refresh_attempt_path: {}",
            terminal_text(&report.refresh_attempt_path)
        ));
        lines.push(format!(
            "refresh_hint: icq sns {refresh_family} refresh {}",
            terminal_text(&report.input)
        ));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.push(String::new());
        lines.extend(attempt_lines(attempt));
    }
    lines.join("\n")
}

fn attempt_lines(attempt: &SnsRefreshAttemptStatus) -> [String; 9] {
    [
        "latest_attempt:".to_string(),
        format!("  status: {}", attempt.lifecycle.status),
        format!(
            "  started_at: {}",
            terminal_text(&attempt.lifecycle.started_at)
        ),
        format!(
            "  updated_at: {}",
            terminal_text(&attempt.lifecycle.updated_at)
        ),
        format!("  page_size: {}", attempt.lifecycle.page_size),
        format!("  pages_fetched: {}", attempt.lifecycle.pages_fetched),
        format!("  rows_fetched: {}", attempt.lifecycle.rows_fetched),
        format!(
            "  last_cursor: {}",
            optional_text(attempt.lifecycle.last_cursor.as_ref())
        ),
        format!(
            "  last_error: {}",
            optional_text(attempt.lifecycle.last_error.as_ref())
        ),
    ]
}
