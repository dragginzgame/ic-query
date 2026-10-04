//! Module: nns::topology::report::registry_versions
//!
//! Responsibility: project shared component Registry provenance into topology rows.
//! Does not own: component collection, relation joins, or text rendering.
//! Boundary: keeps inventory and Subnet-catalog provenance consistent across views.

use super::NnsTopologyRegistryVersionRow;
use crate::{
    nns::{
        data_center::NnsDataCenterListReport, node::NnsNodeListReport,
        node_operator::NnsNodeOperatorListReport, node_provider::NnsNodeProviderListReport,
    },
    subnet_catalog::SubnetCatalogListReport,
};

pub(super) fn topology_registry_versions(
    subnet_report: &SubnetCatalogListReport,
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> Vec<NnsTopologyRegistryVersionRow> {
    let mut rows = Vec::with_capacity(5);
    rows.push(registry_version_row(
        "subnet_catalog",
        subnet_report.registry_version,
        &subnet_report.fetched_at,
        "-",
        Some(subnet_report.catalog_stale),
    ));
    rows.extend(topology_component_registry_versions(
        node_report,
        node_provider_report,
        node_operator_report,
        data_center_report,
    ));
    rows
}

pub(super) fn topology_component_registry_versions(
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> Vec<NnsTopologyRegistryVersionRow> {
    vec![
        registry_version_row(
            "nodes",
            node_report.registry_version,
            &node_report.fetched_at,
            &node_report.source_endpoint,
            None,
        ),
        registry_version_row(
            "node_providers",
            node_provider_report.registry_version,
            &node_provider_report.fetched_at,
            &node_provider_report.source_endpoint,
            None,
        ),
        registry_version_row(
            "node_operators",
            node_operator_report.registry_version,
            &node_operator_report.fetched_at,
            &node_operator_report.source_endpoint,
            None,
        ),
        registry_version_row(
            "data_centers",
            data_center_report.registry_version,
            &data_center_report.fetched_at,
            &data_center_report.source_endpoint,
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
