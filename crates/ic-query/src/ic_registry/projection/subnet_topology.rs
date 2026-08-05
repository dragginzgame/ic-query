use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, MainnetSubnetTopology, MainnetSubnetTopologyNodeProvider,
        MainnetSubnetTopologySubnet, RegistryFetchError,
        projection::subnet_kind_from_registry,
        proto::SubnetRecord,
        relations::{
            RegistryRelationInventory, ResolvedNodeRelation, resolved_node_relations_from_records,
        },
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};
use candid::Principal;
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn subnet_topology_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: &RegistryRelationInventory,
    registry_version: u64,
) -> Result<MainnetSubnetTopology, RegistryFetchError> {
    let mut node_assignments = BTreeMap::<String, String>::new();
    let node_relations = resolved_node_relations_from_records(
        &inventory.node_principals,
        &inventory.node_records,
        &inventory.node_operator_records,
    )?;
    let mut subnets = Vec::with_capacity(inventory.subnet_records.len());

    for (subnet_principal, subnet_record) in &inventory.subnet_records {
        let node_count = u32::try_from(subnet_record.membership.len()).map_err(|_| {
            RegistryFetchError::CountOverflow {
                field: "subnet_record.membership",
            }
        })?;
        let node_providers = subnet_node_providers(
            subnet_principal,
            subnet_record,
            &node_relations,
            &mut node_assignments,
        )?;
        subnets.push(MainnetSubnetTopologySubnet {
            subnet_principal: subnet_principal.clone(),
            subnet_kind: subnet_kind_from_registry(subnet_record.subnet_type),
            node_count,
            node_providers,
        });
    }

    Ok(MainnetSubnetTopology {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        subnets,
    })
}

fn subnet_node_providers(
    subnet_principal: &str,
    subnet_record: &SubnetRecord,
    node_relations: &BTreeMap<String, ResolvedNodeRelation>,
    node_assignments: &mut BTreeMap<String, String>,
) -> Result<Vec<MainnetSubnetTopologyNodeProvider>, RegistryFetchError> {
    let mut provider_counts = BTreeMap::<String, u32>::new();
    for node_raw in &subnet_record.membership {
        let node_principal = Principal::try_from_slice(node_raw)
            .map(|principal| principal.to_text())
            .map_err(|err| RegistryFetchError::InvalidPrincipal {
                field: "subnet_record.membership",
                reason: err.to_string(),
            })?;
        if let Some(first_subnet_principal) =
            node_assignments.insert(node_principal.clone(), subnet_principal.to_string())
        {
            return Err(RegistryFetchError::DuplicateNodeAssignment {
                node_principal,
                first_subnet_principal,
                second_subnet_principal: subnet_principal.to_string(),
            });
        }
        let relation = node_relations.get(&node_principal).ok_or_else(|| {
            RegistryFetchError::MissingNodeRecord {
                node_principal: node_principal.clone(),
            }
        })?;
        let count = provider_counts
            .entry(relation.node_provider_principal.clone())
            .or_default();
        *count = count
            .checked_add(1)
            .ok_or(RegistryFetchError::CountOverflow {
                field: "node_provider.node_count",
            })?;
    }

    Ok(provider_counts
        .into_iter()
        .map(
            |(node_provider_principal, node_count)| MainnetSubnetTopologyNodeProvider {
                node_provider_principal,
                node_count,
            },
        )
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic_registry::proto::{NodeOperatorRecord, NodeRecord, SubnetType};
    use crate::subnet_catalog::SubnetKind;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn exact_version_projection_groups_providers_by_subnet() {
        let (subnet_a, subnet_b, node_a, node_b, node_c, operator_a, operator_b, provider_a) =
            principals();
        let provider_b = Principal::self_authenticating(b"provider-b").to_text();
        let inventory = RegistryRelationInventory {
            node_principals: BTreeSet::from([node_a.clone(), node_b.clone(), node_c.clone()]),
            node_records: BTreeMap::from([
                (node_a.clone(), node_record(&operator_a)),
                (node_b.clone(), node_record(&operator_a)),
                (node_c.clone(), node_record(&operator_b)),
            ]),
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

        let topology =
            subnet_topology_from_inventory(&request(), &inventory, 42).expect("topology");

        assert_eq!(topology.registry_version, 42);
        assert_eq!(topology.subnets.len(), 2);
        let subnet_a = topology
            .subnets
            .iter()
            .find(|subnet| subnet.subnet_principal == subnet_a)
            .expect("Subnet A");
        assert_eq!(subnet_a.subnet_kind, SubnetKind::Application);
        assert_eq!(subnet_a.node_count, 2);
        assert_eq!(subnet_a.node_providers.len(), 1);
        assert_eq!(
            subnet_a.node_providers[0],
            MainnetSubnetTopologyNodeProvider {
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
            node_records: BTreeMap::from([(node.clone(), node_record(&operator))]),
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

        let error =
            subnet_topology_from_inventory(&request(), &inventory, 42).expect_err("duplicate");

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
            node_records: BTreeMap::from([
                (node_a.clone(), node_record(&operator)),
                (node_b.clone(), node_record(&operator)),
            ]),
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
            node_records: BTreeMap::from([(node.clone(), node_record(&operator))]),
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
}
