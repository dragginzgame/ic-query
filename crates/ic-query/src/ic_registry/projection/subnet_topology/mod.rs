use crate::{
    ic_registry::{
        MainnetRegistryFetchRequest, RegistryFetchError,
        projection::subnet_kind_from_registry,
        relations::{RegistryRelationInventory, ResolvedNodeRelation, resolved_node_relations},
    },
    nns::topology::{
        NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION, NnsSubnetNodeProviderRow,
        NnsSubnetTopologyReport, NnsSubnetTopologyRow,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};
use ic_host_tools::registry::proto::SubnetRecord;
use ic_platform::candid::contract::Principal;
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn subnet_topology_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: &RegistryRelationInventory,
    registry_version: u64,
) -> Result<NnsSubnetTopologyReport, RegistryFetchError> {
    let mut node_assignments = BTreeMap::<String, String>::new();
    let node_relations = resolved_node_relations(
        &inventory.node_principals,
        &inventory.node_operator_principals,
        &inventory.node_operator_records,
    )?;
    let mut subnets = Vec::with_capacity(inventory.subnet_records.len());

    for (subnet_principal, subnet_record) in &inventory.subnet_records {
        u32::try_from(subnet_record.membership.len()).map_err(|_| {
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
        subnets.push(NnsSubnetTopologyRow {
            subnet_principal: subnet_principal.clone(),
            subnet_kind: subnet_kind_from_registry(subnet_record.subnet_type),
            node_providers,
        });
    }

    Ok(NnsSubnetTopologyReport {
        schema_version: NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
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
) -> Result<Vec<NnsSubnetNodeProviderRow>, RegistryFetchError> {
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
            |(node_provider_principal, node_count)| NnsSubnetNodeProviderRow {
                node_provider_principal,
                node_count,
            },
        )
        .collect())
}

#[cfg(test)]
mod tests;
