use super::{NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION, NnsTopologyGapRow, NnsTopologyGapsReport};
use crate::nns::data_center::report::NnsDataCenterListReport;
use crate::nns::node::report::NnsNodeListReport;
use crate::nns::node_operator::report::NnsNodeOperatorListReport;
use crate::nns::node_provider::report::NnsNodeProviderListReport;
use std::collections::BTreeSet;

pub(super) fn topology_gaps_report_from_reports(
    network: String,
    source_endpoint: String,
    node_report: NnsNodeListReport,
    node_provider_report: NnsNodeProviderListReport,
    node_operator_report: NnsNodeOperatorListReport,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologyGapsReport {
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
    let mut gaps = Vec::new();

    for node in &node_report.nodes {
        if !node_provider_principals.contains(node.node_provider_principal.as_str()) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "node_provider",
                &node.node_provider_principal,
            ));
        }
        if !node_operator_principals.contains(node.node_operator_principal.as_str()) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "node_operator",
                &node.node_operator_principal,
            ));
        }
        if !data_center_ids.contains(node.data_center_id.as_str()) {
            gaps.push(topology_gap_row(
                "node",
                &node.node_principal,
                "data_center",
                &node.data_center_id,
            ));
        }
    }

    for operator in &node_operator_report.node_operators {
        if !node_provider_principals.contains(operator.node_provider_principal.as_str()) {
            gaps.push(topology_gap_row(
                "node_operator",
                &operator.node_operator_principal,
                "node_provider",
                &operator.node_provider_principal,
            ));
        }
        if !data_center_ids.contains(operator.data_center_id.as_str()) {
            gaps.push(topology_gap_row(
                "node_operator",
                &operator.node_operator_principal,
                "data_center",
                &operator.data_center_id,
            ));
        }
    }

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
    let gap_count = gaps.len();
    let status = if gap_count == 0 { "ok" } else { "attention" }.to_string();

    NnsTopologyGapsReport {
        schema_version: NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION,
        network,
        source_endpoint,
        status,
        gap_count,
        gaps,
    }
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
