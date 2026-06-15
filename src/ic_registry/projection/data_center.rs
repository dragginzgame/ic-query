use super::super::{
    MainnetDataCenter, MainnetDataCenterList, MainnetRegistryFetchRequest, RegistryFetchError,
    normalized_data_center_id,
    proto::DataCenterRecord,
    relations::{
        RegistryRelationInventory, data_center_node_counts_from_records,
        data_center_operator_counts_from_records, data_center_provider_counts_from_records,
    },
};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn data_center_list_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: RegistryRelationInventory,
    registry_version: u64,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    let node_counts = data_center_node_counts_from_records(
        &inventory.node_principals,
        &inventory.node_records,
        &inventory.node_operator_records,
    )?;
    let operator_counts =
        data_center_operator_counts_from_records(&inventory.node_operator_records);
    let provider_counts =
        data_center_provider_counts_from_records(&inventory.node_operator_records)?;
    let mut data_centers = inventory
        .data_center_records
        .into_iter()
        .map(|(id, record)| {
            data_center_from_record(id, record, &operator_counts, &provider_counts, &node_counts)
        })
        .collect::<Result<Vec<_>, _>>()?;
    data_centers.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(MainnetDataCenterList {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        data_centers,
    })
}

fn data_center_from_record(
    id: String,
    record: DataCenterRecord,
    operator_counts: &BTreeMap<String, u32>,
    provider_counts: &BTreeMap<String, u32>,
    node_counts: &BTreeMap<String, u32>,
) -> Result<MainnetDataCenter, RegistryFetchError> {
    if !record.id.is_empty() && normalized_data_center_id(&record.id).as_deref() != Some(&id) {
        return Err(RegistryFetchError::InvalidDataCenterRecordId {
            key_id: id,
            record_id: record.id,
        });
    }
    Ok(MainnetDataCenter {
        node_operator_count: operator_counts.get(&id).copied().unwrap_or(0),
        node_provider_count: provider_counts.get(&id).copied().unwrap_or(0),
        node_count: node_counts.get(&id).copied().unwrap_or(0),
        id,
        region: record.region,
        owner: record.owner,
        latitude: record.gps.as_ref().map(|gps| gps.latitude),
        longitude: record.gps.as_ref().map(|gps| gps.longitude),
    })
}
