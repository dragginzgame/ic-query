use ic_host_text::escape::terminal_text;

use crate::human_quantity::byte_count_text;
use ic_query::nns::registry::NnsRegistryVersionReport;

#[must_use]
/// Render the NNS registry version report as human-facing text.
pub fn nns_registry_version_report_text(report: &NnsRegistryVersionReport) -> String {
    let [
        registry_id,
        version,
        network,
        fetched_at,
        endpoint,
        fetched_by,
    ] = crate::nns::render::registry_provenance_lines(&report.provenance);
    [
        network,
        registry_id,
        version,
        fetched_at,
        endpoint,
        fetched_by,
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
