use crate::nns::{
    data_center::report::NnsDataCenterRefreshReport,
    render::{NnsLeafRefreshText, nns_leaf_refresh_report_text},
};

#[must_use]
pub fn nns_data_center_refresh_report_text(report: &NnsDataCenterRefreshReport) -> String {
    nns_leaf_refresh_report_text(NnsLeafRefreshText {
        network: &report.network,
        cache_path: &report.cache_path,
        refresh_lock_path: &report.refresh_lock_path,
        governance_canister_id: None,
        registry_canister_id: &report.registry_canister_id,
        registry_version: report.registry_version,
        fetched_at: &report.fetched_at,
        source_endpoint: &report.source_endpoint,
        fetched_by: &report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        count_label: "data_center_count",
        count: report.data_center_count,
    })
}
