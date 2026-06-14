//! Live mainnet IC NNS registry adapter for IC query tools.

mod annotations;
mod catalog;
mod client;
mod error;
mod inventory;
mod model;
mod projection;
pub mod proto;
mod relations;
mod source;
mod transport;
mod wire;

use annotations::apply_mainnet_annotations;
use candid::Principal;
pub use client::{
    fetch_mainnet_data_center_list, fetch_mainnet_node_list, fetch_mainnet_node_operator_list,
    fetch_mainnet_node_provider_list, fetch_mainnet_registry_version, fetch_mainnet_subnet_catalog,
};
pub use error::RegistryFetchError;
pub use model::{
    MainnetDataCenter, MainnetDataCenterList, MainnetNode, MainnetNodeList, MainnetNodeOperator,
    MainnetNodeOperatorList, MainnetNodeProvider, MainnetNodeProviderList,
    MainnetRegistryFetchRequest, MainnetRegistryVersion,
};
use proto::{CanisterId, SubnetId};

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
use proto::{
    DataCenterRecord, NodeOperatorRecord, NodeRecord, RoutingTable, SubnetListRecord, SubnetRecord,
    SubnetType,
};

#[cfg(test)]
use relations::{
    RegistryRelationInventory, assigned_node_principals_from_subnets,
    node_provider_counts_from_records,
};

#[cfg(test)]
use std::collections::{BTreeMap, BTreeSet};

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
