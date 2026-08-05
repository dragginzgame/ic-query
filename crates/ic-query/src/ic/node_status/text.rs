//! Module: ic::node_status::text
//!
//! Responsibility: human-readable observed node, Subnet, provider, and refresh reports.
//! Does not own: source collection, cache policy, projection, or JSON output.
//! Boundary: keeps compact operational display separate from raw report fields.

#[cfg(feature = "host")]
use super::IcNodeStatusRefreshReport;
use super::{
    IcNodeAssignmentStatusCounts, IcNodeProviderStatusReport, IcNodeStatusCounts,
    IcNodeStatusObservation, IcNodeStatusReport, IcNodeStatusRow, IcSubnetStatusReport,
};
use crate::{
    table::{ColumnAlign, render_table},
    text_value::{sanitize_text, yes_no},
};

const COMPACT_PRINCIPAL_CHARS: usize = 8;

/// Render a human-readable node-level operational status report.
#[must_use]
pub fn ic_node_status_report_text(report: &IcNodeStatusReport) -> String {
    let mut sections = observation_lines(&report.observation);
    sections.push(counts_line("nodes", &report.counts.statuses));
    sections.push(assignment_status_line(&report.counts.assignment_statuses));
    push_table_section(&mut sections, render_node_table(&report.nodes));
    sections.join("\n")
}

/// Render a human-readable Subnet operational status report.
#[must_use]
pub fn ic_subnet_status_report_text(report: &IcSubnetStatusReport) -> String {
    let mut sections = observation_lines(&report.observation);
    sections.push(format!(
        "subnets: total={} attention={} returned={} assigned_nodes={}",
        report.subnet_count,
        report.attention_subnet_count,
        report.returned_subnet_count,
        report.assigned_node_count
    ));
    let headers = [
        "SUBNET",
        "NODES",
        "UP",
        "DEGRADED",
        "DOWN",
        "DISABLED",
        "UNKNOWN",
        "F",
        "+DOWN >F",
        "+NON-UP >F",
    ];
    let rows = report
        .subnets
        .iter()
        .map(|row| {
            [
                compact(&row.subnet_id),
                row.statuses.total.to_string(),
                row.statuses.up.to_string(),
                row.statuses.degraded.to_string(),
                row.statuses.down.to_string(),
                row.statuses.disabled.to_string(),
                row.statuses.unknown.to_string(),
                row.fault_tolerance_node_count.to_string(),
                row.additional_down_nodes_to_exceed_fault_tolerance
                    .to_string(),
                row.additional_non_up_nodes_to_exceed_fault_tolerance
                    .to_string(),
            ]
        })
        .collect::<Vec<_>>();
    push_table_section(
        &mut sections,
        render_table(
            &headers,
            &rows,
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
            ],
        ),
    );
    let non_up = report
        .subnets
        .iter()
        .flat_map(|subnet| subnet.non_up_nodes.iter())
        .cloned()
        .collect::<Vec<_>>();
    if !non_up.is_empty() {
        sections.push(String::new());
        sections.push("non-up node evidence:".to_string());
        sections.push(render_node_table(&non_up));
    }
    sections.join("\n")
}

/// Render a human-readable node-provider operational status report.
#[must_use]
pub fn ic_node_provider_status_report_text(report: &IcNodeProviderStatusReport) -> String {
    let mut sections = observation_lines(&report.observation);
    sections.push(format!(
        "node_providers: total={} attention={} returned={}",
        report.provider_count, report.attention_provider_count, report.returned_provider_count
    ));
    let headers = [
        "NODE PROVIDER",
        "NAME",
        "NODES",
        "UP",
        "DEGRADED",
        "DOWN",
        "DISABLED",
        "UNKNOWN",
        "ASSIGNED UP/NON-UP",
        "UNASSIGNED UP/NON-UP",
        "API BN UP/NON-UP",
        "UNKNOWN ASN UP/NON-UP",
    ];
    let rows = report
        .providers
        .iter()
        .map(|row| {
            [
                compact(&row.node_provider_id),
                sanitize_text(&row.node_provider_name),
                row.counts.statuses.total.to_string(),
                row.counts.statuses.up.to_string(),
                row.counts.statuses.degraded.to_string(),
                row.counts.statuses.down.to_string(),
                row.counts.statuses.disabled.to_string(),
                row.counts.statuses.unknown.to_string(),
                up_non_up(&row.counts.assignment_statuses.assigned),
                up_non_up(&row.counts.assignment_statuses.unassigned),
                up_non_up(&row.counts.assignment_statuses.api_boundary),
                up_non_up(&row.counts.assignment_statuses.unknown),
            ]
        })
        .collect::<Vec<_>>();
    push_table_section(
        &mut sections,
        render_table(
            &headers,
            &rows,
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
            ],
        ),
    );
    sections.join("\n")
}

