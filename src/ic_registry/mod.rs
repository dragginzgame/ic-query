//! Live mainnet IC NNS registry adapter for IC query tools.

mod client;
pub mod proto;

use crate::subnet_catalog::{
    CATALOG_SCHEMA_VERSION, CatalogError, ClassificationSource, GeographicScope, MAINNET_NETWORK,
    MAINNET_REGISTRY_CANISTER_ID, RoutingRange, SubnetCatalog, SubnetInfo, SubnetKind,
    SubnetSpecialization,
};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
pub use client::{
    fetch_mainnet_data_center_list, fetch_mainnet_node_list, fetch_mainnet_node_operator_list,
    fetch_mainnet_node_provider_list, fetch_mainnet_registry_version, fetch_mainnet_subnet_catalog,
};
use futures::{StreamExt, TryStreamExt, stream};
use ic_agent::Agent;
use prost::Message;
use proto::{
    CanisterId, DataCenterRecord, LargeValueChunkKeys, NodeOperatorRecord, NodeRecord,
    RegistryErrorCode, RegistryGetLatestVersionResponse, RegistryGetValueRequest,
    RegistryGetValueResponse, RoutingTable, SubnetId, SubnetListRecord, SubnetRecord, SubnetType,
    UInt64Value, registry_get_value_response,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error as ThisError;

pub const DEFAULT_MAINNET_ENDPOINT: &str = "https://icp-api.io";
pub const MAINNET_GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

const SUBNET_LIST_KEY: &str = "subnet_list";
const ROUTING_TABLE_KEY: &str = "routing_table";
const SUBNET_RECORD_KEY_PREFIX: &str = "subnet_record_";
const NODE_RECORD_KEY_PREFIX: &str = "node_record_";
const NODE_OPERATOR_RECORD_KEY_PREFIX: &str = "node_operator_record_";
const DATA_CENTER_RECORD_KEY_PREFIX: &str = "data_center_record_";
const NODE_PROVIDER_ENRICHMENT_CONCURRENCY: usize = 32;
const FIDUCIARY_SUBNET: &str = "pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae";
const EUROPEAN_SUBNET: &str = "bkfrj-6k62g-dycql-7h53p-atvkj-zg4to-gaogh-netha-ptybj-ntsgw-rqe";

///
/// MainnetRegistryFetchRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MainnetRegistryFetchRequest {
    pub endpoint: String,
    pub fetched_at: String,
    pub fetched_by: String,
}

impl MainnetRegistryFetchRequest {
    #[must_use]
    pub fn new(fetched_at: String) -> Self {
        Self {
            endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
            fetched_at,
            fetched_by: "ic-query".to_string(),
        }
    }
}

///
/// MainnetRegistryVersion
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetRegistryVersion {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
}

///
/// MainnetNodeProviderList
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeProviderList {
    pub network: String,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub node_providers: Vec<MainnetNodeProvider>,
}

///
/// MainnetNodeProvider
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeProvider {
    pub principal: String,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

///
/// MainnetNodeOperatorList
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeOperatorList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub node_operators: Vec<MainnetNodeOperator>,
}

///
/// MainnetNodeOperator
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeOperator {
    pub principal: String,
    pub node_provider_principal: String,
    pub node_allowance: u64,
    pub data_center_id: String,
    pub node_count: Option<u32>,
}

///
/// MainnetNodeList
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNodeList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub nodes: Vec<MainnetNode>,
}

///
/// MainnetNode
///
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetNode {
    pub principal: String,
    pub node_operator_principal: String,
    pub node_provider_principal: String,
    pub subnet_principal: String,
    pub subnet_kind: String,
    pub data_center_id: String,
}

///
/// MainnetDataCenterList
///
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MainnetDataCenterList {
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub data_centers: Vec<MainnetDataCenter>,
}

///
/// MainnetDataCenter
///
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MainnetDataCenter {
    pub id: String,
    pub region: String,
    pub owner: String,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub node_operator_count: u32,
    pub node_provider_count: u32,
    pub node_count: u32,
}

