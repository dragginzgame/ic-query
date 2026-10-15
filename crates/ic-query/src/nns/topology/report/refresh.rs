//! Module: nns::topology::report::refresh
//!
//! Responsibility: build NNS topology refresh reports from component refreshes.
//! Does not own: component refresh execution, cache writes, or text rendering.
//! Boundary: combines component refresh reports into one topology refresh report.

use super::{NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION, NnsTopologyRefreshReport};
use crate::{
    nns::{
        data_center::NnsDataCenterRefreshReport, node::NnsNodeRefreshReport,
        node_operator::NnsNodeOperatorRefreshReport, node_provider::NnsNodeProviderRefreshReport,
        topology::report::NnsTopologyRefreshRow,
    },
    subnet_catalog::SubnetCatalogRefreshReport,
};

///
/// NnsTopologyRefreshComponentReports
///
/// Component refresh reports combined into one NNS topology refresh result.
///

pub(in crate::nns::topology::report) struct NnsTopologyRefreshComponentReports {
    pub(in crate::nns::topology::report) subnet: SubnetCatalogRefreshReport,
    pub(in crate::nns::topology::report) node: NnsNodeRefreshReport,
    pub(in crate::nns::topology::report) node_provider: NnsNodeProviderRefreshReport,
    pub(in crate::nns::topology::report) node_operator: NnsNodeOperatorRefreshReport,
    pub(in crate::nns::topology::report) data_center: NnsDataCenterRefreshReport,
}

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

macro_rules! inventory_refresh_row {
    ($report:expr, $source:literal, $count:ident) => {{
        let report = $report;
        NnsTopologyRefreshRow {
            source: $source.to_string(),
            cache_path: report.cache_path,
            refresh_lock_path: report.refresh_lock_path,
            registry_version: report.registry_version,
            fetched_at: report.fetched_at,
            source_endpoint: report.source_endpoint,
            fetched_by: report.fetched_by,
            dry_run: report.dry_run,
            wrote_cache: report.wrote_cache,
            replaced_existing_cache: report.replaced_existing_cache,
            item_count: report.$count,
        }
    }};
}

fn refresh_rows_from_reports(
    reports: NnsTopologyRefreshComponentReports,
) -> Vec<NnsTopologyRefreshRow> {
    vec![
        refresh_row_from_subnet_report(reports.subnet),
        inventory_refresh_row!(reports.node, "nodes", node_count),
        inventory_refresh_row!(reports.node_provider, "node_providers", node_provider_count),
        inventory_refresh_row!(reports.node_operator, "node_operators", node_operator_count),
        inventory_refresh_row!(reports.data_center, "data_centers", data_center_count),
    ]
}

fn refresh_row_from_subnet_report(report: SubnetCatalogRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "subnet_catalog".to_string(),
        cache_path: report.catalog_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoints.join(","),
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_catalog,
        replaced_existing_cache: report.replaced_existing_catalog,
        item_count: report.subnet_count,
    }
}
