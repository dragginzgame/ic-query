//! Module: nns::topology::report::registry_versions
//!
//! Responsibility: project shared component Registry provenance into topology rows.
//! Does not own: component collection, relation joins, or text rendering.
//! Boundary: keeps inventory and Subnet-catalog provenance consistent across views.

use super::{NnsTopologyRegistryVersionRow, TopologyInventoryReports};
use crate::subnet_catalog::SubnetCatalogListReport;

pub(super) fn topology_registry_versions(
    subnet_report: &SubnetCatalogListReport,
    reports: &TopologyInventoryReports,
) -> Vec<NnsTopologyRegistryVersionRow> {
    let mut rows = Vec::with_capacity(5);
    rows.push(registry_version_row(
        "subnet_catalog",
        subnet_report.metadata.registry_version,
        &subnet_report.metadata.fetched_at,
        "-",
        Some(subnet_report.catalog_stale),
    ));
    rows.extend(topology_component_registry_versions(reports));
    rows
}

pub(super) fn topology_component_registry_versions(
    reports: &TopologyInventoryReports,
) -> Vec<NnsTopologyRegistryVersionRow> {
    vec![
        registry_version_row(
            "nodes",
            reports.node.provenance.registry_version,
            &reports.node.provenance.fetched_at,
            &reports.node.provenance.source_endpoint,
            None,
        ),
        registry_version_row(
            "node_providers",
            reports.node_provider.provenance.registry_version,
            &reports.node_provider.provenance.fetched_at,
            &reports.node_provider.provenance.source_endpoint,
            None,
        ),
        registry_version_row(
            "node_operators",
            reports.node_operator.provenance.registry_version,
            &reports.node_operator.provenance.fetched_at,
            &reports.node_operator.provenance.source_endpoint,
            None,
        ),
        registry_version_row(
            "data_centers",
            reports.data_center.provenance.registry_version,
            &reports.data_center.provenance.fetched_at,
            &reports.data_center.provenance.source_endpoint,
            None,
        ),
    ]
}

pub(super) fn registry_version_row(
    source: &str,
    registry_version: u64,
    fetched_at: &str,
    source_endpoint: &str,
    stale: Option<bool>,
) -> NnsTopologyRegistryVersionRow {
    NnsTopologyRegistryVersionRow {
        source: source.to_string(),
        registry_version,
        fetched_at: fetched_at.to_string(),
        source_endpoint: source_endpoint.to_string(),
        stale,
    }
}
