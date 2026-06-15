use super::super::{
    MainnetNode, MainnetNodeList, MainnetRegistryFetchRequest, RegistryFetchError,
    principal_text_from_required_raw,
    proto::{NodeOperatorRecord, NodeRecord, SubnetRecord, SubnetType},
    relations::{RegistryRelationInventory, node_subnet_assignments_from_records},
};
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID};
use std::collections::BTreeMap;

pub(in crate::ic_registry) fn node_list_from_inventory(
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

fn subnet_kind_text(record: &SubnetRecord) -> String {
    match SubnetType::try_from(record.subnet_type).ok() {
        Some(SubnetType::Application | SubnetType::VerifiedApplication) => "application",
        Some(SubnetType::CloudEngine) => "cloud_engine",
        Some(SubnetType::System) => "system",
        Some(SubnetType::Unspecified) | None => "unknown",
    }
    .to_string()
}
