use crate::ic_registry::tests::{fixtures::*, *};

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
