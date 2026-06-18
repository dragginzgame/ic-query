//! Module: nns::topology::report::refresh
//!
//! Responsibility: build NNS topology refresh reports from component refreshes.
//! Does not own: component refresh execution, cache writes, or text rendering.
//! Boundary: combines component refresh reports into one topology refresh report.

use super::{NNS_TOPOLOGY_REFRESH_REPORT_SCHEMA_VERSION, NnsTopologyRefreshReport};
use crate::{
    nns::{
        data_center::report::NnsDataCenterRefreshReport, node::report::NnsNodeRefreshReport,
        node_operator::report::NnsNodeOperatorRefreshReport,
        node_provider::report::NnsNodeProviderRefreshReport,
        topology::report::NnsTopologyRefreshRow,
    },
    subnet_catalog::SubnetCatalogRefreshReport,
};

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

fn refresh_rows_from_reports(
    reports: NnsTopologyRefreshComponentReports,
) -> Vec<NnsTopologyRefreshRow> {
    vec![
        refresh_row_from_subnet_report(reports.subnet),
        refresh_row_from_node_report(reports.node),
        refresh_row_from_node_provider_report(reports.node_provider),
        refresh_row_from_node_operator_report(reports.node_operator),
        refresh_row_from_data_center_report(reports.data_center),
    ]
}

fn refresh_row_from_subnet_report(report: SubnetCatalogRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "subnet_catalog".to_string(),
        cache_path: report.catalog_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_catalog,
        replaced_existing_cache: report.replaced_existing_catalog,
        item_count: report.subnet_count,
    }
}

fn refresh_row_from_node_report(report: NnsNodeRefreshReport) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "nodes".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.node_count,
    }
}

fn refresh_row_from_node_provider_report(
    report: NnsNodeProviderRefreshReport,
) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "node_providers".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.node_provider_count,
    }
}

fn refresh_row_from_node_operator_report(
    report: NnsNodeOperatorRefreshReport,
) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "node_operators".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.node_operator_count,
    }
}

fn refresh_row_from_data_center_report(
    report: NnsDataCenterRefreshReport,
) -> NnsTopologyRefreshRow {
    NnsTopologyRefreshRow {
        source: "data_centers".to_string(),
        cache_path: report.cache_path,
        refresh_lock_path: report.refresh_lock_path,
        registry_version: report.registry_version,
        fetched_at: report.fetched_at,
        source_endpoint: report.source_endpoint,
        fetched_by: report.fetched_by,
        dry_run: report.dry_run,
        wrote_cache: report.wrote_cache,
        replaced_existing_cache: report.replaced_existing_cache,
        item_count: report.data_center_count,
    }
}
