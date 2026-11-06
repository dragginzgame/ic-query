use super::{
    NnsTopologyGapRelationKind,
    NnsTopologyGapRelationKind::{DataCenter, NodeOperator, NodeProvider},
    TopologyInventoryReports,
};
use crate::nns::{node::NnsNodeRow, node_operator::NnsNodeOperatorRow};
use std::collections::BTreeSet;

///
/// TopologyRelation
///
/// Membership decision for one borrowed topology reference.
///

pub(super) struct TopologyRelation<'a> {
    pub(super) kind: NnsTopologyGapRelationKind,
    pub(super) referenced_id: &'a str,
    pub(super) known: bool,
}

///
/// TopologyRelationIndex
///
/// Borrowed registry identifiers used to validate topology relation joins.
///

pub(in crate::nns::topology::report) struct TopologyRelationIndex<'a> {
    node_provider_principals: BTreeSet<&'a str>,
    node_operator_principals: BTreeSet<&'a str>,
    data_center_ids: BTreeSet<&'a str>,
}

impl<'a> TopologyRelationIndex<'a> {
    pub(in crate::nns::topology::report) fn from_reports(
        reports: &'a TopologyInventoryReports,
    ) -> Self {
        let node_provider_principals = reports
            .node_provider
            .node_providers
            .iter()
            .map(|provider| provider.node_provider_principal.as_str())
            .collect::<BTreeSet<_>>();
        let node_operator_principals = reports
            .registry
            .node_operators
            .iter()
            .map(|operator| operator.node_operator_principal.as_str())
            .collect::<BTreeSet<_>>();
        let data_center_ids = reports
            .registry
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

    /// Return provider, operator and data-center relations in that order.
    pub(super) fn node_relations<'node>(
        &self,
        node: &'node NnsNodeRow,
    ) -> [TopologyRelation<'node>; 3] {
        [
            (NodeProvider, node.node_provider_principal.as_str()),
            (NodeOperator, node.node_operator_principal.as_str()),
            (DataCenter, node.data_center_id.as_str()),
        ]
        .map(|(kind, referenced_id)| self.relation(kind, referenced_id))
    }

    /// Return provider and data-center relations in that order.
    pub(super) fn operator_relations<'node>(
        &self,
        operator: &'node NnsNodeOperatorRow,
    ) -> [TopologyRelation<'node>; 2] {
        [
            (NodeProvider, operator.node_provider_principal.as_str()),
            (DataCenter, operator.data_center_id.as_str()),
        ]
        .map(|(kind, referenced_id)| self.relation(kind, referenced_id))
    }

    fn relation<'id>(
        &self,
        kind: NnsTopologyGapRelationKind,
        referenced_id: &'id str,
    ) -> TopologyRelation<'id> {
        let known = match kind {
            NodeProvider => self.node_provider_principals.contains(referenced_id),
            NodeOperator => self.node_operator_principals.contains(referenced_id),
            DataCenter => self.data_center_ids.contains(referenced_id),
        };
        TopologyRelation {
            kind,
            referenced_id,
            known,
        }
    }
}
