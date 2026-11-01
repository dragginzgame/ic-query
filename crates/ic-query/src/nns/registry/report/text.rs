use ic_host_text::escape::terminal_text;

use super::model::NnsRegistryVersionReport;
use crate::human_quantity::byte_count_text;

#[must_use]
pub fn nns_registry_version_report_text(report: &NnsRegistryVersionReport) -> String {
    [
        format!("network: {}", terminal_text(&report.provenance.network)),
        format!(
            "registry_canister_id: {}",
            report.provenance.registry_canister_id
        ),
        format!("registry_version: {}", report.provenance.registry_version),
        format!(
            "fetched_at: {}",
            terminal_text(&report.provenance.fetched_at)
        ),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.provenance.source_endpoint)
        ),
        format!(
            "fetched_by: {}",
            terminal_text(&report.provenance.fetched_by)
        ),
        "assurance: certified".to_string(),
        format!(
            "certificate_verified: {}",
            report.certification.certificate_verified
        ),
        format!(
            "certificate_time: {}",
            terminal_text(&report.certification.certificate_time)
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
