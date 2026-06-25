use super::{
    INVENTORY_FETCH_CONCURRENCY,
    data_center::fetch_data_center_records_for_inventory,
    keys::{node_operator_record_key, node_record_key},
};
use crate::ic_registry::{
    RegistryFetchError, SUBNET_LIST_KEY, principal_text_from_raw, principal_text_from_required_raw,
    proto::{NodeOperatorRecord, NodeRecord, SubnetListRecord, SubnetRecord},
    relations::{
        RegistryRelationInventory, RegistryRelationInventoryScope,
        assigned_node_principals_from_subnets, node_provider_counts_from_records,
    },
    subnet_record_key,
    transport::{decode_message, get_registry_value},
};
use candid::Principal;
use futures::{StreamExt, TryStreamExt, stream};
use ic_agent::Agent;
use std::collections::{BTreeMap, BTreeSet};

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
    node_provider_counts_from_records(
        &inventory.node_principals,
        &inventory.node_records,
        &inventory.node_operator_records,
    )
}

pub(in crate::ic_registry) async fn fetch_registry_relation_inventory(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
    scope: RegistryRelationInventoryScope,
) -> Result<RegistryRelationInventory, RegistryFetchError> {
    let subnet_list_bytes =
        get_registry_value(agent, registry_canister, SUBNET_LIST_KEY, registry_version).await?;
    let subnet_list = decode_message::<SubnetListRecord>("SubnetListRecord", &subnet_list_bytes)?;
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }

    let subnet_principals = subnet_list
        .subnets
        .iter()
        .map(|subnet_raw| principal_text_from_raw(subnet_raw, "subnet_list.subnets"))
        .collect::<Result<Vec<_>, _>>()?;
    let subnet_records = stream::iter(subnet_principals)
        .map(|subnet_principal| async move {
            let key = subnet_record_key(&subnet_principal);
            let record_bytes =
                get_registry_value(agent, registry_canister, &key, registry_version).await?;
            let record = decode_message::<SubnetRecord>("SubnetRecord", &record_bytes)?;
            Ok::<_, RegistryFetchError>((subnet_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    let node_principals = assigned_node_principals_from_subnets(&subnet_records)?;
    let node_records = stream::iter(node_principals.iter().cloned())
        .map(|node_principal| async move {
            let key = node_record_key(&node_principal);
            let record_bytes =
                get_registry_value(agent, registry_canister, &key, registry_version).await?;
            let record = decode_message::<NodeRecord>("NodeRecord", &record_bytes)?;
            Ok::<_, RegistryFetchError>((node_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

    let mut node_operator_principals = BTreeSet::new();
    for record in node_records.values() {
        node_operator_principals.insert(principal_text_from_required_raw(
            &record.node_operator_id,
            "node_record.node_operator_id",
        )?);
    }

    let node_operator_records = stream::iter(node_operator_principals)
        .map(|node_operator_principal| async move {
            let key = node_operator_record_key(&node_operator_principal);
            let record_bytes =
                get_registry_value(agent, registry_canister, &key, registry_version).await?;
            let record = decode_message::<NodeOperatorRecord>("NodeOperatorRecord", &record_bytes)?;
            Ok::<_, RegistryFetchError>((node_operator_principal, record))
        })
        .buffer_unordered(INVENTORY_FETCH_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await?;

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

    Ok(RegistryRelationInventory {
        node_principals,
        node_records,
        node_operator_records,
        subnet_records,
        data_center_records,
    })
}
