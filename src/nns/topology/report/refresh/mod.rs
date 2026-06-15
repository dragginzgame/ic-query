mod components;
mod rows;

pub(in crate::nns::topology::report) use components::NnsTopologyRefreshComponentReports;

use super::{NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION, NnsTopologyRefreshReport};
use rows::refresh_rows_from_reports;

pub(super) fn topology_refresh_report_from_reports(
    network: String,
    source_endpoint: String,
    dry_run: bool,
    reports: NnsTopologyRefreshComponentReports,
) -> NnsTopologyRefreshReport {
    let components = refresh_rows_from_reports(reports);
    let wrote_cache_count = components
        .iter()
        .filter(|component| component.wrote_cache)
        .count();
    let replaced_existing_cache_count = components
        .iter()
        .filter(|component| component.replaced_existing_cache)
        .count();

    NnsTopologyRefreshReport {
        schema_version: NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        dry_run,
        component_count: components.len(),
        wrote_cache_count,
        replaced_existing_cache_count,
        components,
    }
}
