//! Module: ic::text
//!
//! Responsibility: render IC Dashboard canister reports as human-facing text.
//! Does not own: REST calls, report construction, JSON serialization, or command output.
//! Boundary: keeps raw values intact in JSON while making nullable text fields readable.

use crate::{
    ic::{IcCanisterCountReport, IcCanisterFilters, IcCanisterPageReport, IcCanisterReport},
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

/// Render one official Dashboard canister-count report as human-facing text.
#[must_use]
pub fn ic_canister_count_report_text(report: &IcCanisterCountReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("authority: {}", sanitize_text(&report.authority)),
        format!("total: {}", report.total),
    ];
    append_filters(&mut lines, &report.filters);
    lines.extend([
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
    ]);
    lines.join("\n")
}

/// Render one bounded official Dashboard canister page as human-facing text.
#[must_use]
pub fn ic_canister_page_report_text(report: &IcCanisterPageReport) -> String {
    let mut lines = vec![
        format!("network: {}", sanitize_text(&report.network)),
        format!("authority: {}", sanitize_text(&report.authority)),
        format!("returned_count: {}", report.returned_count),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "after: {}",
            report.after.as_deref().map_or("-", |value| value)
        ),
        format!(
            "before: {}",
            report.before.as_deref().map_or("-", |value| value)
        ),
        format!(
            "previous_cursor: {}",
            report.previous_cursor.as_deref().map_or("-", |value| value)
        ),
        format!(
            "next_cursor: {}",
            report.next_cursor.as_deref().map_or("-", |value| value)
        ),
    ];
    append_filters(&mut lines, &report.filters);
    lines.extend([
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
    ]);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("canisters:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  name={}  type={}  subnet={}  controllers={}  language={}  updated={}",
                row.canister_id,
                text_or_dash(&row.name),
                row.canister_type
                    .as_deref()
                    .map_or_else(|| "-".to_string(), text_or_dash),
                row.subnet_id,
                row.controllers.len(),
                text_or_dash(&row.language),
                sanitize_text(&row.dashboard_updated_at),
            )
        }));
    }
    lines.join("\n")
}

fn append_filters(lines: &mut Vec<String>, filters: &IcCanisterFilters) {
    if let Some(has_name) = filters.has_name {
        lines.push(format!("filter.has_name: {}", yes_no(has_name)));
    }
    if let Some(subnet_id) = filters.subnet_id.as_deref() {
        lines.push(format!("filter.subnet_id: {subnet_id}"));
    }
    if let Some(controller_id) = filters.controller_id.as_deref() {
        lines.push(format!("filter.controller_id: {controller_id}"));
    }
    if !filters.languages.is_empty() {
        lines.push(format!(
            "filter.languages: {}",
            filters
                .languages
                .iter()
                .map(|value| sanitize_text(value))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if !filters.canister_types.is_empty() {
        lines.push(format!(
            "filter.canister_types: {}",
            filters
                .canister_types
                .iter()
                .map(|value| sanitize_text(value))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if let Some(query) = filters.query.as_deref() {
        lines.push(format!("filter.query: {}", sanitize_text(query)));
    }
}

fn text_or_dash(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        sanitize_text(value)
    }
}
