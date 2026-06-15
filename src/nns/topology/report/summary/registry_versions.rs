use crate::{
    nns::{
        data_center::report::NnsDataCenterListReport, node::report::NnsNodeListReport,
        node_operator::report::NnsNodeOperatorListReport,
        node_provider::report::NnsNodeProviderListReport,
        topology::report::NnsTopologyRegistryVersionRow,
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
    vec![
        registry_version_row(
            "subnet_catalog",
            subnet_report.registry_version,
            subnet_report.fetched_at.clone(),
            None,
            Some(subnet_report.catalog_stale),
        ),
        registry_version_row(
            "nodes",
            node_report.registry_version,
            node_report.fetched_at.clone(),
            Some(node_report.source_endpoint.clone()),
            None,
        ),
        registry_version_row(
            "node_providers",
            node_provider_report.registry_version,
            node_provider_report.fetched_at.clone(),
            Some(node_provider_report.source_endpoint.clone()),
            None,
        ),
        registry_version_row(
            "node_operators",
            node_operator_report.registry_version,
            node_operator_report.fetched_at.clone(),
            Some(node_operator_report.source_endpoint.clone()),
            None,
        ),
        registry_version_row(
            "data_centers",
            data_center_report.registry_version,
            data_center_report.fetched_at.clone(),
            Some(data_center_report.source_endpoint.clone()),
            None,
        ),
    ]
}

fn registry_version_row(
    source: &str,
    registry_version: u64,
    fetched_at: String,
    source_endpoint: Option<String>,
    stale: Option<bool>,
) -> NnsTopologyRegistryVersionRow {
    NnsTopologyRegistryVersionRow {
        source: source.to_string(),
        registry_version,
        fetched_at,
        source_endpoint: source_endpoint.unwrap_or_else(|| "-".to_string()),
        stale,
    }
}
