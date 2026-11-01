use crate::ic_registry::{
    MainnetDataCenterList, MainnetRegistryFetchRequest, RegistryFetchError,
    normalized_data_center_id,
    relations::{
        DataCenterOperatorCounts, RegistryRelationInventory,
        data_center_operator_counts_from_records, node_counts_from_relations,
    },
};
use crate::nns::data_center::NnsDataCenterRow;
use ic_host_tools::registry::proto::DataCenterRecord;
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn data_center_list_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: RegistryRelationInventory,
    registry_version: u64,
) -> Result<MainnetDataCenterList, RegistryFetchError> {
    let node_counts = node_counts_from_relations(
        &inventory.node_principals,
        &inventory.node_operator_principals,
        &inventory.node_operator_records,
        |relation| normalized_data_center_id(&relation.data_center_id),
    )?;
    let operator_counts =
        data_center_operator_counts_from_records(&inventory.node_operator_records)?;
    let data_centers = inventory
        .data_center_records
        .into_iter()
        .map(|(id, record)| data_center_from_record(id, record, &operator_counts, &node_counts))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MainnetDataCenterList {
        provenance: request.inventory_provenance(registry_version),
        data_centers,
    })
}

fn data_center_from_record(
    id: String,
    record: DataCenterRecord,
    operator_counts: &BTreeMap<String, DataCenterOperatorCounts>,
    node_counts: &BTreeMap<String, u32>,
) -> Result<NnsDataCenterRow, RegistryFetchError> {
    if !record.id.is_empty() && normalized_data_center_id(&record.id).as_deref() != Some(&id) {
        return Err(RegistryFetchError::InvalidDataCenterRecordId {
            key_id: id,
            record_id: record.id,
        });
    }
    Ok(NnsDataCenterRow {
        node_operator_count: operator_counts
            .get(&id)
            .map_or(0, |counts| counts.operators),
        node_provider_count: operator_counts
            .get(&id)
            .map_or(0, |counts| counts.providers),
        node_count: node_counts.get(&id).copied().unwrap_or(0),
        data_center_id: id,
        region: record.region,
        owner: record.owner,
        latitude: record.gps.as_ref().map(|gps| gps.latitude),
        longitude: record.gps.as_ref().map(|gps| gps.longitude),
    })
}
