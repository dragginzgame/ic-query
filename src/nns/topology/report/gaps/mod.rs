mod rows;

use super::relations::TopologyRelationIndex;
use super::{NNS_TOPOLOGY_GAPS_REPORT_SCHEMA_VERSION, NnsTopologyGapsReport};
use crate::nns::{
    data_center::report::NnsDataCenterListReport, node::report::NnsNodeListReport,
    node_operator::report::NnsNodeOperatorListReport,
    node_provider::report::NnsNodeProviderListReport,
};
use rows::{collect_node_gaps, collect_node_operator_gaps, sort_gap_rows};

pub(super) fn topology_gaps_report_from_reports(
    network: String,
    source_endpoint: String,
    node_report: NnsNodeListReport,
    node_provider_report: NnsNodeProviderListReport,
    node_operator_report: NnsNodeOperatorListReport,
    data_center_report: NnsDataCenterListReport,
) -> NnsTopologyGapsReport {
    let index = TopologyRelationIndex::from_reports(
        &node_provider_report,
        &node_operator_report,
        &data_center_report,
    );
    let mut gaps = collect_node_gaps(&node_report.nodes, &index);
    gaps.extend(collect_node_operator_gaps(
        &node_operator_report.node_operators,
        &index,
    ));
    sort_gap_rows(&mut gaps);

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
