use super::model::NnsRegistryVersionReport;
use crate::{human_quantity::byte_count_text, text_value::sanitize_text};

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
        "assurance: certified".to_string(),
        format!(
            "certificate_verified: {}",
            report.certification.certificate_verified
        ),
        format!(
            "certificate_time: {}",
            sanitize_text(&report.certification.certificate_time)
        ),
        format!("root_key_digest: {}", report.certification.root_key_digest),
        format!(
            "certificate_bytes: {}",
            byte_count_text(report.certification.certificate_bytes as u128)
        ),
        format!(
            "hash_tree_bytes: {}",
            byte_count_text(report.certification.hash_tree_bytes as u128)
        ),
    ]
    .join("\n")
}
