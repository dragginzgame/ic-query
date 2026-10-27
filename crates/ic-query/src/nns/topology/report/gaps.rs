//! Module: nns::topology::report::gaps
//!
//! Responsibility: build NNS topology join-gap reports.
//! Does not own: source reads, cache refresh, or text rendering.
//! Boundary: projects cached topology component reports into missing-relation rows.

use super::{
    NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION, NnsTopologyAssessmentStatus, NnsTopologyGapsReport,
};
use super::{TopologyInventoryReports, relations::TopologyRelationIndex};
use crate::nns::{
    node::NnsNodeRow,
    node_operator::NnsNodeOperatorRow,
    topology::report::{NnsTopologyGapRelationKind, NnsTopologyGapRow, NnsTopologyGapSubjectKind},
};

pub(super) fn topology_gaps_report_from_reports(
    network: String,
    source_endpoint: String,
    reports: &TopologyInventoryReports,
) -> NnsTopologyGapsReport {
    let index = TopologyRelationIndex::from_reports(
        &reports.node_provider,
        &reports.node_operator,
        &reports.data_center,
    );
    let mut gaps = collect_node_gaps(&reports.node.nodes, &index);
    gaps.extend(collect_node_operator_gaps(
        &reports.node_operator.node_operators,
        &index,
    ));
    sort_gap_rows(&mut gaps);

    let gap_count = gaps.len();
    let status = NnsTopologyAssessmentStatus::from_ok(gap_count == 0);

    NnsTopologyGapsReport {
        schema_version: NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status,
        gap_count,
        gaps,
    }
}

fn collect_node_gaps(
    nodes: &[NnsNodeRow],
    index: &TopologyRelationIndex<'_>,
) -> Vec<NnsTopologyGapRow> {
    let mut gaps = Vec::new();
    for node in nodes {
        if !index.has_node_provider(&node.node_provider_principal) {
            gaps.push(topology_gap_row(
                NnsTopologyGapSubjectKind::Node,
                &node.node_principal,
                NnsTopologyGapRelationKind::NodeProvider,
                &node.node_provider_principal,
            ));
        }
        if !index.has_node_operator(&node.node_operator_principal) {
            gaps.push(topology_gap_row(
                NnsTopologyGapSubjectKind::Node,
                &node.node_principal,
                NnsTopologyGapRelationKind::NodeOperator,
                &node.node_operator_principal,
            ));
        }
        if !index.has_data_center(&node.data_center_id) {
            gaps.push(topology_gap_row(
                NnsTopologyGapSubjectKind::Node,
                &node.node_principal,
                NnsTopologyGapRelationKind::DataCenter,
                &node.data_center_id,
            ));
        }
    }
    gaps
}

fn collect_node_operator_gaps(
    operators: &[NnsNodeOperatorRow],
    index: &TopologyRelationIndex<'_>,
) -> Vec<NnsTopologyGapRow> {
    let mut gaps = Vec::new();
    for operator in operators {
        if !index.has_node_provider(&operator.node_provider_principal) {
            gaps.push(topology_gap_row(
                NnsTopologyGapSubjectKind::NodeOperator,
                &operator.node_operator_principal,
                NnsTopologyGapRelationKind::NodeProvider,
                &operator.node_provider_principal,
            ));
        }
        if !index.has_data_center(&operator.data_center_id) {
            gaps.push(topology_gap_row(
                NnsTopologyGapSubjectKind::NodeOperator,
                &operator.node_operator_principal,
                NnsTopologyGapRelationKind::DataCenter,
                &operator.data_center_id,
            ));
        }
    }
    gaps
}

fn sort_gap_rows(gaps: &mut [NnsTopologyGapRow]) {
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
    subject_kind: NnsTopologyGapSubjectKind,
    subject: &str,
    missing_relation: NnsTopologyGapRelationKind,
    referenced_id: &str,
) -> NnsTopologyGapRow {
    NnsTopologyGapRow {
        subject_kind,
        subject: subject.to_string(),
        missing_relation,
        referenced_id: referenced_id.to_string(),
    }
}
