use super::{fixtures::*, *};

#[test]
fn topology_providers_report_summarizes_provider_distribution() {
    let report = topology_providers_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.registered_node_provider_count, 1);
    assert_eq!(report.referenced_node_provider_count, 2);
    assert_eq!(report.provider_with_nodes_count, 2);
    assert_eq!(report.provider_with_node_operators_count, 2);
    assert_eq!(report.total_node_count, 3);
    assert_eq!(report.total_node_operator_count, 2);
    assert_eq!(report.total_node_allowance, 2);
    assert_eq!(report.over_assigned_provider_count, 1);
    assert_eq!(report.unknown_provider_count, 1);
    assert!(report.providers.iter().any(|provider| {
        provider.node_provider_principal == "provider-a"
            && provider.registered
            && provider.topology_node_count == 2
            && provider.node_operator_count == 1
            && provider.over_assigned_node_count == 1
            && provider.status == "over"
    }));
    assert!(report.providers.iter().any(|provider| {
        provider.node_provider_principal == "provider-z"
            && !provider.registered
            && provider.topology_node_count == 1
            && provider.node_operator_count == 1
            && provider.status == "unknown_provider"
    }));
}

#[test]
fn topology_providers_text_renders_provider_table() {
    let report = topology_providers_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        node_report_fixture(),
        node_provider_report_fixture(),
        node_operator_report_fixture(),
        data_center_report_fixture(),
    );

    let text = nns_topology_providers_report_text(&report);

    assert!(text.contains("NODE_PROVIDER"));
    assert!(text.contains("GOV_NODES"));
    assert!(text.contains("OPERATORS"));
    assert!(text.contains("provider-a"));
    assert!(text.contains("unknown_provider"));
}
