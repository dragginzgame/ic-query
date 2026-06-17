//! Module: sns::report::text::proposals::refresh
//!
//! Responsibility: render SNS proposal cache and refresh reports as text.
//! Does not own: cache discovery, refresh execution, or JSON output shape.
//! Boundary: formats proposal snapshot operational reports for humans.

use crate::sns::report::{
    SnsProposalsCacheListReport, SnsProposalsCacheStatusReport, SnsProposalsRefreshAttemptStatus,
    SnsProposalsRefreshReport,
};
use crate::table::{ColumnAlign, render_table};

pub fn sns_proposals_refresh_report_text(report: &SnsProposalsRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("proposal_count: {}", report.proposal_count),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("complete: {}", yes_no(report.complete)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
    ]
    .join("\n")
}

pub fn sns_proposals_cache_list_report_text(report: &SnsProposalsCacheListReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("cache_root: {}", report.cache_root),
        format!("cache_count: {}", report.cache_count),
    ];
    if !report.caches.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["ID", "NAME", "ROOT", "ROWS", "PAGES", "FETCHED_AT"],
            &report
                .caches
                .iter()
                .map(|cache| {
                    [
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
                ColumnAlign::Right,
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

pub fn sns_proposals_cache_status_report_text(report: &SnsProposalsCacheStatusReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("cache_root: {}", report.cache_root),
        format!("input: {}", report.input),
        format!("found: {}", yes_no(report.found)),
    ];
    if let Some(cache) = report.cache.as_ref() {
        lines.extend([
            format!("sns_id: {}", cache.id),
            format!("name: {}", cache.name),
            format!("root_canister_id: {}", cache.root_canister_id),
            format!("governance_canister_id: {}", cache.governance_canister_id),
            format!("complete: {}", yes_no(cache.complete)),
            format!("row_count: {}", cache.row_count),
            format!("page_count: {}", cache.page_count),
            format!("page_size: {}", cache.page_size),
            format!("fetched_at: {}", cache.fetched_at),
            format!("source_endpoint: {}", cache.source_endpoint),
            format!("cache_path: {}", cache.cache_path),
            format!("refresh_attempt_path: {}", cache.refresh_attempt_path),
        ]);
    } else if let Some(path) = report.expected_cache_path.as_ref() {
        lines.push(format!("expected_cache_path: {path}"));
        lines.push(format!(
            "refresh_hint: icq sns proposals refresh {}",
            report.input
        ));
    }
    if let Some(path) = report.refresh_attempt_path.as_ref() {
        lines.push(format!("refresh_attempt_path: {path}"));
    }
    if let Some(attempt) = report.latest_attempt.as_ref() {
        lines.extend(attempt_lines(attempt));
    }
    lines.join("\n")
}

fn attempt_lines(attempt: &SnsProposalsRefreshAttemptStatus) -> Vec<String> {
    vec![
        "latest_attempt:".to_string(),
        format!("  status: {}", attempt.status),
        format!("  started_at: {}", attempt.started_at),
        format!("  updated_at: {}", attempt.updated_at),
        format!("  page_size: {}", attempt.page_size),
        format!("  pages_fetched: {}", attempt.pages_fetched),
        format!("  rows_fetched: {}", attempt.rows_fetched),
        format!(
            "  last_cursor: {}",
            attempt.last_cursor.as_deref().unwrap_or("-")
        ),
        format!(
            "  last_error: {}",
            attempt.last_error.as_deref().unwrap_or("-")
        ),
    ]
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
