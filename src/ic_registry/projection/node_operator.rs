use crate::{
    ic_registry::{
        MainnetNodeOperator, MainnetNodeOperatorList, MainnetRegistryFetchRequest,
        RegistryFetchError, principal_text_from_required_raw,
        proto::NodeOperatorRecord,
        relations::{RegistryRelationInventory, node_operator_counts_from_records},
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID},
};
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn node_operator_list_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: RegistryRelationInventory,
    registry_version: u64,
) -> Result<MainnetNodeOperatorList, RegistryFetchError> {
    let node_counts = node_operator_counts_from_records(
        &inventory.node_principals,
        &inventory.node_records,
        &inventory.node_operator_records,
    )?;
    let mut node_operators = inventory
        .node_operator_records
        .into_iter()
        .map(|(principal, record)| node_operator_from_record(principal, record, &node_counts))
        .collect::<Result<Vec<_>, _>>()?;
    node_operators.sort_by(|left, right| left.principal.cmp(&right.principal));
    Ok(MainnetNodeOperatorList {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        node_operators,
    })
}

fn node_operator_from_record(
    principal: String,
    record: NodeOperatorRecord,
    node_counts: &BTreeMap<String, u32>,
) -> Result<MainnetNodeOperator, RegistryFetchError> {
    let node_provider_principal = principal_text_from_required_raw(
        &record.node_provider_principal_id,
        "node_operator_record.node_provider_principal_id",
    )?;
    Ok(MainnetNodeOperator {
        node_count: Some(node_counts.get(&principal).copied().unwrap_or(0)),
        principal,
        node_provider_principal,
        node_allowance: record.node_allowance,
        data_center_id: record.dc_id,
    })
}
