use crate::{
    runtime::RuntimeError,
    subnet_catalog::{
        CatalogAssurance, CatalogError, SubnetCatalogRegistryRecordEvidence, SubnetCatalogSubject,
    },
};
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

    /// The certificate signature, delegation, authority, or age check failed.
    #[error("Registry certificate authentication failed: {reason}")]
    CertificateAuthentication {
        /// Authentication failure returned by the IC agent.
        reason: String,
    },

    /// The certified response or committed Registry witness was malformed.
    #[error("certified Registry evidence is invalid: {reason}")]
    InvalidCertifiedRegistry {
        /// Deterministic certified-evidence validation failure.
        reason: String,
    },

    /// A large Registry value did not contain any chunk references.
    #[error("Registry large-value chunk list is empty")]
    EmptyRegistryChunkList,

    /// A Registry chunk reference was not one SHA-256 digest.
    #[error("Registry chunk digest is {actual_bytes} bytes; expected exactly 32")]
    InvalidRegistryChunkDigest {
        /// Actual digest length returned by the Registry.
        actual_bytes: usize,
    },

    /// A Registry chunk collection exceeded an explicit resource ceiling.
    #[error("Registry chunk {field} is {actual}; maximum is {maximum}")]
    RegistryChunkLimit {
        /// Bounded resource that exceeded its ceiling.
        field: &'static str,
        /// Enforced ceiling.
        maximum: usize,
        /// Observed or requested amount.
        actual: usize,
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

    #[error(
        "registry get_value for key {key} returned value version {returned_version} for pinned version {requested_version}"
    )]
    InvalidRegistryValueVersion {
        key: String,
        requested_version: u64,
        returned_version: u64,
    },

    #[error("registry get_changes_since failed with code {code}: {reason}")]
    RegistryChanges { code: String, reason: String },

    #[error(
        "registry key-family enumeration observed latest version {observed_version}, before pinned version {requested_version}"
    )]
    IncompleteRegistryChanges {
        requested_version: u64,
        observed_version: u64,
    },

    #[error("registry key-family evidence is invalid: {reason}")]
    InvalidRegistryKeyFamily { reason: String },

    #[error("registry key-family {field} count {actual} exceeds maximum {maximum}")]
    RegistryKeyFamilyLimit {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },

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

///
/// SubnetCatalogRegistryFailure
///
/// Internal pinned-version and subject provenance for one Registry catalog failure.
///

#[derive(Debug)]
pub struct SubnetCatalogRegistryFailure {
    pub registry_version: Option<u64>,
    pub returned_registry_value_version: Option<u64>,
    pub source_endpoint: Option<String>,
    pub assurance: Option<CatalogAssurance>,
    pub registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    pub subject: Option<SubnetCatalogSubject>,
    pub source: RegistryFetchError,
}

impl SubnetCatalogRegistryFailure {
    pub const fn new(
        registry_version: Option<u64>,
        subject: Option<SubnetCatalogSubject>,
        source: RegistryFetchError,
    ) -> Self {
        Self {
            registry_version,
            returned_registry_value_version: None,
            source_endpoint: None,
            assurance: None,
            registry_records: Vec::new(),
            subject,
            source,
        }
    }

    pub fn with_value_response(
        mut self,
        endpoint: &str,
        returned_registry_value_version: Option<u64>,
    ) -> Self {
        self.returned_registry_value_version = returned_registry_value_version;
        self.source_endpoint = Some(endpoint.to_string());
        self.assurance = Some(CatalogAssurance::UncertifiedQuery);
        self
    }

    pub fn with_registry_records(
        mut self,
        registry_records: Vec<SubnetCatalogRegistryRecordEvidence>,
    ) -> Self {
        self.registry_records = registry_records;
        self
    }
}
