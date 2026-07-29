use crate::{runtime::RuntimeError, subnet_catalog::CatalogError};
use thiserror::Error as ThisError;

///
/// RegistryFetchError
///
/// Errors returned while fetching and decoding mainnet registry records.
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

    #[error("registry count for {field} exceeded the supported u32 range")]
    CountOverflow { field: &'static str },

    #[error("registry principal field {field} is invalid: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    #[error(
        "node {node_principal} is assigned to both Subnet {first_subnet_principal} and Subnet {second_subnet_principal}"
    )]
    DuplicateNodeAssignment {
        node_principal: String,
        first_subnet_principal: String,
        second_subnet_principal: String,
    },

    #[error("Subnet membership references node {node_principal} without a node record")]
    MissingNodeRecord { node_principal: String },

    #[error("node {node_principal} has no node-operator principal")]
    MissingNodeOperatorPrincipal { node_principal: String },

    #[error(
        "nodes {referencing_node_principals:?} reference node operator {node_operator_principal} without a node-operator record"
    )]
    MissingNodeOperatorRecord {
        node_operator_principal: String,
        referencing_node_principals: Vec<String>,
    },

    #[error("node operator {node_operator_principal} has no node-provider principal")]
    MissingNodeProviderPrincipal { node_operator_principal: String },

    #[error("data center record id mismatch: key id {key_id}, record id {record_id}")]
    InvalidDataCenterRecordId { key_id: String, record_id: String },

    #[error("registry subnet list was empty")]
    EmptySubnetList,

    #[error("registry subnet list contains duplicate Subnet {subnet_principal}")]
    DuplicateSubnetPrincipal { subnet_principal: String },

    #[error("registry routing table was empty")]
    EmptyRoutingTable,

    #[error(transparent)]
    Catalog(#[from] CatalogError),

    #[error("failed to create Tokio runtime for registry refresh: {0}")]
    Runtime(#[from] RuntimeError),
}