///
/// RegistryFetchError
///
#[derive(Debug, ThisError)]
pub enum RegistryFetchError {
    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("registry agent call {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("failed to encode protobuf {message}: {reason}")]
    ProtobufEncode {
        message: &'static str,
        reason: String,
    },

    #[error("failed to decode protobuf {message}: {reason}")]
    ProtobufDecode {
        message: &'static str,
        reason: String,
    },

    #[error("registry get_value for key {key} failed with code {code}: {reason}")]
    RegistryValue {
        key: String,
        code: String,
        reason: String,
    },

    #[error("registry get_value for key {key} returned no value content")]
    MissingValue { key: String },

    #[error("failed to encode candid {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("failed to decode candid {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error("registry get_chunk for sha256 {sha256} failed: {reason}")]
    RegistryChunkRejected { sha256: String, reason: String },

    #[error("registry get_chunk for sha256 {sha256} returned no chunk content")]
    MissingChunkContent { sha256: String },

    #[error("registry get_chunk for sha256 {sha256} returned content with sha256 {actual_sha256}")]
    ChunkHashMismatch {
        sha256: String,
        actual_sha256: String,
    },

    #[error("registry protobuf field {field} was missing")]
    MissingField { field: &'static str },

    #[error("registry principal field {field} is invalid: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    #[error("data center record id mismatch: key id {key_id}, record id {record_id}")]
    InvalidDataCenterRecordId { key_id: String, record_id: String },

    #[error("registry subnet list was empty")]
    EmptySubnetList,

    #[error("registry routing table was empty")]
    EmptyRoutingTable,

    #[error(transparent)]
    Catalog(#[from] CatalogError),

    #[error("failed to create Tokio runtime for registry refresh: {0}")]
    Runtime(String),
}

///
/// RegistryValueContent
///
#[derive(Debug)]
enum RegistryValueContent {
    Value(Vec<u8>),
    LargeValueChunkKeys(LargeValueChunkKeys),
}

///
/// RegistryGetChunkRequest
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct RegistryGetChunkRequest {
    content_sha256: Option<Vec<u8>>,
}

///
/// RegistryChunk
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct RegistryChunk {
    content: Option<Vec<u8>>,
}

///
/// ListNodeProvidersResponse
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListNodeProvidersResponse {
    node_providers: Vec<GovernanceNodeProvider>,
}

///
/// GovernanceNodeProvider
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct GovernanceNodeProvider {
    id: Option<Principal>,
    reward_account: Option<GovernanceAccountIdentifier>,
}

///
/// GovernanceAccountIdentifier
///
#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct GovernanceAccountIdentifier {
    hash: Vec<u8>,
}

async fn catalog_from_registry_records(
    request: &MainnetRegistryFetchRequest,
    registry_version: u64,
    agent: &Agent,
    registry_canister: &Principal,
    subnet_list: SubnetListRecord,
    routing_table: RoutingTable,
) -> Result<SubnetCatalog, RegistryFetchError> {
    if subnet_list.subnets.is_empty() {
        return Err(RegistryFetchError::EmptySubnetList);
    }
    if routing_table.entries.is_empty() {
        return Err(RegistryFetchError::EmptyRoutingTable);
    }

    let mut subnets = Vec::with_capacity(subnet_list.subnets.len());
    for subnet_raw in subnet_list.subnets {
        let subnet_principal = principal_text_from_raw(&subnet_raw, "subnet_list.subnets")?;
        let key = subnet_record_key(&subnet_principal);
        let record_bytes =
            get_registry_value(agent, registry_canister, &key, registry_version).await?;
        let record = decode_message::<SubnetRecord>("SubnetRecord", &record_bytes)?;
        subnets.push(subnet_info_from_record(&subnet_principal, &record));
    }

    subnets.sort_by(|left, right| left.subnet_principal.cmp(&right.subnet_principal));

    let mut routing_ranges = routing_ranges_from_table(&routing_table)?;
    routing_ranges.sort_by(|left, right| {
        left.start_canister_id
            .cmp(&right.start_canister_id)
            .then_with(|| left.end_canister_id.cmp(&right.end_canister_id))
            .then_with(|| left.subnet_principal.cmp(&right.subnet_principal))
    });

    let mut catalog = SubnetCatalog {
        catalog_schema_version: CATALOG_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        subnets,
        routing_ranges,
    };
    apply_mainnet_annotations(&mut catalog);
    catalog.validate()?;
    Ok(catalog)
}

fn node_provider_list_from_response(
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

fn node_provider_from_governance(
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

fn node_operator_list_from_inventory(
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

fn node_list_from_inventory(
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

fn data_center_list_from_inventory(
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

async fn get_latest_version(
    agent: &Agent,
    registry_canister: &Principal,
) -> Result<u64, RegistryFetchError> {
    let bytes = agent
        .query(registry_canister, "get_latest_version")
        .with_arg(Vec::<u8>::new())
        .call()
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "get_latest_version",
            reason: err.to_string(),
        })?;
    let response = decode_message::<RegistryGetLatestVersionResponse>(
        "RegistryGetLatestVersionResponse",
        &bytes,
    )?;
    Ok(response.version)
}

async fn get_registry_value(
    agent: &Agent,
    registry_canister: &Principal,
    key: &str,
    version: u64,
) -> Result<Vec<u8>, RegistryFetchError> {
    let request = RegistryGetValueRequest {
        version: Some(UInt64Value { value: version }),
        key: key.as_bytes().to_vec(),
    };
    let mut arg = Vec::new();
    request
        .encode(&mut arg)
        .map_err(|err| RegistryFetchError::ProtobufEncode {
            message: "RegistryGetValueRequest",
            reason: err.to_string(),
        })?;
    let bytes = agent
        .query(registry_canister, "get_value")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "get_value",
            reason: err.to_string(),
        })?;
    let response = decode_message::<RegistryGetValueResponse>("RegistryGetValueResponse", &bytes)?;
    match registry_value_content_from_response(key, response)? {
        RegistryValueContent::Value(value) => Ok(value),
        RegistryValueContent::LargeValueChunkKeys(keys) => {
            get_large_registry_value(agent, registry_canister, &keys).await
        }
    }
}

fn registry_value_content_from_response(
    key: &str,
    response: RegistryGetValueResponse,
) -> Result<RegistryValueContent, RegistryFetchError> {
    if let Some(error) = response.error {
        return Err(RegistryFetchError::RegistryValue {
            key: key.to_string(),
            code: registry_error_code(error.code).to_string(),
            reason: error.reason,
        });
    }
    match response.content {
        Some(registry_get_value_response::Content::Value(value)) => {
            Ok(RegistryValueContent::Value(value))
        }
        Some(registry_get_value_response::Content::LargeValueChunkKeys(keys)) => {
            Ok(RegistryValueContent::LargeValueChunkKeys(keys))
        }
        None => Err(RegistryFetchError::MissingValue {
            key: key.to_string(),
        }),
    }
}

async fn get_large_registry_value(
    agent: &Agent,
    registry_canister: &Principal,
    keys: &LargeValueChunkKeys,
) -> Result<Vec<u8>, RegistryFetchError> {
    let mut value = Vec::new();
    for chunk_sha256 in &keys.chunk_content_sha256s {
        let chunk_content = get_registry_chunk(agent, registry_canister, chunk_sha256).await?;
        append_validated_chunk(&mut value, chunk_sha256, chunk_content)?;
    }
    Ok(value)
}

async fn get_registry_chunk(
    agent: &Agent,
    registry_canister: &Principal,
    content_sha256: &[u8],
) -> Result<Vec<u8>, RegistryFetchError> {
    let request = RegistryGetChunkRequest {
        content_sha256: Some(content_sha256.to_vec()),
    };
    let arg = Encode!(&request).map_err(|err| RegistryFetchError::CandidEncode {
        message: "RegistryGetChunkRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(registry_canister, "get_chunk")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "get_chunk",
            reason: err.to_string(),
        })?;
    let result = Decode!(&bytes, Result<RegistryChunk, String>).map_err(|err| {
        RegistryFetchError::CandidDecode {
            message: "Result<RegistryChunk, String>",
            reason: err.to_string(),
        }
    })?;
    match result {
        Ok(chunk) => chunk
            .content
            .ok_or_else(|| RegistryFetchError::MissingChunkContent {
                sha256: hex_bytes(content_sha256),
            }),
        Err(reason) => Err(RegistryFetchError::RegistryChunkRejected {
            sha256: hex_bytes(content_sha256),
            reason,
        }),
    }
}

fn append_validated_chunk(
    value: &mut Vec<u8>,
    expected_sha256: &[u8],
    chunk_content: Vec<u8>,
) -> Result<(), RegistryFetchError> {
    let actual_sha256 = sha256_digest(&chunk_content);
    if actual_sha256.as_slice() != expected_sha256 {
        return Err(RegistryFetchError::ChunkHashMismatch {
            sha256: hex_bytes(expected_sha256),
            actual_sha256: hex_bytes(&actual_sha256),
        });
    }
    value.extend(chunk_content);
    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[usize::from(byte >> 4)] as char);
        out.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    out
}

fn decode_message<T>(message: &'static str, bytes: &[u8]) -> Result<T, RegistryFetchError>
where
    T: Message + Default,
{
    T::decode(bytes).map_err(|err| RegistryFetchError::ProtobufDecode {
        message,
        reason: err.to_string(),
    })
}

fn subnet_info_from_record(subnet_principal: &str, record: &SubnetRecord) -> SubnetInfo {
    let subnet_kind = match SubnetType::try_from(record.subnet_type).ok() {
        Some(SubnetType::Application | SubnetType::VerifiedApplication) => SubnetKind::Application,
        Some(SubnetType::CloudEngine) => SubnetKind::CloudEngine,
        Some(SubnetType::System) => SubnetKind::System,
        Some(SubnetType::Unspecified) | None => SubnetKind::Unknown,
    };
    let charges_apply_by_default = subnet_kind.charges_apply_by_default();
    SubnetInfo {
        subnet_principal: subnet_principal.to_string(),
        subnet_kind,
        subnet_kind_source: ClassificationSource::Registry,
        subnet_specialization: SubnetSpecialization::None,
        subnet_specialization_source: ClassificationSource::Computed,
        geographic_scope: GeographicScope::Global,
        geographic_scope_source: ClassificationSource::Computed,
        subnet_label: subnet_kind.as_str().to_string(),
        subnet_label_source: ClassificationSource::Computed,
        node_count: Some(u32::try_from(record.membership.len()).unwrap_or(u32::MAX)),
        charges_apply_by_default,
    }
}

fn routing_ranges_from_table(
    table: &RoutingTable,
) -> Result<Vec<RoutingRange>, RegistryFetchError> {
    table
        .entries
        .iter()
        .map(|entry| {
            let range = entry
                .range
                .as_ref()
                .ok_or(RegistryFetchError::MissingField {
                    field: "routing_table.entries.range",
                })?;
            let subnet_id = entry
                .subnet_id
                .as_ref()
                .ok_or(RegistryFetchError::MissingField {
                    field: "routing_table.entries.subnet_id",
                })?;
            Ok(RoutingRange {
                start_canister_id: canister_id_text(
                    range.start_canister_id.as_ref(),
                    "range.start",
                )?,
                end_canister_id: canister_id_text(range.end_canister_id.as_ref(), "range.end")?,
                subnet_principal: subnet_id_text(subnet_id)?,
            })
        })
        .collect()
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

fn assigned_node_principals_from_subnets(
    subnet_records: &BTreeMap<String, SubnetRecord>,
) -> Result<BTreeSet<String>, RegistryFetchError> {
    let mut node_principals = BTreeSet::new();
    for record in subnet_records.values() {
        for raw in &record.membership {
            node_principals.insert(principal_text_from_raw(raw, "subnet_record.membership")?);
        }
    }
    Ok(node_principals)
}

fn node_subnet_assignments_from_records(
    subnet_records: &BTreeMap<String, SubnetRecord>,
) -> Result<BTreeMap<String, String>, RegistryFetchError> {
    let mut assignments = BTreeMap::new();
    for (subnet_principal, record) in subnet_records {
        for raw in &record.membership {
            let node_principal = principal_text_from_raw(raw, "subnet_record.membership")?;
            assignments.insert(node_principal, subnet_principal.clone());
        }
    }
    Ok(assignments)
}

fn node_provider_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in assigned_node_relations(node_principals, node_records, node_operator_records)? {
        let count = counts.entry(relation.node_provider_principal).or_default();
        *count = count.saturating_add(1);
    }
    Ok(counts)
}

fn node_operator_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in assigned_node_relations(node_principals, node_records, node_operator_records)? {
        let count = counts.entry(relation.node_operator_principal).or_default();
        *count = count.saturating_add(1);
    }
    Ok(counts)
}

fn data_center_node_counts_from_records(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut counts = BTreeMap::<String, u32>::new();
    for relation in assigned_node_relations(node_principals, node_records, node_operator_records)? {
        if let Some(data_center_id) = relation.data_center_id {
            let count = counts.entry(data_center_id).or_default();
            *count = count.saturating_add(1);
        }
    }
    Ok(counts)
}

fn assigned_node_relations(
    node_principals: &BTreeSet<String>,
    node_records: &BTreeMap<String, NodeRecord>,
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<Vec<AssignedNodeRelation>, RegistryFetchError> {
    let mut relations = Vec::with_capacity(node_principals.len());
    for node_principal in node_principals {
        let node_record =
            node_records
                .get(node_principal)
                .ok_or(RegistryFetchError::MissingField {
                    field: "node_record",
                })?;
        let node_operator_principal = principal_text_from_required_raw(
            &node_record.node_operator_id,
            "node_record.node_operator_id",
        )?;
        let node_operator_record = node_operator_records.get(&node_operator_principal).ok_or(
            RegistryFetchError::MissingField {
                field: "node_operator_record",
            },
        )?;
        let node_provider_principal = principal_text_from_required_raw(
            &node_operator_record.node_provider_principal_id,
            "node_operator_record.node_provider_principal_id",
        )?;
        relations.push(AssignedNodeRelation {
            node_operator_principal,
            node_provider_principal,
            data_center_id: normalized_data_center_id(&node_operator_record.dc_id),
        });
    }
    Ok(relations)
}

fn data_center_operator_counts_from_records(
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::<String, u32>::new();
    for record in node_operator_records.values() {
        if let Some(data_center_id) = normalized_data_center_id(&record.dc_id) {
            let count = counts.entry(data_center_id).or_default();
            *count = count.saturating_add(1);
        }
    }
    counts
}

fn data_center_provider_counts_from_records(
    node_operator_records: &BTreeMap<String, NodeOperatorRecord>,
) -> Result<BTreeMap<String, u32>, RegistryFetchError> {
    let mut providers_by_data_center = BTreeMap::<String, BTreeSet<String>>::new();
    for record in node_operator_records.values() {
        let Some(data_center_id) = normalized_data_center_id(&record.dc_id) else {
            continue;
        };
        let node_provider_principal = principal_text_from_required_raw(
            &record.node_provider_principal_id,
            "node_operator_record.node_provider_principal_id",
        )?;
        providers_by_data_center
            .entry(data_center_id)
            .or_default()
            .insert(node_provider_principal);
    }
    Ok(providers_by_data_center
        .into_iter()
        .map(|(data_center_id, providers)| {
            (
                data_center_id,
                u32::try_from(providers.len()).unwrap_or(u32::MAX),
            )
        })
        .collect())
}

///
/// RegistryRelationInventory
///
struct RegistryRelationInventory {
    node_principals: BTreeSet<String>,
    node_records: BTreeMap<String, NodeRecord>,
    node_operator_records: BTreeMap<String, NodeOperatorRecord>,
    subnet_records: BTreeMap<String, SubnetRecord>,
    data_center_records: BTreeMap<String, DataCenterRecord>,
}

///
/// RegistryRelationInventoryScope
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RegistryRelationInventoryScope {
    BaseRelations,
    WithDataCenters,
}

///
/// AssignedNodeRelation
///
struct AssignedNodeRelation {
    node_operator_principal: String,
    node_provider_principal: String,
    data_center_id: Option<String>,
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

fn apply_mainnet_annotations(catalog: &mut SubnetCatalog) {
    let annotations = mainnet_annotations();
    for subnet in &mut catalog.subnets {
        let Some(annotation) = annotations.get(subnet.subnet_principal.as_str()) else {
            continue;
        };
        subnet.subnet_specialization = annotation.specialization;
        subnet.subnet_specialization_source = ClassificationSource::Curated;
        subnet.geographic_scope = annotation.geographic_scope;
        subnet.geographic_scope_source = ClassificationSource::Curated;
        subnet.subnet_label.clone_from(&annotation.label);
        subnet.subnet_label_source = ClassificationSource::Curated;
    }
}

fn mainnet_annotations() -> BTreeMap<&'static str, MainnetAnnotation> {
    BTreeMap::from([
        (
            FIDUCIARY_SUBNET,
            MainnetAnnotation {
                specialization: SubnetSpecialization::Fiduciary,
                geographic_scope: GeographicScope::Global,
                label: "fiduciary".to_string(),
            },
        ),
        (
            EUROPEAN_SUBNET,
            MainnetAnnotation {
                specialization: SubnetSpecialization::European,
                geographic_scope: GeographicScope::Europe,
                label: "european".to_string(),
            },
        ),
    ])
}

///
/// MainnetAnnotation
///
#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetAnnotation {
    specialization: SubnetSpecialization,
    geographic_scope: GeographicScope,
    label: String,
}

fn registry_error_code(code: i32) -> &'static str {
    match RegistryErrorCode::try_from(code).ok() {
        Some(RegistryErrorCode::MalformedMessage) => "malformed_message",
        Some(RegistryErrorCode::KeyNotPresent) => "key_not_present",
        Some(RegistryErrorCode::KeyAlreadyPresent) => "key_already_present",
        Some(RegistryErrorCode::VersionNotLatest) => "version_not_latest",
        Some(RegistryErrorCode::VersionBeyondLatest) => "version_beyond_latest",
        Some(RegistryErrorCode::Authorization) => "authorization",
        Some(RegistryErrorCode::InternalError) => "internal_error",
        None => "unknown",
    }
}

#[cfg(test)]
mod tests;
