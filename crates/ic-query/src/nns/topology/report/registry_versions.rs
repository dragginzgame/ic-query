//! Module: nns::topology::report::registry_versions
//!
//! Responsibility: project shared component Registry provenance into topology rows.
//! Does not own: Subnet-catalog staleness or report-specific row selection.
//! Boundary: keeps common node/provider/operator/data-center provenance consistent.

use super::NnsTopologyRegistryVersionRow;
use crate::nns::{
    data_center::NnsDataCenterListReport, node::NnsNodeListReport,
    node_operator::NnsNodeOperatorListReport, node_provider::NnsNodeProviderListReport,
};

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
