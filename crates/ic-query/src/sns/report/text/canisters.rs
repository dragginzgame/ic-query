//! Module: sns::report::text::canisters
//!
//! Responsibility: render SNS Root canister inventory and health reports.
//! Does not own: Root calls, report construction, JSON serialization, or lookup.
//! Boundary: presents raw report fields compactly without changing the JSON contract.

use crate::{
    human_quantity::{decimal_byte_count_text, decimal_cycle_count_text},
    sns::report::SnsCanisterReport,
    table::{ColumnAlign, render_table},
    text_value::{optional_text, sanitize_text, truncate_text, yes_no},
};

const MODULE_HASH_TEXT_CHARS: usize = 12;

/// Render one SNS Root canister report as human-facing text.
#[must_use]
pub fn sns_canister_report_text(report: &SnsCanisterReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", sanitize_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("canister_count: {}", report.canister_count),
        format!("health_status_count: {}", report.health_status_count),
        format!(
            "reported_zero_cycles_count: {}",
            report.reported_zero_cycles_count
        ),
        format!(
            "cycles_unavailable_count: {}",
            report.cycles_unavailable_count
        ),
        format!("gap_count: {}", report.gap_count),
        format!(
            "health_query_status: {}",
            if report.health_query_gap.is_some() {
                "failed"
            } else {
                "succeeded"
            }
        ),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!("inventory_method: {}", report.inventory_method.as_str()),
        format!("health_method: {}", report.health_method.as_str()),
        format!("health_call_type: {}", report.health_call_type.as_str()),
        format!(
            "health_update_canister_list: {}",
            yes_no(report.health_update_canister_list)
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        String::new(),
        canisters_table(report),
    ];

    if let Some(gap) = &report.health_query_gap {
        lines.extend([
            String::new(),
            "health_query_gap:".to_string(),
            render_table(
                &["METHOD", "REASON"],
                &[[gap.method.as_str().to_string(), sanitize_text(&gap.reason)]],
                &[ColumnAlign::Left, ColumnAlign::Left],
            ),
        ]);
    }

    if !report.gaps.is_empty() {
        lines.push(String::new());
        lines.push("gaps:".to_string());
        lines.push(relation_gaps_table(report));
    }

    lines.join("\n")
}

fn canisters_table(report: &SnsCanisterReport) -> String {
    render_table(
        &[
            "ROLE",
            "CANISTER",
            "STATUS",
            "MODULE HASH",
            "CYCLES",
            "MEMORY",
            "CONTROLLERS",
        ],
        &report
            .canisters
            .iter()
            .map(|canister| {
                [
                    canister.role.as_str().to_string(),
                    canister.canister_id.clone(),
                    canister
                        .status
                        .map_or_else(|| "-".to_string(), |status| status.as_str().to_string()),
                    canister.module_hash_hex.as_ref().map_or_else(
                        || "-".to_string(),
                        |hash| truncate_text(hash, MODULE_HASH_TEXT_CHARS),
                    ),
                    canister
                        .cycles
                        .as_deref()
                        .map_or_else(|| "-".to_string(), decimal_cycle_count_text),
                    canister
                        .memory_size
                        .as_deref()
                        .map_or_else(|| "-".to_string(), decimal_byte_count_text),
                    canister.controllers.len().to_string(),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Right,
            ColumnAlign::Right,
            ColumnAlign::Right,
        ],
    )
}

fn relation_gaps_table(report: &SnsCanisterReport) -> String {
    render_table(
        &["KIND", "ROLE", "INVENTORY CANISTER", "SUMMARY CANISTER"],
        &report
            .gaps
            .iter()
            .map(|gap| {
                [
                    gap.kind.as_str().to_string(),
                    gap.role.as_str().to_string(),
                    optional_text(gap.inventory_canister_id.as_ref()),
                    optional_text(gap.summary_canister_id.as_ref()),
                ]
            })
            .collect::<Vec<_>>(),
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
        ],
    )
}
