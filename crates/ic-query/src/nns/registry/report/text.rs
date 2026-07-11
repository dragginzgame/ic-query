use super::model::NnsRegistryVersionReport;
use crate::text_value::sanitize_text;

#[must_use]
pub fn nns_registry_version_report_text(report: &NnsRegistryVersionReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
    ]
    .join("\n")
}
