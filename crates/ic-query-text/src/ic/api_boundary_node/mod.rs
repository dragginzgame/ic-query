//! Module: ic::api_boundary_node
//!
//! Responsibility: render certified API boundary-node reports as human-facing text.
//! Does not own: JSON serialization, state-tree collection, or report validation.
//! Boundary: keeps certificate provenance separate from the following node table.

use crate::text_value::table;
use crate::text_value::yes_no;
use ic_host_text::table::Alignment;

use ic_query::ic::IcApiBoundaryNodeReport;

/// Render one complete certified API boundary-node report as human-facing text.
#[must_use]
pub fn ic_api_boundary_node_report_text(report: &IcApiBoundaryNodeReport) -> String {
    let provenance = &report.provenance;
    let mut lines = Vec::from(crate::text_value::report_fields!(report;
        schema_version = provenance.schema_version,
        network = provenance.network,
        authority = provenance.authority,
        source_endpoint = provenance.source_endpoint,
        effective_canister_id = provenance.effective_canister_id,
        fetched_at_unix_seconds = provenance.fetched_at_unix_seconds,
        fetched_at = provenance.fetched_at,
        fetched_by = provenance.fetched_by,
        certificate_time_unix_seconds = provenance.certificate_time_unix_seconds,
        certificate_time = provenance.certificate_time,
        certified = yes_no(provenance.certified),
        point_in_time_guaranteed = yes_no(provenance.point_in_time_guaranteed),
        node_count = report.rows.len(),
    ));

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
