//! Registered subnet inventory from the mainnet Registry's `subnet_list` record.
//!
//! The complete selected record is returned or the operation fails. Registration
//! establishes neither liveness, available capacity nor permission to deploy.

use ic_platform::nns_registry_types::protobuf;
use ic_platform_types::{Principal, PrincipalError};
use serde::Serialize;
use thiserror::Error;

use crate::mainnet::{REGISTRY_LIMITS as LIMITS, REGISTRY_METHOD as METHOD};
use crate::{ReadError, mainnet};

const KEY: &str = "subnet_list";

/// Every registered subnet in one returned Registry record.
#[derive(Debug, Serialize)]
pub struct SubnetListReport {
    /// Query-owned schema; remains 1 before 1.0.
    pub schema: u8,
    /// Sorted, unique subnet identities from the selected record.
    pub subnet_ids: Vec<Principal>,
    /// Source, observation, mutation and verification evidence.
    pub evidence: SubnetListEvidence,
}

/// Evidence for the selected record, not a global Registry snapshot.
#[derive(Debug, Serialize)]
pub struct SubnetListEvidence {
    /// Fixed mainnet HTTPS endpoint.
    pub endpoint: &'static str,
    /// Queried Registry canister.
    pub source_canister: &'static str,
    /// Explicit query method.
    pub source_method: &'static str,
    /// Selected Registry key.
    pub source_key: &'static str,
    /// Local completion time, in raw Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Version at which this record last changed, not the latest global version.
    pub record_version: u64,
    /// Raw record mutation timestamp supplied by the Registry.
    pub record_timestamp_nanoseconds: u64,
    /// Agent authenticated the query signature using verified node keys.
    pub query_signature_verified: bool,
    /// False: query signatures do not certify the returned Registry value.
    pub result_certified: bool,
    /// Every entry of this returned record was decoded and retained.
    pub selected_record_complete: bool,
}

/// Refusals from subnet inventory; no error yields a partial success.
#[derive(Debug, Error)]
pub enum SubnetListError {
    /// Shared bounded transport, verification or observation-time failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The fixed Registry identity could not cross the Agent boundary.
    #[error("invalid Registry identity: {0}")]
    SourceIdentity(String),
    /// Platform refused malformed, chunked or over-budget protocol data.
    #[error(transparent)]
    Codec(#[from] protobuf::Error),
    /// Application error preserved as structured values, without terminal injection.
    #[error("Registry rejected subnet inventory (code {code})")]
    RegistryRejected {
        /// Raw external error code.
        code: i32,
        /// Raw source reason; callers must escape it for terminal presentation.
        reason: String,
        /// Raw source key bytes.
        key: Vec<u8>,
    },
    /// The response did not contain an inline record.
    #[error("Registry returned no subnet-list content")]
    MissingContent,
    /// Empty inventory is not accepted as successful mainnet evidence.
    #[error("Registry returned an empty subnet list")]
    EmptyList,
    /// One source entry was not valid canonical Principal bytes.
    #[error("invalid subnet identity at entry {index}: {source}")]
    InvalidIdentity {
        /// Zero-based position in the source record.
        index: usize,
        /// Canonical identity admission failure.
        #[source]
        source: PrincipalError,
    },
    /// Repeated subnet IDs make the selected inventory ambiguous.
    #[error("Registry subnet list contains a duplicate identity")]
    DuplicateIdentity,
}

/// Read the latest subnet-list record through a bounded anonymous mainnet query.
///
/// Uses the shared 20-second deadline and 1 MiB HTTP ceiling; Platform admits at
/// most 64 KiB of protobuf input and 1024 entries before Query projects identities.
/// Requires a Tokio runtime with IO and time. No persistence or retries are added.
///
/// # Errors
/// Returns typed transport, codec, source and identity failures. Chunked records
/// are refused explicitly. Completeness applies to the returned record only.
pub async fn fetch() -> Result<SubnetListReport, SubnetListError> {
    let argument = protobuf::encode_get_value_request(KEY.as_bytes(), None, 256)?;
    let agent = mainnet::agent()?;
    let registry = mainnet::REGISTRY
        .parse()
        .map_err(|error| SubnetListError::SourceIdentity(format!("{error}")))?;
    let bytes = mainnet::read(agent.query(&registry, METHOD).with_arg(argument).call()).await?;
    project(&bytes, mainnet::observed_at()?)
}

fn project(bytes: &[u8], observed_at: u64) -> Result<SubnetListReport, SubnetListError> {
    let response = protobuf::decode_get_value_response(bytes, LIMITS)?;
    if let Some(error) = response.error {
        return Err(SubnetListError::RegistryRejected {
            code: error.code,
            reason: error.reason.to_owned(),
            key: error.key.to_vec(),
        });
    }
    let entries = protobuf::decode_subnet_list(
        response.value.ok_or(SubnetListError::MissingContent)?,
        LIMITS,
    )?;
    if entries.is_empty() {
        return Err(SubnetListError::EmptyList);
    }
    let mut subnet_ids = Vec::with_capacity(entries.len());
    for (index, bytes) in entries.into_iter().enumerate() {
        subnet_ids.push(
            Principal::try_from_slice(bytes)
                .map_err(|source| SubnetListError::InvalidIdentity { index, source })?,
        );
    }
    subnet_ids.sort_unstable();
    if subnet_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(SubnetListError::DuplicateIdentity);
    }
    Ok(SubnetListReport {
        schema: 1,
        subnet_ids,
        evidence: SubnetListEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: mainnet::REGISTRY,
            source_method: METHOD,
            source_key: KEY,
            observed_at_unix_seconds: observed_at,
            record_version: response.version,
            record_timestamp_nanoseconds: response.timestamp_nanoseconds,
            query_signature_verified: true,
            result_certified: false,
            selected_record_complete: true,
        },
    })
}

#[cfg(test)]
mod tests;
