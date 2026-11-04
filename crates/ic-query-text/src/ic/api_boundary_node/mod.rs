//! Module: ic::api_boundary_node
//!
//! Responsibility: render certified API boundary-node reports as human-facing text.
//! Does not own: JSON serialization, state-tree collection, or report validation.
//! Boundary: keeps certificate provenance separate from the following node table.

use crate::text_value::field;
use crate::text_value::table;
use crate::text_value::yes_no;
use ic_host_text::table::Alignment;

use ic_query::ic::IcApiBoundaryNodeReport;

/// Render one complete certified API boundary-node report as human-facing text.
#[must_use]
pub fn ic_api_boundary_node_report_text(report: &IcApiBoundaryNodeReport) -> String {
    let provenance = &report.provenance;
    let mut lines = vec![
        field("schema_version", &(provenance.schema_version)),
        field("network", &provenance.network),
        field("authority", &provenance.authority),
        field("source_endpoint", &provenance.source_endpoint),
        field("effective_canister_id", &provenance.effective_canister_id),
        field(
            "fetched_at_unix_seconds",
            &(provenance.fetched_at_unix_seconds),
        ),
        field("fetched_at", &provenance.fetched_at),
        field("fetched_by", &provenance.fetched_by),
        field(
            "certificate_time_unix_seconds",
            &(provenance.certificate_time_unix_seconds),
        ),
        field("certificate_time", &provenance.certificate_time),
        field("certified", &(yes_no(provenance.certified))),
        field(
            "point_in_time_guaranteed",
            &(yes_no(provenance.point_in_time_guaranteed)),
        ),
        field("node_count", &(report.rows.len())),
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
        lines.push(table(
            &["NODE ID", "DOMAIN", "IPV4", "IPV6"],
            &rows,
            &[Alignment::Left; 4],
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests;
