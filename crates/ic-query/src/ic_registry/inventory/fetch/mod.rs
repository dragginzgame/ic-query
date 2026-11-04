#[cfg(feature = "nns-host")]
use super::data_center::fetch_data_center_records_for_inventory;
use super::{
    INVENTORY_FETCH_CONCURRENCY,
    keys::{node_operator_record_key, node_record_key},
};
use crate::ic_registry::{
    RegistryFetchError, SUBNET_LIST_KEY, principal_text_from_raw,
    relations::{
        RegistryRelationInventory, RegistryRelationInventoryScope,
        assigned_node_principals_from_subnets, node_operator_principals_from_records,
        node_operator_references,
    },
    subnet_record_key,
    transport::get_registry_record,
};
use futures_util::{StreamExt, TryStreamExt, stream};
use ic_host_tools::registry::proto::{
    NodeOperatorRecord, NodeRecord, SubnetListRecord, SubnetRecord,
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "nns-host")]
use crate::ic_registry::relations::node_counts_from_relations;

#[cfg(feature = "nns-host")]
pub(in crate::ic_registry) async fn fetch_node_provider_node_counts(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let inventory = fetch_registry_relation_inventory(
        agent,
        registry_canister,
        registry_version,
        RegistryRelationInventoryScope::BaseRelations,
    )
    .await?;
    node_counts_from_relations(
        &inventory.node_principals,
        &inventory.node_operator_principals,
        &inventory.node_operator_records,
        |relation| Some(relation.node_provider_principal),
    )
}

pub(in crate::ic_registry) async fn fetch_registry_relation_inventory(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
    scope: RegistryRelationInventoryScope,
) -> Result<RegistryRelationInventory, RegistryFetchError> {
    let subnet_records = fetch_subnet_records(agent, registry_canister, registry_version).await?;

    let node_principals = assigned_node_principals_from_subnets(&subnet_records)?;
    let node_records = stream::iter(node_principals.iter().cloned())
        .map(|node_principal| async move {
            let key = node_record_key(&node_principal);
            let record = get_registry_record::<NodeRecord>(
                agent,
                registry_canister,
                &key,
                registry_version,
                "NodeRecord",
            )
            .await
            .map_err(|error| translate_missing_node_record(&node_principal, error))?;
            Ok::<_, RegistryFetchError>((node_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    let node_operator_principals = node_operator_principals_from_records(node_records)?;
    let node_operator_references = node_operator_references(&node_operator_principals);

    let node_operator_records = stream::iter(node_operator_references)
        .map(
            |(node_operator_principal, referencing_node_principals)| async move {
                let key = node_operator_record_key(&node_operator_principal);
                let record = get_registry_record::<NodeOperatorRecord>(
                    agent,
                    registry_canister,
                    &key,
                    registry_version,
                    "NodeOperatorRecord",
                )
                .await
                .map_err(|error| {
                    translate_missing_node_operator_record(
                        &node_operator_principal,
                        &referencing_node_principals,
                        error,
                    )
                })?;
                Ok::<_, RegistryFetchError>((node_operator_principal, record))
            },
        )
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    #[cfg(feature = "nns-host")]
    let data_center_records = match scope {
        RegistryRelationInventoryScope::BaseRelations => BTreeMap::new(),
        RegistryRelationInventoryScope::WithDataCenters => {
            fetch_data_center_records_for_inventory(
                agent,
                registry_canister,
                registry_version,
                &node_operator_records,
            )
            .await?
        }
    };
    #[cfg(not(feature = "nns-host"))]
    let _ = scope;

    Ok(RegistryRelationInventory {
        node_principals,
        node_operator_principals,
        node_operator_records,
        subnet_records,
        #[cfg(feature = "nns-host")]
        data_center_records,
    })
}

async fn fetch_subnet_records(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
) -> Result<BTreeMap<String, SubnetRecord>, RegistryFetchError> {
    let subnet_list = get_registry_record::<SubnetListRecord>(
        agent,
        registry_canister,
        SUBNET_LIST_KEY,
        registry_version,
        "SubnetListRecord",
    )
    .await?;
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }

    let subnet_principals = subnet_list
        .subnets
        .iter()
        .map(|subnet_raw| principal_text_from_raw(subnet_raw, "subnet_list.subnets"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut unique_subnet_principals = BTreeSet::new();
    for subnet_principal in &subnet_principals {
        if !unique_subnet_principals.insert(subnet_principal.clone()) {
            return Err(RegistryFetchError::DuplicateSubnetPrincipal {
                subnet_principal: subnet_principal.clone(),
            });
        }
    }
    stream::iter(subnet_principals)
        .map(|subnet_principal| async move {
            let key = subnet_record_key(&subnet_principal);
            let record = get_registry_record::<SubnetRecord>(
                agent,
                registry_canister,
                &key,
                registry_version,
                "SubnetRecord",
            )
            .await?;
            Ok::<_, RegistryFetchError>((subnet_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await
}

fn translate_missing_node_record(
    node_principal: &str,
    error: RegistryFetchError,
) -> RegistryFetchError {
    if registry_key_not_present_for(&error, &node_record_key(node_principal)) {
        return RegistryFetchError::MissingNodeRecord {
            node_principal: node_principal.to_string(),
        };
    }
    error
}

fn translate_missing_node_operator_record(
    node_operator_principal: &str,
    referencing_node_principals: &[String],
    error: RegistryFetchError,
) -> RegistryFetchError {
    if registry_key_not_present_for(&error, &node_operator_record_key(node_operator_principal)) {
        return RegistryFetchError::MissingNodeOperatorRecord {
            node_operator_principal: node_operator_principal.to_string(),
            referencing_node_principals: referencing_node_principals.to_vec(),
        };
    }
    error
}

fn registry_key_not_present_for(error: &RegistryFetchError, expected_key: &str) -> bool {
    matches!(
        error,
        RegistryFetchError::RegistryValue { key, code, .. }
            if key == expected_key && code == "key_not_present"
    )
}

#[cfg(test)]
mod tests;
