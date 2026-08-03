use crate::{
    nns::{node::report::NnsNodeInfoReport, render::text_or_dash},
    text_value::sanitize_text,
};

#[must_use]
pub fn nns_node_info_report_text(report: &NnsNodeInfoReport) -> String {
    [
        format!("input: {}", sanitize_text(&report.input)),
        format!("resolved_from: {}", sanitize_text(&report.resolved_from)),
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
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("network: {}", sanitize_text(&report.network)),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
    ]
    .join("\n")
}
