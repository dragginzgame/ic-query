//! Live mainnet IC NNS registry adapter for IC query tools.

mod catalog;
mod client;
mod error;
#[cfg(feature = "nns-topology-host")]
mod inventory;
mod model;
mod projection;
pub mod proto;
#[cfg(feature = "nns-topology-host")]
mod relations;
mod source;
mod transport;
mod wire;

use candid::Principal;
#[cfg(feature = "nns-host")]
pub use client::fetch_mainnet_certified_registry_delta_batch_async;
pub use client::fetch_mainnet_subnet_catalog_async;
#[cfg(feature = "nns-topology-host")]
pub use client::fetch_mainnet_subnet_topology;
#[cfg(feature = "nns-host")]
pub use client::{
    fetch_mainnet_data_center_list, fetch_mainnet_node_list, fetch_mainnet_node_operator_list,
    fetch_mainnet_node_provider_list, fetch_mainnet_registry_version,
};
pub use error::RegistryFetchError;
pub use model::MainnetRegistryFetchRequest;
#[cfg(feature = "nns-host")]
pub use model::{
    CertifiedRegistryDeltaBatch, CertifiedRegistryDeltaVersion, CertifiedRegistryMutation,
    CertifiedRegistryPrecondition, CertifiedRegistryValueEncoding,
};
#[cfg(feature = "nns-host")]
pub use model::{
    MainnetDataCenter, MainnetDataCenterList, MainnetNode, MainnetNodeList, MainnetNodeOperator,
    MainnetNodeOperatorList, MainnetNodeProvider, MainnetNodeProviderList,
    MainnetRegistryCertification, MainnetRegistryVersion,
};
#[cfg(feature = "nns-topology-host")]
pub use model::{
    MainnetSubnetTopology, MainnetSubnetTopologyNodeProvider, MainnetSubnetTopologySubnet,
};
use proto::{CanisterId, SubnetId};
#[cfg(feature = "nns-host")]
pub use transport::{
    MAX_CERTIFIED_DELTA_INLINE_VALUE_BYTES, MAX_CERTIFIED_DELTA_KEY_BYTES,
    MAX_CERTIFIED_DELTA_MUTATIONS, MAX_CERTIFIED_DELTA_PRECONDITIONS,
    MAX_CERTIFIED_DELTA_VALUE_BYTES, MAX_CERTIFIED_DELTA_VERSIONS, MAX_REGISTRY_CHUNK_BYTES,
    MAX_REGISTRY_CHUNK_REFERENCES, MAX_REGISTRY_CHUNK_RESPONSE_BYTES,
    MAX_REGISTRY_RECONSTRUCTED_VALUE_BYTES,
};

#[cfg(all(test, feature = "nns-host"))]
use crate::subnet_catalog::{
    MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, RawSubnetCatalog, UncertifiedCatalogCollection,
};

#[cfg(all(test, feature = "nns-host"))]
use catalog::{routing_ranges_from_table, subnet_info_from_record};

#[cfg(all(test, feature = "nns-host"))]
use candid::{Decode, Encode};

#[cfg(all(test, feature = "nns-host"))]
use projection::{
    data_center_list_from_inventory, node_list_from_inventory, node_operator_list_from_inventory,
    node_provider_from_governance, node_provider_list_from_response,
};

#[cfg(all(test, feature = "nns-host"))]
use proto::{
    DataCenterRecord, NodeOperatorRecord, NodeRecord, RoutingTable, SubnetListRecord, SubnetRecord,
    SubnetType,
};

#[cfg(all(test, feature = "nns-host"))]
use relations::{
    RegistryRelationInventory, assigned_node_principals_from_subnets,
    node_provider_counts_from_records,
};

#[cfg(all(test, feature = "nns-host"))]
use std::collections::{BTreeMap, BTreeSet};

#[cfg(all(test, feature = "nns-host"))]
use transport::registry_value_content_from_response;

#[cfg(all(test, feature = "nns-host"))]
use wire::{
    GovernanceAccountIdentifier, GovernanceNodeProvider, ListNodeProvidersResponse,
    RegistryGetChunkRequest, RegistryValueContent,
};

pub const DEFAULT_MAINNET_ENDPOINT: &str = "https://icp-api.io";
#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-topology-host")]
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

#[cfg(feature = "nns-host")]
fn normalized_data_center_id(data_center_id: &str) -> Option<String> {
    let trimmed = data_center_id.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

#[cfg(all(test, feature = "nns-host"))]
mod tests;
