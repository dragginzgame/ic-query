use crate::nns::{
    node::report::NnsNodeRow,
    node_operator::report::NnsNodeOperatorRow,
    topology::report::{NnsTopologyGapRow, relations::TopologyRelationIndex},
};

pub(super) fn collect_node_gaps(
    nodes: &[NnsNodeRow],
    index: &TopologyRelationIndex<'_>,
) -> Vec<NnsTopologyGapRow> {
    let mut gaps = Vec::new();
    for node in nodes {
        if !index.has_node_provider(&node.node_provider_principal) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "node_provider",
                &node.node_provider_principal,
            ));
        }
        if !index.has_node_operator(&node.node_operator_principal) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "node_operator",
                &node.node_operator_principal,
            ));
        }
        if !index.has_data_center(&node.data_center_id) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "data_center",
                &node.data_center_id,
            ));
        }
    }
    gaps
}

pub(super) fn collect_node_operator_gaps(
    operators: &[NnsNodeOperatorRow],
    index: &TopologyRelationIndex<'_>,
) -> Vec<NnsTopologyGapRow> {
    let mut gaps = Vec::new();
    for operator in operators {
        if !index.has_node_provider(&operator.node_provider_principal) {
            gaps.push(topology_gap_row(
                "node_operator",
                &operator.node_operator_principal,
                "node_provider",
                &operator.node_provider_principal,
            ));
        }
        if !index.has_data_center(&operator.data_center_id) {
            gaps.push(topology_gap_row(
                "node_operator",
                &operator.node_operator_principal,
                "data_center",
                &operator.data_center_id,
            ));
        }
    }
    gaps
}

pub(super) fn sort_gap_rows(gaps: &mut [NnsTopologyGapRow]) {
    gaps.sort_by(|left, right| {
        (
            left.subject_kind.as_str(),
            left.subject.as_str(),
            left.missing_relation.as_str(),
            left.referenced_id.as_str(),
        )
            .cmp(&(
                right.subject_kind.as_str(),
                right.subject.as_str(),
                right.missing_relation.as_str(),
                right.referenced_id.as_str(),
            ))
    });
}

fn topology_gap_row(
    subject_kind: &str,
    subject: &str,
    missing_relation: &str,
    referenced_id: &str,
) -> NnsTopologyGapRow {
    NnsTopologyGapRow {
        subject_kind: subject_kind.to_string(),
        subject: subject.to_string(),
        missing_relation: missing_relation.to_string(),
        referenced_id: referenced_id.to_string(),
    }
}
