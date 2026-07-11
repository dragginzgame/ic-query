use crate::{
    subnet_catalog::SubnetCatalogRefreshReport,
    text_value::{sanitize_text, yes_no},
};

#[must_use]
pub fn subnet_catalog_refresh_report_text(report: &SubnetCatalogRefreshReport) -> String {
    [
        format!("network: {}", sanitize_text(&report.network)),
        format!("catalog_path: {}", sanitize_text(&report.catalog_path)),
        format!(
            "refresh_lock_path: {}",
            sanitize_text(&report.refresh_lock_path)
        ),
        format!("registry_canister_id: {}", report.registry_canister_id),
        format!("registry_version: {}", report.registry_version),
        format!("fetched_at: {}", sanitize_text(&report.fetched_at)),
        format!(
            "source_endpoint: {}",
            sanitize_text(&report.source_endpoint)
        ),
        format!("fetched_by: {}", sanitize_text(&report.fetched_by)),
        format!("dry_run: {}", yes_no(report.dry_run)),
        format!("wrote_catalog: {}", yes_no(report.wrote_catalog)),
        format!(
            "replaced_existing_catalog: {}",
            yes_no(report.replaced_existing_catalog)
        ),
        format!("subnet_count: {}", report.subnet_count),
        format!("routing_range_count: {}", report.routing_range_count),
    ]
    .join("\n")
}
