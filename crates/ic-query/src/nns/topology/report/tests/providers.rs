use super::fixtures::*;
use super::*;

#[test]
fn topology_providers_report_summarizes_provider_distribution() {
    let report = topology_providers_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        &topology_inventory_fixture(),
    );

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.registered_node_provider_count, 1);
    assert_eq!(report.providers.len(), 2);
    assert!(
        serde_json::to_value(&report)
            .unwrap()
            .get("referenced_node_provider_count")
            .is_none()
    );
    assert_eq!(report.provider_with_nodes_count, 2);
    assert_eq!(report.provider_with_node_operators_count, 2);
    assert_eq!(report.total_node_count, 3);
    assert_eq!(report.total_node_operator_count, 2);
    assert_eq!(report.total_node_allowance, 2);
    assert_eq!(report.over_assigned_provider_count, 1);
    assert_eq!(report.unknown_provider_count, 1);
    assert_eq!(report.registry_versions.len(), 4);
    assert_eq!(
        report
            .registry_versions
            .iter()
            .map(|row| (row.source.as_str(), row.registry_version))
            .collect::<Vec<_>>(),
        vec![
            ("nodes", 43),
            ("node_providers", 44),
            ("node_operators", 43),
            ("data_centers", 43),
        ]
    );
    assert!(report.providers.iter().any(|provider| {
        provider.node_provider_principal == "provider-a"
            && provider.registered
            && provider.topology_node_count == 2
            && provider.node_operator_count == 1
            && provider.over_assigned_node_count == 1
            && provider.status == NnsTopologyProviderStatus::Over
    }));
    assert!(report.providers.iter().any(|provider| {
        provider.node_provider_principal == "provider-z"
            && !provider.registered
            && provider.topology_node_count == 1
            && provider.node_operator_count == 1
            && provider.status == NnsTopologyProviderStatus::UnknownProvider
    }));
}

#[test]
fn topology_providers_keep_registration_and_operator_local_capacity_distinct() {
    let mut providers = node_provider_report_fixture();
    providers.node_providers[0].node_count = None;
    let mut unused = providers.node_providers[0].clone();
    unused.node_provider_principal = "provider-unused".to_string();
    providers.node_providers.push(unused);

    let mut operators = node_operator_report_fixture();
    let mut spare = operators.node_operators[0].clone();
    spare.node_operator_principal = "operator-spare".to_string();
    spare.node_allowance = 4;
    spare.node_count = Some(1);
    spare.data_center_id = "dc-z".to_string();
    operators.node_operators.push(spare);
    operators.node_operators[1].node_count = None;

    let report = topology_providers_report_from_reports(
        MAINNET_NETWORK.to_string(),
        "https://icp-api.io".to_string(),
        &TopologyInventoryReports {
            registry: crate::nns::NnsRegistryInventorySnapshot {
                node_operators: operators.node_operators,
                ..topology_inventory_fixture().registry
            },
            node_provider: providers,
        },
    );

    let registered = report
        .providers
        .iter()
        .find(|row| row.node_provider_principal == "provider-a")
        .expect("registered provider");
    assert!(registered.registered);
    assert_eq!(registered.name, None);
    assert_eq!(registered.governance_node_count, None);
    assert_eq!(registered.topology_node_count, 2);
    assert_eq!(registered.node_operator_count, 2);
    assert_eq!(registered.total_node_allowance, 5);
    assert_eq!(registered.assigned_node_count, 3);
    assert_eq!(registered.available_node_slots, 3);
    assert_eq!(registered.over_assigned_node_count, 1);
    assert_eq!(registered.status, NnsTopologyProviderStatus::Over);
    assert_eq!(registered.data_center_count, 2);
    assert_eq!(registered.region_count, 1);

    let unknown = report
        .providers
        .iter()
        .find(|row| row.node_provider_principal == "provider-z")
        .expect("unregistered provider");
    assert!(!unknown.registered);
    assert_eq!(unknown.governance_node_count, None);
    assert_eq!(unknown.assigned_node_count, 0);
    assert_eq!(unknown.available_node_slots, 1);
    assert_eq!(unknown.data_center_count, 1);
    assert_eq!(unknown.region_count, 0);
    assert_eq!(unknown.status, NnsTopologyProviderStatus::UnknownProvider);

    let unused = report
        .providers
        .iter()
        .find(|row| row.node_provider_principal == "provider-unused")
        .expect("unused registered provider");
    assert!(unused.registered);
    assert_eq!(unused.topology_node_count, 0);
    assert_eq!(unused.node_operator_count, 0);
    assert_eq!(unused.data_center_count, 0);
    assert_eq!(unused.status, NnsTopologyProviderStatus::Unused);
    assert_eq!(report.registered_node_provider_count, 2);
    assert_eq!(report.providers.len(), 3);
    assert_eq!(report.unknown_provider_count, 1);
    assert_eq!(report.over_assigned_provider_count, 1);
}
