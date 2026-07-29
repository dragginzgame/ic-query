//! Module: nns::topology::report::summary::registry_versions
//!
//! Responsibility: collect registry-version rows for topology summaries.
//! Does not own: component report construction, relation joins, or rendering.
//! Boundary: projects component registry versions into summary report rows.

use crate::{
    nns::{
        data_center::NnsDataCenterListReport,
        node::NnsNodeListReport,
        node_operator::NnsNodeOperatorListReport,
        node_provider::NnsNodeProviderListReport,
        topology::report::{
            NnsTopologyRegistryVersionRow,
            registry_versions::{registry_version_row, topology_component_registry_versions},
        },
    },
    subnet_catalog::SubnetCatalogListReport,
};

pub(super) fn topology_summary_registry_versions(
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
