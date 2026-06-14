//! Live mainnet IC NNS registry adapter for IC query tools.

mod annotations;
mod catalog;
mod client;
mod error;
mod model;
mod projection;
pub mod proto;
mod relations;
mod transport;
mod wire;

use annotations::apply_mainnet_annotations;
use candid::Principal;
pub use client::{
    fetch_mainnet_data_center_list, fetch_mainnet_node_list, fetch_mainnet_node_operator_list,
    fetch_mainnet_node_provider_list, fetch_mainnet_registry_version, fetch_mainnet_subnet_catalog,
};
pub use error::RegistryFetchError;
use futures::{StreamExt, TryStreamExt, stream};
use ic_agent::Agent;
pub use model::{
    MainnetDataCenter, MainnetDataCenterList, MainnetNode, MainnetNodeList, MainnetNodeOperator,
    MainnetNodeOperatorList, MainnetNodeProvider, MainnetNodeProviderList,
    MainnetRegistryFetchRequest, MainnetRegistryVersion,
};
use proto::{
    CanisterId, DataCenterRecord, NodeOperatorRecord, NodeRecord, SubnetId, SubnetListRecord,
    SubnetRecord,
};
use relations::{
    RegistryRelationInventory, RegistryRelationInventoryScope,
    assigned_node_principals_from_subnets, node_provider_counts_from_records,
};
use std::collections::{BTreeMap, BTreeSet};
use transport::{decode_message, get_registry_value};

#[cfg(test)]
use crate::subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, SubnetCatalog};

#[cfg(test)]
use catalog::{routing_ranges_from_table, subnet_info_from_record};

#[cfg(test)]
use candid::{Decode, Encode};

#[cfg(test)]
use projection::{
    data_center_list_from_inventory, node_list_from_inventory, node_operator_list_from_inventory,
    node_provider_from_governance, node_provider_list_from_response,
};

#[cfg(test)]
use proto::{RoutingTable, SubnetType};

#[cfg(test)]
use transport::{
    append_validated_chunk, hex_bytes, registry_value_content_from_response, sha256_digest,
};

#[cfg(test)]
use wire::{
    GovernanceAccountIdentifier, GovernanceNodeProvider, ListNodeProvidersResponse,
    RegistryGetChunkRequest, RegistryValueContent,
};

pub const DEFAULT_MAINNET_ENDPOINT: &str = "https://icp-api.io";
pub const MAINNET_GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

const SUBNET_LIST_KEY: &str = "subnet_list";
const ROUTING_TABLE_KEY: &str = "routing_table";
const SUBNET_RECORD_KEY_PREFIX: &str = "subnet_record_";
const NODE_RECORD_KEY_PREFIX: &str = "node_record_";
const NODE_OPERATOR_RECORD_KEY_PREFIX: &str = "node_operator_record_";
const DATA_CENTER_RECORD_KEY_PREFIX: &str = "data_center_record_";
const NODE_PROVIDER_ENRICHMENT_CONCURRENCY: usize = 32;

async fn fetch_node_provider_node_counts(
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

async fn fetch_registry_relation_inventory(
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
        .buffer_unordered(NODE_PROVIDER_ENRICHMENT_CONCURRENCY)
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
        .buffer_unordered(NODE_PROVIDER_ENRICHMENT_CONCURRENCY)
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
        .buffer_unordered(NODE_PROVIDER_ENRICHMENT_CONCURRENCY)
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

async fn fetch_data_center_records_for_inventory(
    agent: &Agent,
    registry_canister: &Principal,
    registry_version: u64,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, DataCenterRecord>, RegistryFetchError> {
    let data_center_ids = node_operator_records
        .values()
        .filter_map(|record| normalized_data_center_id(&record.dc_id))
        .collect::<BTreeSet<_>>();
    stream::iter(data_center_ids)
        .map(|data_center_id| async move {
            let key = data_center_record_key(&data_center_id);
            let record_bytes =
                get_registry_value(agent, registry_canister, &key, registry_version).await?;
            let record = decode_message::<DataCenterRecord>("DataCenterRecord", &record_bytes)?;
            Ok::<_, RegistryFetchError>((data_center_id, record))
        })
        .buffer_unordered(NODE_PROVIDER_ENRICHMENT_CONCURRENCY)
        .try_collect::<BTreeMap<_, _>>()
        .await
}

fn canister_id_text(
    canister_id: Option<&CanisterId>,
    field: &'static str,
) -> Result<String, RegistryFetchError> {
    let principal = canister_id
        .and_then(|id| id.principal_id.as_ref())
        .ok_or(RegistryFetchError::MissingField { field })?;
    principal_text_from_raw(&principal.raw, field)
}

fn subnet_id_text(subnet_id: &SubnetId) -> Result<String, RegistryFetchError> {
    let principal = subnet_id
        .principal_id
        .as_ref()
        .ok_or(RegistryFetchError::MissingField {
            field: "routing_table.entries.subnet_id.principal_id",
        })?;
    principal_text_from_raw(&principal.raw, "routing_table.entries.subnet_id")
}

fn principal_text_from_raw(raw: &[u8], field: &'static str) -> Result<String, RegistryFetchError> {
    Principal::try_from_slice(raw)
        .map(|principal| principal.to_text())
        .map_err(|err| RegistryFetchError::InvalidPrincipal {
            field,
            reason: err.to_string(),
        })
}

fn principal_text_from_required_raw(
    raw: &[u8],
    field: &'static str,
) -> Result<String, RegistryFetchError> {
    if raw.is_empty() {
        return Err(RegistryFetchError::MissingField { field });
    }
    principal_text_from_raw(raw, field)
}

fn subnet_record_key(subnet_principal: &str) -> String {
    format!("{SUBNET_RECORD_KEY_PREFIX}{subnet_principal}")
}

fn node_record_key(node_principal: &str) -> String {
    format!("{NODE_RECORD_KEY_PREFIX}{node_principal}")
}

fn node_operator_record_key(node_operator_principal: &str) -> String {
    format!("{NODE_OPERATOR_RECORD_KEY_PREFIX}{node_operator_principal}")
}

fn data_center_record_key(data_center_id: &str) -> String {
    format!("{DATA_CENTER_RECORD_KEY_PREFIX}{data_center_id}")
}

fn normalized_data_center_id(data_center_id: &str) -> Option<String> {
    let trimmed = data_center_id.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests;