fn push_table_section(sections: &mut Vec<String>, table: String) {
    sections.push(String::new());
    sections.push(table);
}

/// Render a human-readable forced node-status cache refresh report.
#[cfg(feature = "host")]
#[must_use]
pub fn ic_node_status_refresh_report_text(report: &IcNodeStatusRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        counts_line("nodes", &report.counts.statuses),
        assignment_status_line(&report.counts.assignment_statuses),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("cache_path: {}", sanitize_text(&report.cache_path)),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
    ]
    .join("\n")
}

fn observation_lines(observation: &IcNodeStatusObservation) -> Vec<String> {
    let mut lines = vec![format!(
        "observed node status: network={} fetched_at={} source={} certified={} point_in_time={}",
        sanitize_text(&observation.source.network),
        sanitize_text(&observation.source.fetched_at),
        sanitize_text(&observation.source.source_endpoint),
        yes_no(observation.source.certified),
        yes_no(observation.source.point_in_time_guaranteed)
    )];
    lines.push(format!(
        "scope: {} cloud_engine_nodes_included={}",
        observation.scope.as_str(),
        yes_no(observation.cloud_engine_nodes_included)
    ));
    if let Some(cache) = &observation.cache {
        lines.push(format!(
            "cache: fresh={} age={}s stale_after={}s path={}",
            yes_no(cache.cache_fresh),
            cache.age_seconds,
            cache.stale_after_seconds,
            sanitize_text(&cache.cache_path)
        ));
    }
    lines
}

fn counts_line(label: &str, counts: &IcNodeStatusCounts) -> String {
    format!(
        "{label}: total={} up={} degraded={} down={} disabled={} unknown={} non_up={}",
        counts.total,
        counts.up,
        counts.degraded,
        counts.down,
        counts.disabled,
        counts.unknown,
        counts.non_up()
    )
}

fn assignment_status_line(counts: &IcNodeAssignmentStatusCounts) -> String {
    format!(
        "assignments (total/up/non_up): assigned={}/{}/{} unassigned={}/{}/{} api_boundary={}/{}/{} unknown={}/{}/{}",
        counts.assigned.total,
        counts.assigned.up,
        counts.assigned.non_up(),
        counts.unassigned.total,
        counts.unassigned.up,
        counts.unassigned.non_up(),
        counts.api_boundary.total,
        counts.api_boundary.up,
        counts.api_boundary.non_up(),
        counts.unknown.total,
        counts.unknown.up,
        counts.unknown.non_up()
    )
}

fn up_non_up(counts: &IcNodeStatusCounts) -> String {
    format!("{}/{}", counts.up, counts.non_up())
}

fn render_node_table(nodes: &[IcNodeStatusRow]) -> String {
    let headers = ["NODE", "STATUS", "TYPE", "SUBNET", "PROVIDER", "ALERT"];
    let rows = nodes
        .iter()
        .map(|node| {
            [
                compact(&node.node_id),
                sanitize_text(&node.status),
                sanitize_text(&node.node_type),
                node.subnet_id
                    .as_deref()
                    .map_or_else(|| "-".to_string(), compact),
                compact(&node.node_provider_id),
                node.alert_name
                    .as_deref()
                    .map_or_else(|| "-".to_string(), sanitize_text),
            ]
        })
        .collect::<Vec<_>>();
    render_table(
        &headers,
        &rows,
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
        ],
    )
}

fn compact(value: &str) -> String {
    sanitize_text(value)
        .chars()
        .take(COMPACT_PRINCIPAL_CHARS)
        .collect()
}
