use super::{
    MAINNET_GOVERNANCE_CANISTER_ID, MainnetDataCenter, MainnetDataCenterList, MainnetNode,
    MainnetNodeList, MainnetNodeOperator, MainnetNodeOperatorList, MainnetNodeProvider,
    MainnetNodeProviderList, MainnetRegistryFetchRequest, RegistryFetchError,
    normalized_data_center_id, principal_text_from_required_raw,
};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};
use std::collections::BTreeMap;

use super::{
    proto::{DataCenterRecord, NodeOperatorRecord, NodeRecord, SubnetRecord, SubnetType},
    relations::{
        RegistryRelationInventory, data_center_node_counts_from_records,
        data_center_operator_counts_from_records, data_center_provider_counts_from_records,
        node_operator_counts_from_records, node_subnet_assignments_from_records,
    },
    transport::hex_bytes,
    wire::{GovernanceNodeProvider, ListNodeProvidersResponse},
};

pub(super) fn node_provider_list_from_response(
    request: &MainnetRegistryFetchRequest,
    response: ListNodeProvidersResponse,
    node_counts: BTreeMap<String, u32>,
    registry_version: u64,
) -> Result<MainnetNodeProviderList, RegistryFetchError> {
    let mut node_providers = response
        .node_providers
        .into_iter()
        .map(|node_provider| node_provider_from_governance(node_provider, &node_counts))
        .collect::<Result<Vec<_>, _>>()?;
    node_providers.sort_by(|left, right| left.principal.cmp(&right.principal));
    Ok(MainnetNodeProviderList {
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        node_providers,
    })
}

pub(super) fn node_provider_from_governance(
    node_provider: GovernanceNodeProvider,
    node_counts: &BTreeMap<String, u32>,
) -> Result<MainnetNodeProvider, RegistryFetchError> {
    let principal = node_provider
        .id
        .ok_or(RegistryFetchError::MissingField {
            field: "node_provider.id",
        })?
        .to_text();
    let reward_account_hex = node_provider
        .reward_account
        .map(|account| hex_bytes(&account.hash));
    let node_count = Some(node_counts.get(&principal).copied().unwrap_or(0));
    Ok(MainnetNodeProvider {
        principal,
        node_count,
        reward_account_hex,
    })
}

pub(super) fn node_operator_list_from_inventory(
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

pub(super) fn node_list_from_inventory(
    request: &MainnetRegistryFetchRequest,
    inventory: RegistryRelationInventory,
    registry_version: u64,
) -> Result<MainnetNodeList, RegistryFetchError> {
    let node_subnets = node_subnet_assignments_from_records(&inventory.subnet_records)?;
    let mut nodes = inventory
        .node_records
        .into_iter()
        .map(|(principal, record)| {
            node_from_record(
                principal,
                record,
                &inventory.node_operator_records,
                &inventory.subnet_records,
                &node_subnets,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    nodes.sort_by(|left, right| left.principal.cmp(&right.principal));
    Ok(MainnetNodeList {
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        nodes,
    })
}

fn node_from_record(
    principal: String,
    record: NodeRecord,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
    subnet_records: &BTreeMap<String, SubnetRecord>,
    node_subnets: &BTreeMap<String, String>,
) -> Result<MainnetNode, RegistryFetchError> {
    let node_operator_principal =
        principal_text_from_required_raw(&record.node_operator_id, "node_record.node_operator_id")?;
    let node_operator_record = node_operator_records.get(&node_operator_principal).ok_or(
        RegistryFetchError::MissingField {
            field: "node_operator_record",
        },
    )?;
    let node_provider_principal = principal_text_from_required_raw(
        &node_operator_record.node_provider_principal_id,
        "node_operator_record.node_provider_principal_id",
    )?;
    let subnet_principal =
        node_subnets
            .get(&principal)
            .ok_or(RegistryFetchError::MissingField {
                field: "node_subnet_assignment",
            })?;
    let subnet_record =
        subnet_records
            .get(subnet_principal)
            .ok_or(RegistryFetchError::MissingField {
                field: "subnet_record",
            })?;
    Ok(MainnetNode {
        principal,
        node_operator_principal,
        node_provider_principal,
        subnet_principal: subnet_principal.clone(),
        subnet_kind: subnet_kind_text(subnet_record),
        data_center_id: node_operator_record.dc_id.clone(),
    })
}

pub(super) fn data_center_list_from_inventory(
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

fn subnet_kind_text(record: &SubnetRecord) -> String {
    match SubnetType::try_from(record.subnet_type).ok() {
        Some(SubnetType::Application | SubnetType::VerifiedApplication) => "application",
        Some(SubnetType::CloudEngine) => "cloud_engine",
        Some(SubnetType::System) => "system",
        Some(SubnetType::Unspecified) | None => "unknown",
    }
    .to_string()
}
