use crate::nns::{
    node_provider::report::NnsNodeProviderRefreshReport,
    render::{NnsLeafRefreshText, nns_leaf_refresh_report_text},
};

#[must_use]
pub fn nns_node_provider_refresh_report_text(report: &NnsNodeProviderRefreshReport) -> String {
    nns_leaf_refresh_report_text(NnsLeafRefreshText {
        network: &report.network,
        cache_path: &report.cache_path,
        refresh_lock_path: &report.refresh_lock_path,
        governance_canister_id: Some(&report.governance_canister_id),
        registry_canister_id: &report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: &report.fetched_at,
        source_endpoint: &report.source_endpoint,
        fetched_by: &report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        count_label: "node_provider_count",
        count: report.node_provider_count,
    })
}
