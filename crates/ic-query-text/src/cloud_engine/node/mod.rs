//! Module: cloud_engine::node
//!
//! Responsibility: render CloudEngine Type4 node reports as human-facing text.
//! Does not own: JSON serialization, source validation, live calls, or process output.
//! Boundary: separates off-chain Dashboard provenance from node tables.

use crate::text_value::field;
use crate::text_value::table;
use ic_host_text::escape::terminal_text;
use ic_host_text::table::Alignment;

use crate::ic::dashboard_provenance_lines;
use crate::text_value::optional_bool_text;
use crate::text_value::optional_text;
use ic_query::cloud_engine::CloudEngineNodeInfoReport;
use ic_query::cloud_engine::CloudEngineNodeListReport;
use ic_query::cloud_engine::CloudEngineNodeRow;

/// Render one complete explicitly scoped Type4 node list.
#[must_use]
pub fn cloud_engine_node_list_report_text(report: &CloudEngineNodeListReport) -> String {
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([
        field("node_reward_type", &(report.node_reward_type)),
        field("included_statuses", &(report.included_statuses.join(","))),
        field(
            "requested_node_provider_id",
            &(optional_text(report.requested_node_provider_id.as_ref())),
        ),
        field("node_count", &(report.nodes.len())),
        field("status_up", &(report.status_counts.up)),
        field("status_down", &(report.status_counts.down)),
        field("status_disabled", &(report.status_counts.disabled)),
        field("status_degraded", &(report.status_counts.degraded)),
        field("status_unknown", &(report.status_counts.unknown)),
        field("node_provider_count", &(report.node_provider_count)),
        field(
            "cloud_engine_subnet_count",
            &(report.cloud_engine_subnet_count),
        ),
        field(
            "unassigned_cloud_engine_node_count",
            &(report.unassigned_cloud_engine_node_count),
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
    let mut lines = dashboard_provenance_lines(&report.provenance);
    lines.extend([
        field("node_id", &(node.node_id)),
        field("status", &node.status),
        field("node_type", &node.node_type),
        field("node_reward_type", &node.node_reward_type),
        field("node_provider_id", &(node.node_provider_id)),
        field("node_provider_name", &node.node_provider_name),
        field("node_operator_id", &(node.node_operator_id)),
        field(
            "cloud_engine_subnet_id",
            &(optional_text(node.cloud_engine_subnet_id.as_ref())),
        ),
        field("subnet_id", &(optional_text(node.subnet_id.as_ref()))),
        field("data_center_id", &node.data_center_id),
        field("data_center_name", &node.data_center_name),
        field("owner", &(node.owner.trim())),
        field("region", &node.region),
        field(
            "guestos_version",
            &(optional_text(node.guestos_version.as_ref())),
        ),
        field(
            "guestos_tee_active",
            &(optional_bool_text(node.guestos_tee_active)),
        ),
        field("ip_address", &(optional_text(node.ip_address.as_ref()))),
        field(
            "ipv4_connectivity_status",
            &(optional_bool_text(node.ipv4_connectivity_status)),
        ),
        field(
            "node_hardware_generation",
            &(optional_text(node.node_hardware_generation.as_ref())),
        ),
        field("alert_name", &(optional_text(node.alert_name.as_ref()))),
    ]);
    lines.join("\n")
}

fn node_table(nodes: &[CloudEngineNodeRow]) -> String {
    let rows = nodes
        .iter()
        .map(|node| {
            [
                node.node_id.clone(),
                terminal_text(&node.status).into_owned(),
                node.cloud_engine_subnet_id
                    .as_ref()
                    .map_or_else(|| "-".to_string(), Clone::clone),
                node.node_provider_id.clone(),
                node.node_operator_id.clone(),
                terminal_text(&node.data_center_id).into_owned(),
                node.guestos_version
                    .as_ref()
                    .map_or_else(|| "-".to_string(), Clone::clone),
            ]
        })
        .collect::<Vec<_>>();
    table(
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
        &[Alignment::Left; 7],
    )
}

#[cfg(test)]
mod tests;
