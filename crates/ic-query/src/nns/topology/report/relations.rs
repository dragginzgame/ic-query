use crate::nns::{
    data_center::report::NnsDataCenterListReport, node::report::NnsNodeListReport,
    node_operator::report::NnsNodeOperatorListReport,
    node_provider::report::NnsNodeProviderListReport,
};
use std::collections::BTreeSet;

pub(in crate::nns::topology::report) struct TopologyRelationIndex<'a> {
    node_provider_principals: BTreeSet<&'a str>,
    node_operator_principals: BTreeSet<&'a str>,
    data_center_ids: BTreeSet<&'a str>,
}

impl<'a> TopologyRelationIndex<'a> {
    pub(in crate::nns::topology::report) fn from_reports(
        node_provider_report: &'a NnsNodeProviderListReport,
        node_operator_report: &'a NnsNodeOperatorListReport,
        data_center_report: &'a NnsDataCenterListReport,
    ) -> Self {
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

        Self {
            node_provider_principals,
            node_operator_principals,
            data_center_ids,
        }
    }

    pub(in crate::nns::topology::report) fn has_node_provider(&self, principal: &str) -> bool {
        self.node_provider_principals.contains(principal)
    }

    pub(in crate::nns::topology::report) fn has_node_operator(&self, principal: &str) -> bool {
        self.node_operator_principals.contains(principal)
    }

    pub(in crate::nns::topology::report) fn has_data_center(&self, data_center_id: &str) -> bool {
        self.data_center_ids.contains(data_center_id)
    }

    pub(in crate::nns::topology::report) fn nodes_with_known_node_provider_count(
        &self,
        report: &NnsNodeListReport,
    ) -> usize {
        report
            .nodes
            .iter()
            .filter(|node| self.has_node_provider(&node.node_provider_principal))
            .count()
    }

    pub(in crate::nns::topology::report) fn nodes_with_known_node_operator_count(
        &self,
        report: &NnsNodeListReport,
    ) -> usize {
        report
            .nodes
            .iter()
            .filter(|node| self.has_node_operator(&node.node_operator_principal))
            .count()
    }

    pub(in crate::nns::topology::report) fn nodes_with_known_data_center_count(
        &self,
        report: &NnsNodeListReport,
    ) -> usize {
        report
            .nodes
            .iter()
            .filter(|node| self.has_data_center(&node.data_center_id))
            .count()
    }

    pub(in crate::nns::topology::report) fn node_operators_with_known_node_provider_count(
        &self,
        report: &NnsNodeOperatorListReport,
    ) -> usize {
        report
            .node_operators
            .iter()
            .filter(|operator| self.has_node_provider(&operator.node_provider_principal))
            .count()
    }

    pub(in crate::nns::topology::report) fn node_operators_with_known_data_center_count(
        &self,
        report: &NnsNodeOperatorListReport,
    ) -> usize {
        report
            .node_operators
            .iter()
            .filter(|operator| self.has_data_center(&operator.data_center_id))
            .count()
    }
}
