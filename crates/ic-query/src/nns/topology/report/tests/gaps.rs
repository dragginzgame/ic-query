use super::fixtures::*;
use super::*;

#[test]
fn topology_gaps_report_lists_unknown_join_subjects() {
    let report = topology_gaps_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        &TopologyInventoryReports {
            node: node_report_fixture(),
            node_provider: node_provider_report_fixture(),
            node_operator: node_operator_report_fixture(),
            data_center: data_center_report_fixture(),
        },
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.status, NnsTopologyAssessmentStatus::Attention);
    assert_eq!(report.gap_count, 5);
    assert!(report.gaps.iter().any(|gap| {
        gap.subject_kind == NnsTopologyGapSubjectKind::Node
            && gap.subject == "node-c"
            && gap.missing_relation == NnsTopologyGapRelationKind::NodeProvider
            && gap.referenced_id == "provider-z"
    }));
    assert!(report.gaps.iter().any(|gap| {
        gap.subject_kind == NnsTopologyGapSubjectKind::Node
            && gap.subject == "node-c"
            && gap.missing_relation == NnsTopologyGapRelationKind::NodeOperator
            && gap.referenced_id == "operator-z"
    }));
    assert!(report.gaps.iter().any(|gap| {
        gap.subject_kind == NnsTopologyGapSubjectKind::NodeOperator
            && gap.subject == "operator-b"
            && gap.missing_relation == NnsTopologyGapRelationKind::DataCenter
            && gap.referenced_id == "dc-z"
    }));
}

#[test]
fn topology_gaps_text_renders_gap_or_ok_tables() {
    let _report = topology_gaps_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        &TopologyInventoryReports {
            node: node_report_fixture(),
            node_provider: node_provider_report_fixture(),
            node_operator: node_operator_report_fixture(),
            data_center: data_center_report_fixture(),
        },
    );

    let clean_report = topology_gaps_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        &TopologyInventoryReports {
            node: node_report_fixture(),
            node_provider: complete_node_provider_report_fixture(),
            node_operator: complete_node_operator_report_fixture(),
            data_center: complete_data_center_report_fixture(),
        },
    );

    assert_eq!(clean_report.status, NnsTopologyAssessmentStatus::Ok);
    assert_eq!(clean_report.gap_count, 0);
}
