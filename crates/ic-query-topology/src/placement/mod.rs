//! A single canister's current routing placement according to the NNS Registry.
//!
//! This report does not establish canister existence, liveness, capacity or a
//! consensus-certified Registry snapshot. Transport admits only mainnet, using
//! Platform's built-in trust root and query-signature verification.

pub mod collection;
#[cfg(test)]
mod tests;

use ic_platform_protocol::{
    candid as decoder,
    cloud_engine::EncodeError,
    placement::{
        PlacementResult, decode_get_subnet_for_canister_response,
        encode_get_subnet_for_canister_args,
    },
};

use ic_platform_types::Principal;
use serde::Serialize;
use thiserror::Error;

use crate::{
    ReadError,
    mainnet::{self, REGISTRY},
};

const METHOD: &str = "get_subnet_for_canister";
const REPLY_LIMITS: decoder::Limits = decoder::Limits {
    bytes: 64 * 1024,
    header: 4096,
    types: 64,
    work: 256 * 1024,
    allocation: 128 * 1024,
    vector: 1024,
    skipping: 128 * 1024,
};

/// Complete observation for one selected canister, with explicit source evidence.
#[derive(Debug, Serialize)]
pub struct PlacementReport {
    /// Query-owned report schema; remains 1 before 1.0.
    pub schema: u8,
    /// Exactly the principal requested by the consumer.
    pub canister_id: Principal,
    /// Subnet returned by the Registry; does not prove canister existence.
    pub subnet_id: Principal,
    /// Source and observation claims for this lookup only.
    pub evidence: PlacementEvidence,
}

/// Evidence accompanying one successful Registry lookup.
#[derive(Debug, Serialize)]
pub struct PlacementEvidence {
    /// Admitted mainnet HTTPS endpoint.
    pub endpoint: &'static str,
    /// Mainnet Registry canister queried.
    pub source_canister: &'static str,
    /// Explicit source query method.
    pub source_method: &'static str,
    /// Local observation completion time, in raw Unix seconds.
    pub observed_at_unix_seconds: u64,
    /// Platform checked the query response signature using authenticated node keys.
    pub query_signature_verified: bool,
    /// False: the query result is not a consensus-certified Registry snapshot.
    pub result_certified: bool,
    /// None: this method does not return a Registry version.
    pub registry_version: Option<u64>,
    /// True for the one selected lookup only, never for network-wide coverage.
    pub selected_lookup_complete: bool,
}

/// Typed failures; no failed lookup produces a partial report.
#[derive(Debug, Error)]
pub enum PlacementError {
    /// The fixed source identity could not be parsed.
    #[error("invalid Registry identity: {0}")]
    SourceIdentity(String),
    /// Mainnet transport, verification, deadline or observation-clock failure.
    #[error(transparent)]
    Read(#[from] ReadError),
    /// The bounded request encoder refused the request.
    #[error("Registry request encoding failed: {0:?}")]
    Encode(EncodeError),
    /// The bounded reply decoder refused malformed, incompatible or excessive data.
    #[error("Registry reply decoding failed: {0:?}")]
    Decode(decoder::Error),
    /// The Registry returned an application error, preserved without interpretation.
    #[error("Registry rejected the lookup: {0}")]
    RegistryRejected(String),
    /// A successful source envelope did not identify a subnet.
    #[error("Registry returned no subnet for the selected canister")]
    MissingSubnet,
}

/// Query mainnet for one canister's placement, without persistence or mutations.
///
/// Requires a Tokio runtime with IO and time enabled. Limits and trust admission
/// are fixed for this initial operation; no custom endpoint or trust-root override
/// is accepted. Transport retries are disabled; all verification reads share the
/// overall query deadline.
///
/// # Errors
/// Returns a typed failure for transport/verification, deadlines, codec limits,
/// Registry rejection, missing subnet evidence or an invalid local clock.
pub async fn fetch(canister_id: Principal) -> Result<PlacementReport, PlacementError> {
    let argument =
        encode_get_subnet_for_canister_args(canister_id, 256).map_err(PlacementError::Encode)?;
    let client = mainnet::client()?;
    let registry = REGISTRY
        .parse()
        .map_err(|error| PlacementError::SourceIdentity(format!("{error}")))?;
    let bytes = mainnet::query(&client, registry, METHOD, &argument).await?;
    let subnet_id = decode_subnet(&bytes)?;
    let observed_at = mainnet::observed_at()?;
    Ok(report(canister_id, subnet_id, observed_at))
}

fn decode_subnet(bytes: &[u8]) -> Result<Principal, PlacementError> {
    match decode_get_subnet_for_canister_response(bytes, REPLY_LIMITS)
        .map_err(PlacementError::Decode)?
    {
        PlacementResult::Ok(value) => value.subnet_id.ok_or(PlacementError::MissingSubnet),
        PlacementResult::Err(message) => Err(PlacementError::RegistryRejected(message)),
    }
}

fn report(canister_id: Principal, subnet_id: Principal, observed_at: u64) -> PlacementReport {
    PlacementReport {
        schema: 1,
        canister_id,
        subnet_id,
        evidence: PlacementEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: REGISTRY,
            source_method: METHOD,
            observed_at_unix_seconds: observed_at,
            query_signature_verified: true,
            result_certified: false,
            registry_version: None,
            selected_lookup_complete: true,
        },
    }
}
