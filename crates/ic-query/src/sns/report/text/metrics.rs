//! Module: sns::report::text::metrics
//!
//! Responsibility: render bounded SNS Governance metrics as human-facing text.
//! Does not own: live calls, report construction, source validation, or JSON output.
//! Boundary: labels cached treasury evidence and raw timestamps without conflation.

use crate::text_value::REPORT_TABLE_OPTIONS;
use ic_host_text::{
    escape::terminal_text,
    table::{Alignment, render},
};

use crate::{
    duration::display_duration_seconds,
    sns::report::{SnsMetricsReport, SnsTreasuryMetricRow},
    text_value::{optional_text, optional_u64_text, yes_no},
};

/// Render one SNS Governance metrics report as human-facing text.
#[must_use]
pub fn sns_metrics_report_text(report: &SnsMetricsReport) -> String {
    let mut lines = vec![
        format!("network: {}", terminal_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", terminal_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("method: {}", report.method.as_str()),
        format!("call_type: {}", report.call_type.as_str()),
        format!(
            "time_window: {}",
            display_duration_seconds(report.time_window_seconds)
        ),
        format!("time_window_seconds: {}", report.time_window_seconds),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!(
            "treasury_metrics_cached: {}",
            yes_no(report.treasury_metrics_cached)
        ),
        format!(
            "num_recently_submitted_proposals: {}",
            optional_u64_text(report.num_recently_submitted_proposals)
        ),
        format!(
            "num_recently_executed_proposals: {}",
            optional_u64_text(report.num_recently_executed_proposals)
        ),
        format!(
            "last_ledger_block_timestamp: {}",
            optional_u64_text(report.last_ledger_block_timestamp)
        ),
        format!(
            "genesis_timestamp_seconds: {}",
            optional_u64_text(report.genesis_timestamp_seconds)
        ),
        format!("treasury_metric_count: {}", report.treasury_metric_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", terminal_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", terminal_text(&report.fetched_by)),
        String::new(),
        "treasury_metrics:".to_string(),
        treasury_metrics_text(&report.treasury_metrics),
        String::new(),
        "voting_power_metrics:".to_string(),
    ];
    lines.push(report.voting_power_metrics.as_ref().map_or_else(
        || "-".to_string(),
        |metrics| {
            render(
                &["FIELD", "VALUE"],
                &[
                    [
                        "governance_total_potential_voting_power".to_string(),
                        optional_u64_text(metrics.governance_total_potential_voting_power),
                    ],
                    [
                        "timestamp_seconds".to_string(),
                        optional_u64_text(metrics.timestamp_seconds),
                    ],
                ],
                &[Alignment::Left, Alignment::Right],
                REPORT_TABLE_OPTIONS,
            )
        },
    ));
    lines.join("\n")
}

fn treasury_metrics_text(metrics: &[SnsTreasuryMetricRow]) -> String {
    if metrics.is_empty() {
        return "-".to_string();
    }
    render(
        &[
            "KIND",
            "CODE",
            "NAME",
            "LEDGER",
            "OWNER",
            "SUBACCOUNT",
            "AMOUNT_E8S",
            "ORIGINAL_E8S",
            "TIMESTAMP",
        ],
        &metrics
            .iter()
            .map(|metric| {
                [
                    metric.treasury_kind.as_str().to_string(),
                    metric.treasury.to_string(),
                    optional_text(metric.name.as_ref()),
                    optional_text(metric.ledger_canister_id.as_ref()),
                    optional_text(metric.account_owner.as_ref()),
                    optional_text(metric.account_subaccount_hex.as_ref()),
                    optional_u64_text(metric.amount_e8s),
                    optional_u64_text(metric.original_amount_e8s),
                    optional_u64_text(metric.timestamp_seconds),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            Alignment::Left,
            Alignment::Right,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ],
        REPORT_TABLE_OPTIONS,
    )
}
