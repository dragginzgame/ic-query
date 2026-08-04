//! Module: ic::text
//!
//! Responsibility: render IC Dashboard reports as human-facing text.
//! Does not own: REST calls, report construction, JSON serialization, or command output.
//! Boundary: keeps raw values intact in JSON while making nullable text fields readable.

use crate::{
    human_quantity::decimal_cycle_rate_text,
    ic::{
        IcBoundaryNodeDataCentersReport, IcCanisterCountReport, IcCanisterFilters,
        IcCanisterPageReport, IcCanisterReport, IcDailyStatsReport, IcDashboardReportProvenance,
        IcIcrcHolderCountReport, IcIcrcTotalSupplyReport, IcMetricKind, IcMetricReport,
    },
    text_value::{sanitize_text, yes_no},
};

/// Render one official boundary-node data-center report as human-facing text.
#[must_use]
pub fn ic_boundary_node_data_centers_report_text(
    report: &IcBoundaryNodeDataCentersReport,
) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("data_center_count: {}", report.data_center_count),
        format!("total_node_count: {}", report.total_node_count),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("boundary_node_data_centers:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  name={}  owner={}  region={}  latitude={}  longitude={}  nodes={}",
                sanitize_text(&row.dc_id),
                sanitize_text(&row.name),
                sanitize_text(&row.owner),
                sanitize_text(&row.region),
                sanitize_text(&row.latitude),
                sanitize_text(&row.longitude),
                sanitize_text(&row.total_nodes),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard daily-statistics report as human-facing text.
#[must_use]
pub fn ic_daily_stats_report_text(report: &IcDailyStatsReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("returned_day_count: {}", report.returned_day_count),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.rows.is_empty() {
        lines.push(String::new());
        lines.push("daily_stats:".to_string());
        lines.extend(report.rows.iter().map(|row| {
            format!(
                "  {}  timestamp={}  avg_total={}  avg_update={}  avg_query={}  max_total={}  max_update={}  max_query={}  blocks_avg={}",
                sanitize_text(&row.day),
                row.timestamp_unix_secs,
                sanitize_text(&row.average_transactions_per_second),
                sanitize_text(&row.average_update_transactions_per_second),
                sanitize_text(&row.average_query_transactions_per_second),
                sanitize_text(&row.max_total_transactions_per_second),
                sanitize_text(&row.max_update_transactions_per_second),
                sanitize_text(&row.max_query_transactions_per_second),
                sanitize_text(&row.blocks_per_second_average),
            )
        }));
    }
    lines.join("\n")
}

/// Render one bounded official Dashboard metric report as human-facing text.
#[must_use]
pub fn ic_metric_report_text(report: &IcMetricReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!("metric: {}", report.query.metric),
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("step_secs: {}", report.query.step_secs),
        format!("returned_series_count: {}", report.returned_series_count),
        format!(
            "returned_observation_count: {}",
            report.returned_observation_count
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    for series in &report.series {
        lines.push(String::new());
        lines.push(format!("{}:", sanitize_text(&series.name)));
        lines.extend(series.observations.iter().map(|observation| {
            format!(
                "  {}  {}",
                observation.timestamp_unix_secs,
                metric_observation_text(report.query.metric, &observation.value)
            )
        }));
    }
    lines.join("\n")
}

/// Render one official ICRC holder-count report as human-facing text.
#[must_use]
pub fn icrc_holder_count_report_text(report: &IcIcrcHolderCountReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("total: {}", report.total),
    ]);
    append_report_footer(&mut lines, &report.provenance);
    lines.join("\n")
}

/// Render one bounded official ICRC total-supply series as human-facing text.
#[must_use]
pub fn icrc_total_supply_report_text(report: &IcIcrcTotalSupplyReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
        format!(
            "ledger_canister_id: {}",
            sanitize_text(&report.ledger_canister_id)
        ),
        format!("start_unix_secs: {}", report.query.start_unix_secs),
        format!("end_unix_secs: {}", report.query.end_unix_secs),
        format!("step_secs: {}", report.query.step_secs),
        format!(
            "requested_observation_limit: {}",
            report.requested_observation_limit
        ),
        format!(
            "returned_observation_count: {}",
            report.returned_observation_count
        ),
    ]);
    append_report_footer(&mut lines, &report.provenance);

    if !report.observations.is_empty() {
        lines.push(String::new());
        lines.push("total_supply_base_units:".to_string());
        lines.extend(report.observations.iter().map(|observation| {
            format!(
                "  {}  {}",
                observation.timestamp_unix_secs,
                sanitize_text(&observation.total_supply_base_units)
            )
        }));
    }
    lines.join("\n")
}

fn metric_observation_text(metric: IcMetricKind, value: &str) -> String {
    if metric == IcMetricKind::CycleBurnRate {
        decimal_cycle_rate_text(value)
    } else {
        sanitize_text(value)
    }
}

/// Render one official Dashboard canister report as human-facing text.
#[must_use]
pub fn ic_canister_report_text(report: &IcCanisterReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
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
    ]);
    append_report_footer(&mut lines, &report.provenance);

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
    let mut lines = report_header(&report.provenance);
    lines.push(format!("total: {}", report.total));
    append_filters(&mut lines, &report.filters);
    append_report_footer(&mut lines, &report.provenance);
    lines.join("\n")
}

/// Render one bounded official Dashboard canister page as human-facing text.
#[must_use]
pub fn ic_canister_page_report_text(report: &IcCanisterPageReport) -> String {
    let mut lines = report_header(&report.provenance);
    lines.extend([
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
    ]);
    append_filters(&mut lines, &report.filters);
    append_report_footer(&mut lines, &report.provenance);

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

fn report_header(provenance: &IcDashboardReportProvenance) -> Vec<String> {
    vec![
        format!("network: {}", sanitize_text(&provenance.network)),
        format!("authority: {}", sanitize_text(&provenance.authority)),
    ]
}

fn append_report_footer(lines: &mut Vec<String>, provenance: &IcDashboardReportProvenance) {
    lines.extend([
        format!("certified: {}", yes_no(provenance.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(provenance.point_in_time_guaranteed)
        ),
        format!("fetched_at: {}", sanitize_text(&provenance.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&provenance.source_endpoint)
        ),
    ]);
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
