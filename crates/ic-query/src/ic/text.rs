//! Module: ic::text
//!
//! Responsibility: render IC Dashboard canister reports as human-facing text.
//! Does not own: REST calls, report construction, JSON serialization, or command output.
//! Boundary: keeps raw values intact in JSON while making nullable text fields readable.

use crate::{
    ic::IcCanisterReport,
    text_value::{sanitize_text, yes_no},
};

/// Render one official Dashboard canister report as human-facing text.
#[must_use]
pub fn ic_canister_report_text(report: &IcCanisterReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("authority: {}", sanitize_text(&report.authority)),
        format!("canister_id: {}", report.canister_id),
        format!("dashboard_id: {}", report.dashboard_id),
        format!("name: {}", text_or_dash(&report.name)),
        format!(
            "canister_type: {}",
            report
                .canister_type
                .as_deref()
                .map_or_else(|| "-".to_string(), text_or_dash)
        ),
        format!("subnet_id: {}", report.subnet_id),
        format!("controller_count: {}", report.controllers.len()),
        format!("language: {}", text_or_dash(&report.language)),
        format!("module_hash: {}", text_or_dash(&report.module_hash)),
        format!(
            "dashboard_updated_at: {}",
            sanitize_text(&report.dashboard_updated_at)
        ),
        format!(
            "upgrade_history_available: {}",
            yes_no(report.upgrades.is_some())
        ),
        format!(
            "upgrade_count: {}",
            report
                .upgrade_count
                .map_or_else(|| "-".to_string(), |count| count.to_string())
        ),
        format!("certified: {}", yes_no(report.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(report.point_in_time_guaranteed)
        ),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
    ];

    if !report.controllers.is_empty() {
        lines.push(String::new());
        lines.push("controllers:".to_string());
        lines.extend(
            report
                .controllers
                .iter()
                .map(|controller| format!("  {controller}")),
        );
    }

    if let Some(latest) = report
        .upgrades
        .as_ref()
        .and_then(|upgrades| upgrades.first())
    {
        lines.push(String::new());
        lines.push("latest_upgrade:".to_string());
        lines.push(format!("  proposal_id: {}", latest.proposal_id));
        lines.push(format!(
            "  executed_timestamp_seconds: {}",
            latest.executed_timestamp_seconds
        ));
        lines.push(format!("  module_hash: {}", latest.module_hash));
    }

    lines.join("\n")
}

fn text_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        sanitize_text(value)
    }
}
