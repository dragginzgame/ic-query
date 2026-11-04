//! Module: sns::metrics
//!
//! Responsibility: render bounded SNS Governance metrics as human-facing text.
//! Does not own: live calls, report construction, source validation, or JSON output.
//! Boundary: labels cached treasury evidence and raw timestamps without conflation.

use crate::sns::labels::sns_treasury_kind;
use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::duration::display_duration_seconds;
use ic_host_text::table::Alignment;

use crate::text_value::optional_text;
use crate::text_value::optional_u64_text;
use crate::text_value::yes_no;
use ic_query::sns::SnsMetricsReport;
use ic_query::sns::SnsTreasuryMetricRow;

/// Render one SNS Governance metrics report as human-facing text.
#[must_use]
pub fn sns_metrics_report_text(report: &SnsMetricsReport) -> String {
    let mut lines = vec![
        field("network", &report.context.network),
        field("sns_id", &(report.context.id)),
        field("name", &report.context.name),
        field("root_canister_id", &(report.context.root_canister_id)),
        field(
            "governance_canister_id",
            &(report.metrics.governance_canister_id),
        ),
        field("method", &(report.metrics.method.as_str())),
        field("call_type", &(report.metrics.call_type.as_str())),
        field(
            "time_window",
            &(display_duration_seconds(report.metrics.time_window_seconds)),
        ),
        field("time_window_seconds", &(report.metrics.time_window_seconds)),
        field(
            "point_in_time_guaranteed",
            &(yes_no(report.metrics.point_in_time_guaranteed)),
        ),
        field(
            "treasury_metrics_cached",
            &(yes_no(report.metrics.treasury_metrics_cached)),
        ),
        field(
            "num_recently_submitted_proposals",
            &(optional_u64_text(report.metrics.num_recently_submitted_proposals)),
        ),
        field(
            "num_recently_executed_proposals",
            &(optional_u64_text(report.metrics.num_recently_executed_proposals)),
        ),
        field(
            "last_ledger_block_timestamp",
            &(optional_u64_text(report.metrics.last_ledger_block_timestamp)),
        ),
        field(
            "genesis_timestamp_seconds",
            &(optional_u64_text(report.metrics.genesis_timestamp_seconds)),
        ),
        field(
            "treasury_metric_count",
            &(report.metrics.treasury_metrics.len()),
        ),
        field(
            "sns_wasm_canister_id",
            &(report.context.sns_wasm_canister_id),
        ),
        field("fetched_at", &report.context.fetched_at),
        field("source_endpoint", &report.context.source_endpoint),
        field("fetched_by", &report.context.fetched_by),
        String::new(),
        "treasury_metrics:".to_string(),
        treasury_metrics_text(&report.metrics.treasury_metrics),
        String::new(),
        "voting_power_metrics:".to_string(),
    ];
    lines.push(report.metrics.voting_power_metrics.as_ref().map_or_else(
        || "-".to_string(),
        |metrics| {
            table(
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
            )
        },
    ));
    lines.join("\n")
}

fn treasury_metrics_text(metrics: &[SnsTreasuryMetricRow]) -> String {
    if metrics.is_empty() {
        return "-".to_string();
    }
    table(
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
                    sns_treasury_kind(metric.treasury).to_string(),
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
    )
}
