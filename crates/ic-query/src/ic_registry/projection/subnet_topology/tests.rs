use super::*;
use crate::subnet_catalog::SubnetKind;
use ic_host_tools::registry::proto::{NodeOperatorRecord, NodeRecord, SubnetType};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn exact_version_projection_groups_providers_by_subnet() {
    let (subnet_a, subnet_b, node_a, node_b, node_c, operator_a, operator_b, provider_a) =
        principals();
    let provider_b = Principal::self_authenticating(b"provider-b").to_text();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node_a.clone(), node_b.clone(), node_c.clone()]),
        node_operator_principals:
            crate::ic_registry::relations::node_operator_principals_from_records(BTreeMap::from([
                (node_a.clone(), node_record(&operator_a)),
                (node_b.clone(), node_record(&operator_a)),
                (node_c.clone(), node_record(&operator_b)),
            ]))
            .expect("node operator principals"),
        node_operator_records: BTreeMap::from([
            (operator_a, node_operator_record(&provider_a)),
            (operator_b, node_operator_record(&provider_b)),
        ]),
        subnet_records: BTreeMap::from([
            (
                subnet_a.clone(),
                subnet_record(SubnetType::Application, &[node_a, node_b]),
            ),
            (
                subnet_b.clone(),
                subnet_record(SubnetType::CloudEngine, &[node_c]),
            ),
        ]),
        #[cfg(feature = "nns-host")]
        data_center_records: BTreeMap::new(),
    };

    let topology = subnet_topology_from_inventory(&request(), &inventory, 42).expect("topology");

    assert_eq!(topology.registry_version, 42);
    assert_eq!(topology.subnets.len(), 2);
    let subnet_a = topology
        .subnets
        .iter()
        .find(|subnet| subnet.subnet_principal == subnet_a)
        .expect("Subnet A");
    assert_eq!(subnet_a.subnet_kind, SubnetKind::Application);
    assert_eq!(subnet_a.node_count(), Ok(2));
    assert_eq!(subnet_a.node_providers.len(), 1);
    assert_eq!(
        subnet_a.node_providers[0],
        NnsSubnetNodeProviderRow {
            node_provider_principal: provider_a,
            node_count: 2,
        }
    );
    let subnet_b = topology
        .subnets
        .iter()
        .find(|subnet| subnet.subnet_principal == subnet_b)
        .expect("Subnet B");
    assert_eq!(subnet_b.subnet_kind, SubnetKind::CloudEngine);
    assert_eq!(
        subnet_b.node_providers[0].node_provider_principal,
        provider_b
    );
}

#[test]
fn projection_rejects_duplicate_node_assignment() {
    let (subnet_a, subnet_b, node, _, _, operator, _, provider) = principals();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node.clone()]),
        node_operator_principals:
            crate::ic_registry::relations::node_operator_principals_from_records(BTreeMap::from([
                (node.clone(), node_record(&operator)),
            ]))
            .expect("node operator principals"),
        node_operator_records: BTreeMap::from([(operator, node_operator_record(&provider))]),
        subnet_records: BTreeMap::from([
            (
                subnet_a,
                subnet_record(SubnetType::Application, std::slice::from_ref(&node)),
            ),
            (
                subnet_b,
                subnet_record(SubnetType::System, std::slice::from_ref(&node)),
            ),
        ]),
        #[cfg(feature = "nns-host")]
        data_center_records: BTreeMap::new(),
    };

    let error = subnet_topology_from_inventory(&request(), &inventory, 42).expect_err("duplicate");

    assert!(matches!(
        error,
        RegistryFetchError::DuplicateNodeAssignment { node_principal, .. }
            if node_principal == node
    ));
}

#[test]
fn projection_reports_missing_operator_relation() {
    let (subnet, _, node_a, node_b, _, operator, _, _) = principals();
    let mut expected_referencing_nodes = vec![node_a.clone(), node_b.clone()];
    expected_referencing_nodes.sort();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node_a.clone(), node_b.clone()]),
        node_operator_principals:
            crate::ic_registry::relations::node_operator_principals_from_records(BTreeMap::from([
                (node_a.clone(), node_record(&operator)),
                (node_b.clone(), node_record(&operator)),
            ]))
            .expect("node operator principals"),
        node_operator_records: BTreeMap::new(),
        subnet_records: BTreeMap::from([(
            subnet,
            subnet_record(SubnetType::System, &[node_a, node_b]),
        )]),
        #[cfg(feature = "nns-host")]
        data_center_records: BTreeMap::new(),
    };

    let error = subnet_topology_from_inventory(&request(), &inventory, 42).expect_err("gap");

    assert!(matches!(
        error,
        RegistryFetchError::MissingNodeOperatorRecord {
            node_operator_principal,
            referencing_node_principals,
        } if node_operator_principal == operator
            && referencing_node_principals == expected_referencing_nodes
    ));
}

#[test]
fn projection_reports_missing_provider_principal() {
    let (subnet, _, node, _, _, operator, _, _) = principals();
    let inventory = RegistryRelationInventory {
        node_principals: BTreeSet::from([node.clone()]),
        node_operator_principals:
            crate::ic_registry::relations::node_operator_principals_from_records(BTreeMap::from([
                (node.clone(), node_record(&operator)),
            ]))
            .expect("node operator principals"),
        node_operator_records: BTreeMap::from([(
            operator.clone(),
            NodeOperatorRecord {
                node_operator_principal_id: Vec::new(),
                node_allowance: 0,
                node_provider_principal_id: Vec::new(),
                dc_id: String::new(),
            },
        )]),
        subnet_records: BTreeMap::from([(subnet, subnet_record(SubnetType::System, &[node]))]),
        #[cfg(feature = "nns-host")]
        data_center_records: BTreeMap::new(),
    };

    let error = subnet_topology_from_inventory(&request(), &inventory, 42).expect_err("gap");

    assert!(matches!(
        error,
        RegistryFetchError::MissingNodeProviderPrincipal {
            node_operator_principal,
        } if node_operator_principal == operator
    ));
}

fn request() -> MainnetRegistryFetchRequest {
    MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-07-29T00:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    }
}

fn principals() -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    (
        Principal::self_authenticating(b"subnet-a").to_text(),
        Principal::self_authenticating(b"subnet-b").to_text(),
        Principal::self_authenticating(b"node-a").to_text(),
        Principal::self_authenticating(b"node-b").to_text(),
        Principal::self_authenticating(b"node-c").to_text(),
        Principal::self_authenticating(b"operator-a").to_text(),
        Principal::self_authenticating(b"operator-b").to_text(),
        Principal::self_authenticating(b"provider-a").to_text(),
    )
}

fn node_record(operator: &str) -> NodeRecord {
    NodeRecord {
        node_operator_id: Principal::from_text(operator)
            .expect("operator")
            .as_slice()
            .to_vec(),
    }
}

fn node_operator_record(provider: &str) -> NodeOperatorRecord {
    NodeOperatorRecord {
        node_operator_principal_id: Vec::new(),
        node_allowance: 0,
        node_provider_principal_id: Principal::from_text(provider)
            .expect("provider")
            .as_slice()
            .to_vec(),
        dc_id: String::new(),
    }
}

fn subnet_record(kind: SubnetType, nodes: &[String]) -> SubnetRecord {
    SubnetRecord {
        membership: nodes
            .iter()
            .map(|node| {
                Principal::from_text(node)
                    .expect("node")
                    .as_slice()
                    .to_vec()
            })
            .collect(),
        subnet_type: kind as i32,
        canister_cycles_cost_schedule: 0,
    }
}
