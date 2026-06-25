use crate::ic_registry::tests::{fixtures::*, *};

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
