use super::super::{fixtures::*, *};

#[test]
fn node_operator_list_follows_assigned_nodes_to_operator_records() {
    let request = registry_fetch_request();
    let provider = Principal::self_authenticating(b"provider").to_text();
    let primary_operator = Principal::self_authenticating(b"operator-a").to_text();
    let secondary_operator = Principal::self_authenticating(b"operator-b").to_text();
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_c = Principal::self_authenticating(b"node-c").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node_a.clone(), node_b.clone(), node_c.clone()]),
        node_records: BTreeMap::from([
            (node_a.clone(), node_record(&primary_operator)),
            (node_b.clone(), node_record(&primary_operator)),
            (node_c.clone(), node_record(&secondary_operator)),
        ]),
        node_operator_records: BTreeMap::from([
            (
                primary_operator.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&primary_operator),
                    node_allowance: 4,
                    node_provider_principal_id: principal_raw(&provider),
                    dc_id: "dc-a".to_string(),
                },
            ),
            (
                secondary_operator.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&secondary_operator),
                    node_allowance: 7,
                    node_provider_principal_id: principal_raw(&provider),
                    dc_id: "dc-b".to_string(),
                },
            ),
        ]),
        subnet_records: BTreeMap::from([(
            subnet,
            SubnetRecord {
                membership: vec![
                    principal_raw(&node_a),
                    principal_raw(&node_b),
                    principal_raw(&node_c),
                ],
                subnet_type: SubnetType::Application as i32,
                canister_cycles_cost_schedule: 0,
            },
        )]),
        data_center_records: BTreeMap::new(),
    };

    let list = node_operator_list_from_inventory(&request, inventory, 42).expect("node operators");

    assert_eq!(list.network, MAINNET_NETWORK);
    assert_eq!(list.registry_canister_id, MAINNET_REGISTRY_CANISTER_ID);
    assert_eq!(list.registry_version, 42);
    assert_eq!(list.node_operators.len(), 2);
    let primary_result = list
        .node_operators
        .iter()
        .find(|operator| operator.principal == primary_operator)
        .expect("primary operator");
    assert_eq!(primary_result.node_provider_principal, provider);
    assert_eq!(primary_result.node_allowance, 4);
    assert_eq!(primary_result.data_center_id, "dc-a");
    assert_eq!(primary_result.node_count, Some(2));
    let secondary_result = list
        .node_operators
        .iter()
        .find(|operator| operator.principal == secondary_operator)
        .expect("secondary operator");
    assert_eq!(secondary_result.node_count, Some(1));
}
