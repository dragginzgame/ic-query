use ic_host_text::escape::terminal_text;

use crate::nns::{node::report::NnsNodeInfoReport, render::text_or_dash};

#[must_use]
pub fn nns_node_info_report_text(report: &NnsNodeInfoReport) -> String {
    let mut lines = vec![
        format!("input: {}", terminal_text(&report.input)),
        format!("resolved_from: {}", terminal_text(&report.resolved_from)),
        format!("node_principal: {}", report.node_principal),
        format!(
            "node_operator_principal: {}",
            report.node_operator_principal
        ),
        format!(
            "node_provider_principal: {}",
            report.node_provider_principal
        ),
        format!("subnet_principal: {}", report.subnet_principal),
        format!("subnet_kind: {}", report.subnet_kind.as_str()),
        format!(
            "data_center_id: {}",
            text_or_dash(Some(&report.data_center_id))
        ),
    ];
    lines.extend(crate::nns::render::registry_provenance_lines(
        &report.registry_canister_id,
        report.registry_version,
        &report.network,
        &report.fetched_at,
        &report.source_endpoint,
        &report.fetched_by,
    ));
    lines.join("\n")
}
