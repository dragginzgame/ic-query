//! Module: ic::api_boundary_node::text
//!
//! Responsibility: render certified API boundary-node reports as human-facing text.
//! Does not own: JSON serialization, state-tree collection, or report validation.
//! Boundary: keeps certificate provenance separate from the following node table.

use super::IcApiBoundaryNodeReport;
use crate::{
    table::{ColumnAlign, render_table},
    text_value::sanitize_text,
};

/// Render one complete certified API boundary-node report as human-facing text.
#[must_use]
pub fn ic_api_boundary_node_report_text(report: &IcApiBoundaryNodeReport) -> String {
    let provenance = &report.provenance;
    let mut lines = vec![
        format!("schema_version: {}", provenance.schema_version),
        format!("network: {}", sanitize_text(&provenance.network)),
        format!("authority: {}", sanitize_text(&provenance.authority)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&provenance.source_endpoint)
        ),
        format!(
            "effective_canister_id: {}",
            sanitize_text(&provenance.effective_canister_id)
        ),
        format!(
            "fetched_at_unix_seconds: {}",
            provenance.fetched_at_unix_seconds
        ),
        format!("fetched_at: {}", sanitize_text(&provenance.fetched_at)),
        format!("fetched_by: {}", sanitize_text(&provenance.fetched_by)),
        format!(
            "certificate_time_unix_seconds: {}",
            provenance.certificate_time_unix_seconds
        ),
        format!(
            "certificate_time: {}",
            sanitize_text(&provenance.certificate_time)
        ),
        format!("certified: {}", yes_no(provenance.certified)),
        format!(
            "point_in_time_guaranteed: {}",
            yes_no(provenance.point_in_time_guaranteed)
        ),
        format!("node_count: {}", report.node_count),
    ];

    if !report.rows.is_empty() {
        let rows = report
            .rows
            .iter()
            .map(|row| {
                [
                    row.node_id.clone(),
                    row.domain.clone(),
                    row.ipv4_address.clone().unwrap_or_else(|| "-".to_string()),
                    row.ipv6_address.clone(),
                ]
            })
            .collect::<Vec<_>>();
        lines.push(String::new());
        lines.push("api_boundary_nodes:".to_string());
        lines.push(render_table(
            &["NODE ID", "DOMAIN", "IPV4", "IPV6"],
            &rows,
            &[
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }

    lines.join("\n")
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
