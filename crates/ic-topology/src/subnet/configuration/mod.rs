//! Selected Registry subnet configuration, not observed running state or availability.

use ic_platform::nns_registry_types::protobuf;
use ic_platform_types::{Principal, PrincipalError};
use serde::Serialize;
use thiserror::Error;

use crate::mainnet::{REGISTRY_LIMITS as LIMITS, REGISTRY_METHOD as METHOD};
use crate::{ReadError, mainnet};

const MAX_VERSION_BYTES: usize = 256;

/// Selected fields of one returned Registry record; no effective version is resolved.
#[derive(Debug, Serialize)]
pub struct SubnetConfigurationReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Requested subnet, identified by the queried key rather than an embedded ID.
    pub subnet_id: Principal,
    /// Sorted unique membership from this record, possibly empty.
    pub registered_node_ids: Vec<Principal>,
    /// Raw configured version; empty can mean a separate cloud-engine version source.
    /// This is not the effective or observed running replica version.
    pub replica_version_id: String,
    /// Raw signed Registry enum, preserving unspecified and unknown values.
    pub registered_subnet_type: i32,
    /// Registry halt configuration; not proof of observed downtime.
    pub is_halted: bool,
    /// Configuration to halt at the next catch-up-package height; not observed state.
    pub halt_at_cup_height: bool,
    /// Evidence for this record and its selected membership.
    pub evidence: SubnetConfigurationEvidence,
}

/// Source evidence for a single record, not a global Registry or liveness claim.
#[derive(Debug, Serialize)]
pub struct SubnetConfigurationEvidence {
    /// Admitted mainnet endpoint.
    pub endpoint: &'static str,
    /// Queried Registry canister.
    pub source_canister: &'static str,
    /// Explicit Registry query method.
    pub source_method: &'static str,
    /// Exact key constructed from the selected canonical subnet ID.
    pub source_key: String,
    /// Local completion time, in raw Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Record mutation version, not the latest global Registry version.
    pub record_version: u64,
    /// Raw mutation timestamp supplied by the Registry.
    pub record_timestamp_nanoseconds: u64,
    /// Agent authenticated the query signature with verified node keys.
    pub query_signature_verified: bool,
    /// False: a query signature does not consensus-certify the returned value.
    pub result_certified: bool,
    /// Every membership entry in this returned record is retained.
    /// Does not prove active subnet-list membership, live nodes or global coverage.
    pub membership_complete: bool,
}

/// Refusals from configuration reads; no error produces partial success.
#[derive(Debug, Error)]
pub enum SubnetConfigurationError {
    /// Mainnet transport, verification, timeout or clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The fixed Registry identity could not cross the Agent boundary.
    #[error("invalid Registry identity: {0}")]
    SourceIdentity(String),
    /// Platform refused malformed, unsupported or excessive protocol content.
    #[error(transparent)]
    Codec(#[from] protobuf::Error),
    /// Application rejection with raw fields retained and source text not displayed.
    #[error("Registry rejected subnet configuration (code {code})")]
    RegistryRejected {
        /// Raw source error code.
        code: i32,
        /// Raw reason; escape before terminal presentation.
        reason: String,
        /// Raw source key bytes.
        key: Vec<u8>,
    },
    /// No inline record was supplied; present empty proto3 records remain distinct.
    #[error("Registry returned no subnet configuration content")]
    MissingContent,
    /// One membership entry failed canonical principal admission.
    #[error("invalid registered node at entry {index}: {source}")]
    InvalidIdentity {
        /// Zero-based entry position in source order.
        index: usize,
        /// Canonical principal admission failure.
        #[source]
        source: PrincipalError,
    },
    /// Duplicate members make the selected membership ambiguous.
    #[error("Registry subnet configuration repeats a node identity")]
    DuplicateIdentity,
    /// The retained version exceeds the product's UTF-8 byte budget.
    #[error("configured replica version exceeds the 256-byte limit")]
    VersionLimit,
}

/// Fetch the latest record for one selected subnet through the mainnet Registry.
///
/// Uses the shared 20-second asynchronous deadline and 1 MiB HTTP ceiling.
/// Protobuf admission permits 64 KiB, 4096 fields, 128 KiB of allocation and 1024
/// members. The version string is bounded to 256 UTF-8 bytes. Requires Tokio IO
/// and time. No version resolution, Registry joins, caches or retries are added.
///
/// # Errors
/// Preserves transport/codec/source errors; rejects missing content, invalid or
/// duplicate members and excessive version text. Missing fields within a present
/// proto3 record retain schema defaults. Failure does not establish downtime.
pub async fn fetch(
    subnet_id: Principal,
) -> Result<SubnetConfigurationReport, SubnetConfigurationError> {
    let key = source_key(subnet_id);
    let argument = protobuf::encode_get_value_request(key.as_bytes(), None, 256)?;
    let agent = mainnet::agent()?;
    let registry = mainnet::REGISTRY
        .parse()
        .map_err(|error| SubnetConfigurationError::SourceIdentity(format!("{error}")))?;
    let bytes = mainnet::read(agent.query(&registry, METHOD).with_arg(argument).call()).await?;
    project(subnet_id, &bytes, mainnet::observed_at()?)
}

fn source_key(subnet_id: Principal) -> String {
    format!("subnet_record_{subnet_id}")
}

fn project(
    subnet_id: Principal,
    bytes: &[u8],
    observed_at: u64,
) -> Result<SubnetConfigurationReport, SubnetConfigurationError> {
    let response = protobuf::decode_get_value_response(bytes, LIMITS)?;
    if let Some(error) = response.error {
        return Err(SubnetConfigurationError::RegistryRejected {
            code: error.code,
            reason: error.reason.to_owned(),
            key: error.key.to_vec(),
        });
    }
    let record = protobuf::decode_subnet_record(
        response
            .value
            .ok_or(SubnetConfigurationError::MissingContent)?,
        LIMITS,
    )?;
    if record.replica_version_id.len() > MAX_VERSION_BYTES {
        return Err(SubnetConfigurationError::VersionLimit);
    }
    let mut registered_node_ids = Vec::with_capacity(record.membership.len());
    for (index, bytes) in record.membership.into_iter().enumerate() {
        registered_node_ids.push(
            Principal::try_from_slice(bytes)
                .map_err(|source| SubnetConfigurationError::InvalidIdentity { index, source })?,
        );
    }
    registered_node_ids.sort_unstable();
    if registered_node_ids
        .windows(2)
        .any(|pair| pair[0] == pair[1])
    {
        return Err(SubnetConfigurationError::DuplicateIdentity);
    }
    Ok(SubnetConfigurationReport {
        schema: 1,
        subnet_id,
        registered_node_ids,
        replica_version_id: record.replica_version_id.to_owned(),
        registered_subnet_type: record.subnet_type,
        is_halted: record.is_halted,
        halt_at_cup_height: record.halt_at_cup_height,
        evidence: SubnetConfigurationEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: mainnet::REGISTRY,
            source_method: METHOD,
            source_key: source_key(subnet_id),
            observed_at_unix_seconds: observed_at,
            record_version: response.version,
            record_timestamp_nanoseconds: response.timestamp_nanoseconds,
            query_signature_verified: true,
            result_certified: false,
            membership_complete: true,
        },
    })
}

#[cfg(test)]
mod tests;
