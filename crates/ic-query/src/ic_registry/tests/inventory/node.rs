use crate::ic_registry::tests::{fixtures::*, *};

#[test]
fn node_list_follows_nodes_to_subnets_operators_and_providers() {
    let request = registry_fetch_request();
    let provider = Principal::self_authenticating(b"provider").to_text();
    let operator = Principal::self_authenticating(b"operator").to_text();
    let node = Principal::self_authenticating(b"node").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node.clone()]),
        node_records: BTreeMap::from([(node.clone(), node_record(&operator))]),
        node_operator_records: BTreeMap::from([(
            operator.clone(),
            NodeOperatorRecord {
                node_operator_principal_id: principal_raw(&operator),
                node_allowance: 4,
                node_provider_principal_id: principal_raw(&provider),
                dc_id: "dc-a".to_string(),
            },
        )]),
        subnet_records: BTreeMap::from([(
            subnet.clone(),
            SubnetRecord {
                membership: vec![principal_raw(&node)],
                subnet_type: SubnetType::Application as i32,
                canister_cycles_cost_schedule: 0,
            },
        )]),
        data_center_records: BTreeMap::new(),
    };

    let list = node_list_from_inventory(&request, inventory, 42).expect("nodes");

    assert_eq!(list.registry_version, 42);
    assert_eq!(list.nodes.len(), 1);
    assert_eq!(list.nodes[0].principal, node);
    assert_eq!(list.nodes[0].node_operator_principal, operator);
    assert_eq!(list.nodes[0].node_provider_principal, provider);
    assert_eq!(list.nodes[0].subnet_principal, subnet);
    assert_eq!(list.nodes[0].subnet_kind, "application");
    assert_eq!(list.nodes[0].data_center_id, "dc-a");
}
