//! Selected node Registry data, not verified hardware or observed running state.

use ic_platform::nns_registry_types::protobuf;
use ic_platform_types::{Principal, PrincipalError};
use serde::Serialize;
use thiserror::Error;

use crate::mainnet::{REGISTRY_LIMITS as LIMITS, REGISTRY_METHOD as METHOD};
use crate::{ReadError, mainnet};

const MAX_CHIP_ID_BYTES: usize = 256;
const MAX_HOSTOS_VERSION_BYTES: usize = 256;

/// Selected fields of one node's Registry record, retaining optional presence.
#[derive(Debug, Serialize)]
pub struct NodeReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Requested node, identified by the queried key rather than an embedded ID.
    pub node_id: Principal,
    /// Validated registering operator identity, without provider or location joins.
    pub node_operator_id: Principal,
    /// Raw registered SEV-SNP identifier; absence differs from present-empty bytes.
    /// Presence does not establish verified attestation or usable TEE protection.
    pub chip_id: Option<Vec<u8>>,
    /// Raw configured HostOS text, not measured running software.
    pub hostos_version_id: Option<String>,
    /// Source evidence and explicit assurance limits.
    pub evidence: NodeEvidence,
}

/// Evidence for the returned node record, not a liveness or hardware verdict.
#[derive(Debug, Serialize)]
pub struct NodeEvidence {
    /// Admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Queried Registry canister.
    pub source_canister: &'static str,
    /// Explicit Registry query method.
    pub source_method: &'static str,
    /// Exact key constructed from the canonical node ID.
    pub source_key: String,
    /// Local completion time, in raw Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Record mutation version, not the latest global Registry version.
    pub record_version: u64,
    /// Raw mutation timestamp supplied by Registry.
    pub record_timestamp_nanoseconds: u64,
    /// Agent authenticated the query signature with verified node keys.
    pub query_signature_verified: bool,
    /// False: query signatures do not consensus-certify the returned value.
    pub result_certified: bool,
    /// False: a registered identifier supplies no verified attestation.
    pub tee_attestation_verified: bool,
}

/// Refusals from node reads; failures do not establish node downtime.
#[derive(Debug, Error)]
pub enum NodeError {
    /// Mainnet transport, verification, timeout or clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The fixed Registry identity could not cross the Agent boundary.
    #[error("invalid Registry identity: {0}")]
    SourceIdentity(String),
    /// Platform refused malformed, unsupported or excessive protocol content.
    #[error(transparent)]
    Codec(#[from] protobuf::Error),
    /// Application rejection; raw source text is retained without terminal display.
    #[error("Registry rejected node inventory (code {code})")]
    RegistryRejected {
        /// Raw source error code.
        code: i32,
        /// Raw reason; escape before terminal presentation.
        reason: String,
        /// Raw source key bytes.
        key: Vec<u8>,
    },
    /// No inline record was supplied.
    #[error("Registry returned no node record content")]
    MissingContent,
    /// Empty/default operator bytes are missing data, not the management principal.
    #[error("Registry node record has no operator identity")]
    MissingOperator,
    /// Operator bytes failed canonical principal admission.
    #[error("invalid node operator identity: {0}")]
    InvalidOperator(#[source] PrincipalError),
    /// The retained chip identifier exceeds the byte budget.
    #[error("registered chip identifier exceeds the 256-byte limit")]
    ChipIdLimit,
    /// The retained HostOS text exceeds the UTF-8 byte budget.
    #[error("configured HostOS version exceeds the 256-byte limit")]
    HostosVersionLimit,
}

/// Read the latest Registry record for one explicitly selected node.
///
/// Uses the shared 20-second asynchronous deadline and 1 MiB HTTP ceiling.
/// Protobuf admission permits 64 KiB and 4096 fields per envelope/record.
/// Chip bytes and HostOS UTF-8 bytes are each limited to 256. Requires Tokio IO
/// and time; no network scan, relation join, attestation or retry is added.
///
/// # Errors
/// Preserves typed transport, codec and Registry causes. Refuses missing content,
/// missing/invalid operator identity and excessive fields without truncation.
pub async fn fetch(node_id: Principal) -> Result<NodeReport, NodeError> {
    let key = source_key(node_id);
    let argument = protobuf::encode_get_value_request(key.as_bytes(), None, 256)?;
    let agent = mainnet::agent()?;
    let registry = mainnet::REGISTRY
        .parse()
        .map_err(|error| NodeError::SourceIdentity(format!("{error}")))?;
    let bytes = mainnet::read(agent.query(&registry, METHOD).with_arg(argument).call()).await?;
    project(node_id, &bytes, mainnet::observed_at()?)
}

fn source_key(node_id: Principal) -> String {
    format!("node_record_{node_id}")
}

fn project(node_id: Principal, bytes: &[u8], observed_at: u64) -> Result<NodeReport, NodeError> {
    let response = protobuf::decode_get_value_response(bytes, LIMITS)?;
    if let Some(error) = response.error {
        return Err(NodeError::RegistryRejected {
            code: error.code,
            reason: error.reason.to_owned(),
            key: error.key.to_vec(),
        });
    }
    let record =
        protobuf::decode_node_record(response.value.ok_or(NodeError::MissingContent)?, LIMITS)?;
    if record.node_operator_id.is_empty() {
        return Err(NodeError::MissingOperator);
    }
    let node_operator_id =
        Principal::try_from_slice(record.node_operator_id).map_err(NodeError::InvalidOperator)?;
    if record
        .chip_id
        .is_some_and(|bytes| bytes.len() > MAX_CHIP_ID_BYTES)
    {
        return Err(NodeError::ChipIdLimit);
    }
    if record
        .hostos_version_id
        .is_some_and(|text| text.len() > MAX_HOSTOS_VERSION_BYTES)
    {
        return Err(NodeError::HostosVersionLimit);
    }
    Ok(NodeReport {
        schema: 1,
        node_id,
        node_operator_id,
        chip_id: record.chip_id.map(<[u8]>::to_vec),
        hostos_version_id: record.hostos_version_id.map(str::to_owned),
        evidence: NodeEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: mainnet::REGISTRY,
            source_method: METHOD,
            source_key: source_key(node_id),
            observed_at_unix_seconds: observed_at,
            record_version: response.version,
            record_timestamp_nanoseconds: response.timestamp_nanoseconds,
            query_signature_verified: true,
            result_certified: false,
            tee_attestation_verified: false,
        },
    })
}

#[cfg(test)]
mod tests;
