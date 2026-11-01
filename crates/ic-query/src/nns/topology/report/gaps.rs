//! Module: nns::topology::report::gaps
//!
//! Responsibility: build NNS topology join-gap reports.
//! Does not own: source reads, cache refresh, or text rendering.
//! Boundary: projects cached topology component reports into missing-relation rows.

use super::{
    NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION, NnsTopologyAssessmentStatus, NnsTopologyGapsReport,
};
use super::{
    TopologyInventoryReports,
    relations::{TopologyRelation, TopologyRelationIndex},
};
use crate::nns::topology::report::{
    NnsTopologyGapRelationKind, NnsTopologyGapRow, NnsTopologyGapSubjectKind,
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
    let mut gaps = Vec::new();
    for node in &reports.node.nodes {
        append_missing_relations(
            &mut gaps,
            NnsTopologyGapSubjectKind::Node,
            &node.node_principal,
            index.node_relations(node),
        );
    }
    for operator in &reports.node_operator.node_operators {
        append_missing_relations(
            &mut gaps,
            NnsTopologyGapSubjectKind::NodeOperator,
            &operator.node_operator_principal,
            index.operator_relations(operator),
        );
    }
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

fn append_missing_relations<'a>(
    gaps: &mut Vec<NnsTopologyGapRow>,
    subject_kind: NnsTopologyGapSubjectKind,
    subject: &str,
    relations: impl IntoIterator<Item = TopologyRelation<'a>>,
) {
    gaps.extend(
        relations
            .into_iter()
            .filter(|relation| !relation.known)
            .map(|relation| {
                topology_gap_row(subject_kind, subject, relation.kind, relation.referenced_id)
            }),
    );
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
