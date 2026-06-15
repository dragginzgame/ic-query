use super::{fixtures::*, *};

#[test]
fn node_provider_counts_follow_subnet_nodes_to_providers() {
    let provider_a = Principal::self_authenticating(b"provider-a").to_text();
    let provider_b = Principal::self_authenticating(b"provider-b").to_text();
    let operator_a = Principal::self_authenticating(b"operator-a").to_text();
    let operator_b = Principal::self_authenticating(b"operator-b").to_text();
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_c = Principal::self_authenticating(b"node-c").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let subnet_records = BTreeMap::from([(
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
    )]);
    let node_principals =
        assigned_node_principals_from_subnets(&subnet_records).expect("node principals");
    let node_records = BTreeMap::from([
        (node_a, node_record(&operator_a)),
        (node_b, node_record(&operator_a)),
        (node_c, node_record(&operator_b)),
    ]);
    let node_operator_records = BTreeMap::from([
        (operator_a, node_operator_record(&provider_a)),
        (operator_b, node_operator_record(&provider_b)),
    ]);

    let counts =
        node_provider_counts_from_records(&node_principals, &node_records, &node_operator_records)
            .expect("provider counts");

    assert_eq!(counts.get(&provider_a), Some(&2));
    assert_eq!(counts.get(&provider_b), Some(&1));
}

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

#[test]
fn data_center_list_aggregates_registry_relations() {
    let request = registry_fetch_request();
    let provider_a = Principal::self_authenticating(b"provider-a").to_text();
    let provider_b = Principal::self_authenticating(b"provider-b").to_text();
    let operator_a = Principal::self_authenticating(b"operator-a").to_text();
    let operator_b = Principal::self_authenticating(b"operator-b").to_text();
    let node_a = Principal::self_authenticating(b"node-a").to_text();
    let node_b = Principal::self_authenticating(b"node-b").to_text();
    let node_c = Principal::self_authenticating(b"node-c").to_text();
    let subnet = Principal::self_authenticating(b"subnet").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node_a.clone(), node_b.clone(), node_c.clone()]),
        node_records: BTreeMap::from([
            (node_a.clone(), node_record(&operator_a)),
            (node_b.clone(), node_record(&operator_a)),
            (node_c.clone(), node_record(&operator_b)),
        ]),
        node_operator_records: BTreeMap::from([
            (
                operator_a.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&operator_a),
                    node_allowance: 4,
                    node_provider_principal_id: principal_raw(&provider_a),
                    dc_id: "dc-a".to_string(),
                },
            ),
            (
                operator_b.clone(),
                NodeOperatorRecord {
                    node_operator_principal_id: principal_raw(&operator_b),
                    node_allowance: 7,
                    node_provider_principal_id: principal_raw(&provider_b),
                    dc_id: "DC-A".to_string(),
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
        data_center_records: BTreeMap::from([(
            "dc-a".to_string(),
            DataCenterRecord {
                id: "dc-a".to_string(),
                region: "eu-west".to_string(),
                owner: "example owner".to_string(),
                gps: Some(proto::Gps {
                    latitude: 48.8566,
                    longitude: 2.3522,
                }),
            },
        )]),
    };

    let list = data_center_list_from_inventory(&request, inventory, 42).expect("data centers");

    assert_eq!(list.registry_version, 42);
    assert_eq!(list.data_centers.len(), 1);
    assert_eq!(list.data_centers[0].id, "dc-a");
    assert_eq!(list.data_centers[0].region, "eu-west");
    assert_eq!(list.data_centers[0].owner, "example owner");
    assert_eq!(list.data_centers[0].latitude, Some(48.8566));
    assert_eq!(list.data_centers[0].longitude, Some(2.3522));
    assert_eq!(list.data_centers[0].node_operator_count, 2);
    assert_eq!(list.data_centers[0].node_provider_count, 2);
    assert_eq!(list.data_centers[0].node_count, 3);
}
