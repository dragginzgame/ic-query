use crate::nns::{
    data_center::report::NnsDataCenterListReport, node::report::NnsNodeListReport,
    node_operator::report::NnsNodeOperatorListReport,
    node_provider::report::NnsNodeProviderListReport,
};
use std::collections::BTreeSet;

///
/// NnsTopologyJoinCoverageCounts
///
pub(super) struct NnsTopologyJoinCoverageCounts {
    pub(super) nodes_with_known_node_provider_count: usize,
    pub(super) nodes_with_known_node_operator_count: usize,
    pub(super) nodes_with_known_data_center_count: usize,
    pub(super) node_operators_with_known_node_provider_count: usize,
    pub(super) node_operators_with_known_data_center_count: usize,
}

pub(super) fn topology_summary_join_coverage_counts(
    node_report: &NnsNodeListReport,
    node_provider_report: &NnsNodeProviderListReport,
    node_operator_report: &NnsNodeOperatorListReport,
    data_center_report: &NnsDataCenterListReport,
) -> NnsTopologyJoinCoverageCounts {
    let node_provider_principals = node_provider_report
        .node_providers
        .iter()
        .map(|provider| provider.node_provider_principal.as_str())
        .collect::<BTreeSet<_>>();
    let node_operator_principals = node_operator_report
        .node_operators
        .iter()
        .map(|operator| operator.node_operator_principal.as_str())
        .collect::<BTreeSet<_>>();
    let data_center_ids = data_center_report
        .data_centers
        .iter()
        .map(|data_center| data_center.data_center_id.as_str())
        .collect::<BTreeSet<_>>();

    NnsTopologyJoinCoverageCounts {
        nodes_with_known_node_provider_count: node_count_with_known_node_provider(
            node_report,
            &node_provider_principals,
        ),
        nodes_with_known_node_operator_count: node_count_with_known_node_operator(
            node_report,
            &node_operator_principals,
        ),
        nodes_with_known_data_center_count: node_count_with_known_data_center(
            node_report,
            &data_center_ids,
        ),
        node_operators_with_known_node_provider_count: operator_count_with_known_node_provider(
            node_operator_report,
            &node_provider_principals,
        ),
        node_operators_with_known_data_center_count: operator_count_with_known_data_center(
            node_operator_report,
            &data_center_ids,
        ),
    }
}

fn node_count_with_known_node_provider(
    report: &NnsNodeListReport,
    providers: &BTreeSet<&str>,
) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| providers.contains(node.node_provider_principal.as_str()))
        .count()
}

fn node_count_with_known_node_operator(
    report: &NnsNodeListReport,
    operators: &BTreeSet<&str>,
) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| operators.contains(node.node_operator_principal.as_str()))
        .count()
}

fn node_count_with_known_data_center(
    report: &NnsNodeListReport,
    data_centers: &BTreeSet<&str>,
) -> usize {
    report
        .nodes
        .iter()
        .filter(|node| data_centers.contains(node.data_center_id.as_str()))
        .count()
}

fn operator_count_with_known_node_provider(
    report: &NnsNodeOperatorListReport,
    providers: &BTreeSet<&str>,
) -> usize {
    report
        .node_operators
        .iter()
        .filter(|operator| providers.contains(operator.node_provider_principal.as_str()))
        .count()
}

fn operator_count_with_known_data_center(
    report: &NnsNodeOperatorListReport,
    data_centers: &BTreeSet<&str>,
) -> usize {
    report
        .node_operators
        .iter()
        .filter(|operator| data_centers.contains(operator.data_center_id.as_str()))
        .count()
}
