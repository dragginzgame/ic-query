use super::model::NnsRegistryVersionReport;

#[must_use]
pub fn nns_registry_version_report_text(report: &NnsRegistryVersionReport) -> String {
    [
        format!("network: {}", report.network),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
        format!("fetched_by: {}", report.fetched_by),
    ]
    .join("\n")
}
