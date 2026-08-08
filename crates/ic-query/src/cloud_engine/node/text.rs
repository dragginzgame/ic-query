//! Module: cloud_engine::node::text
//!
//! Responsibility: render CloudEngine Type4 node reports as human-facing text.
//! Does not own: JSON serialization, source validation, live calls, or process output.
//! Boundary: separates off-chain Dashboard provenance from node tables.

use super::{CloudEngineNodeInfoReport, CloudEngineNodeListReport, CloudEngineNodeRow};
use crate::{
    ic::IcDashboardReportProvenance,
    table::{ColumnAlign, render_table},
    text_value::{optional_text, sanitize_text, yes_no},
};

/// Render one complete explicitly scoped Type4 node list.
#[must_use]
pub fn cloud_engine_node_list_report_text(report: &CloudEngineNodeListReport) -> String {
    let mut lines = provenance_lines(&report.provenance);
    lines.extend([
        format!("node_reward_type: {}", report.node_reward_type),
        format!("included_statuses: {}", report.included_statuses.join(",")),
        format!(
            "requested_node_provider_id: {}",
            optional_text(report.requested_node_provider_id.as_ref())
        ),
        format!("node_count: {}", report.node_count),
        format!("status_up: {}", report.status_counts.up),
        format!("status_down: {}", report.status_counts.down),
        format!("status_disabled: {}", report.status_counts.disabled),
        format!("status_degraded: {}", report.status_counts.degraded),
        format!("status_unknown: {}", report.status_counts.unknown),
        format!("node_provider_count: {}", report.node_provider_count),
        format!(
            "cloud_engine_subnet_count: {}",
            report.cloud_engine_subnet_count
        ),
        format!(
            "unassigned_cloud_engine_node_count: {}",
            report.unassigned_cloud_engine_node_count
        ),
    ]);
    if !report.nodes.is_empty() {
        lines.push(String::new());
        lines.push("CloudEngine nodes".to_string());
        lines.push(node_table(&report.nodes));
    }
    lines.join("\n")
}

/// Render one exact Type4 node observation.
#[must_use]
pub fn cloud_engine_node_info_report_text(report: &CloudEngineNodeInfoReport) -> String {
    let node = &report.node;
    let mut lines = provenance_lines(&report.provenance);
    lines.extend([
        format!("node_id: {}", node.node_id),
        format!("status: {}", sanitize_text(&node.status)),
        format!("node_type: {}", sanitize_text(&node.node_type)),
        format!(
            "node_reward_type: {}",
            sanitize_text(&node.node_reward_type)
        ),
        format!("node_provider_id: {}", node.node_provider_id),
        format!(
            "node_provider_name: {}",
            sanitize_text(&node.node_provider_name)
        ),
        format!("node_operator_id: {}", node.node_operator_id),
        format!(
            "cloud_engine_subnet_id: {}",
            optional_text(node.cloud_engine_subnet_id.as_ref())
        ),
        format!("subnet_id: {}", optional_text(node.subnet_id.as_ref())),
        format!("data_center_id: {}", sanitize_text(&node.data_center_id)),
        format!(
            "data_center_name: {}",
            sanitize_text(&node.data_center_name)
        ),
        format!("owner: {}", sanitize_text(node.owner.trim())),
        format!("region: {}", sanitize_text(&node.region)),
        format!(
            "guestos_version: {}",
            optional_text(node.guestos_version.as_ref())
        ),
        format!(
            "guestos_tee_active: {}",
            optional_bool(node.guestos_tee_active)
        ),
        format!("ip_address: {}", optional_text(node.ip_address.as_ref())),
        format!(
            "ipv4_connectivity_status: {}",
            optional_bool(node.ipv4_connectivity_status)
        ),
        format!(
            "node_hardware_generation: {}",
            optional_text(node.node_hardware_generation.as_ref())
        ),
        format!("alert_name: {}", optional_text(node.alert_name.as_ref())),
    ]);
    lines.join("\n")
}

fn provenance_lines(provenance: &IcDashboardReportProvenance) -> Vec<String> {
    vec![
        format!("schema_version: {}", provenance.schema_version),
        format!("network: {}", sanitize_text(&provenance.network)),
        format!("authority: {}", sanitize_text(&provenance.authority)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&provenance.source_endpoint)
        ),
        format!("fetched_at: {}", sanitize_text(&provenance.fetched_at)),
        format!("fetched_by: {}", sanitize_text(&provenance.fetched_by)),
        format!("certified: {}", yes_no(provenance.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(provenance.point_in_time_guaranteed)
        ),
    ]
}

fn node_table(nodes: &[CloudEngineNodeRow]) -> String {
    let rows = nodes
        .iter()
        .map(|node| {
            [
                node.node_id.clone(),
                sanitize_text(&node.status),
                node.cloud_engine_subnet_id
                    .as_ref()
                    .map_or_else(|| "-".to_string(), Clone::clone),
                node.node_provider_id.clone(),
                node.node_operator_id.clone(),
                sanitize_text(&node.data_center_id),
                node.guestos_version
                    .as_ref()
                    .map_or_else(|| "-".to_string(), Clone::clone),
            ]
        })
        .collect::<Vec<_>>();
    render_table(
        &[
            "Node",
            "Status",
            "CE Subnet",
            "Provider",
            "Operator",
            "DC",
            "GuestOS",
        ],
        &rows,
        &[
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
            ColumnAlign::Left,
        ],
    )
}

fn optional_bool(value: Option<bool>) -> &'static str {
    value.map_or("-", yes_no)
}
