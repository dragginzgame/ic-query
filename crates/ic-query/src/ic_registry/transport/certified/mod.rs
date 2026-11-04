//! Module: ic_registry::transport::certified
//!
//! Responsibility: authenticate bounded Registry mixed-hash-tree responses.
//! Does not own: catalog delta replay, cache publication, or report rendering.
//! Boundary: accepts a Registry value only after certificate and witness validation.

use super::codec::decode_message;
use crate::{
    certification::{CertifiedDataError, authenticate_canister_tree},
    hex::hex_bytes,
    ic_registry::RegistryFetchError,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_tools::registry::proto::{
    RegistryCertifiedResponse, RegistryMixedHashTree, registry_mixed_hash_tree::Tree,
};
use ic_host_tools::{
    certificate::{DecodeLimits, decode_certificate},
    encoding::decode_canonical_unsigned_u64,
};
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use ic_platform::certification::{
    HashTree, LookupResult, empty, fork, labeled as label, leaf, pruned,
};
use prost::Message;

const GET_CERTIFIED_LATEST_VERSION_METHOD: &str = "get_certified_latest_version";
pub(super) const CURRENT_VERSION_LABEL: &[u8] = b"current_version";
const MAX_MIXED_HASH_TREE_DEPTH: usize = 128;
const MAX_MIXED_HASH_TREE_NODES: usize = 65_536;
const SHA_256_BYTES: usize = 32;

///
/// CertifiedRegistryVersion
///
/// Authenticated low-level evidence returned by one certified Registry query.
///

pub(in crate::ic_registry) struct CertifiedRegistryVersion {
    /// Authenticated Registry version.
    pub(in crate::ic_registry) registry_version: u64,
    /// Certificate and witness evidence established by authentication.
    pub(in crate::ic_registry) certification: crate::nns::registry::NnsRegistryCertification,
}

///
/// AuthenticatedRegistryResponse
///
/// Authenticated common evidence from one certified Registry response.
///

pub(super) struct AuthenticatedRegistryResponse {
    pub(super) hash_tree: HashTree,
    /// Certificate and witness evidence established by authentication.
    pub(super) certification: crate::nns::registry::NnsRegistryCertification,
}

pub(in crate::ic_registry) async fn get_certified_latest_version(
    agent: &Agent,
    registry_canister: &Principal,
) -> Result<CertifiedRegistryVersion, RegistryFetchError> {
    let bytes = crate::agent_transport::query(
        agent,
        registry_canister,
        GET_CERTIFIED_LATEST_VERSION_METHOD,
        Vec::new(),
    )
    .await
    .map_err(|error| RegistryFetchError::AgentCall {
        method: GET_CERTIFIED_LATEST_VERSION_METHOD,
        reason: error.to_string(),
    })?;
    let response =
        decode_message::<RegistryCertifiedResponse>("CertifiedResponse", bytes.as_slice())?;
    verified_certified_registry_version(agent, registry_canister, response)
}

fn verified_certified_registry_version(
    agent: &Agent,
    registry_canister: &Principal,
    response: RegistryCertifiedResponse,
) -> Result<CertifiedRegistryVersion, RegistryFetchError> {
    let authenticated = authenticate_registry_response(
        agent,
        registry_canister,
        response,
        GET_CERTIFIED_LATEST_VERSION_METHOD,
    )?;
    let registry_version = required_leb128_leaf(
        &authenticated.hash_tree,
        CURRENT_VERSION_LABEL,
        "current_version",
    )?;

    Ok(CertifiedRegistryVersion {
        registry_version,
        certification: authenticated.certification,
    })
}

pub(super) fn authenticate_registry_response(
    agent: &Agent,
    registry_canister: &Principal,
    response: RegistryCertifiedResponse,
    method: &'static str,
) -> Result<AuthenticatedRegistryResponse, RegistryFetchError> {
    let raw_hash_tree = response
        .hash_tree
        .ok_or_else(|| invalid_certified_registry(format!("{method} returned no hash_tree")))?;
    let encoded_hash_tree = raw_hash_tree.encode_to_vec();
    let hash_tree = decode_mixed_hash_tree(raw_hash_tree)?;
    let certificate = decode_certificate(
        &response.certificate,
        DecodeLimits {
            bytes: crate::agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES,
        },
    )
    .map_err(|error| invalid_certified_registry(format!("certificate CBOR is invalid: {error}")))?;
    let certificate = authenticate_canister_tree(
        agent,
        registry_canister,
        &certificate,
        &hash_tree,
        "Registry",
    )
    .map_err(map_certified_data_error)?;

    let certificate_time = ic_platform::agent::lookup_value(&certificate, [b"time".as_slice()])
        .map_err(|error| {
            invalid_certified_registry(format!(
                "certificate does not prove its time value: {error}"
            ))
        })?;
    let certificate_time_nanos =
        decode_canonical_unsigned_leb128("certificate time", certificate_time)?;

    Ok(AuthenticatedRegistryResponse {
        hash_tree,
        certification: crate::nns::registry::NnsRegistryCertification {
            certificate_verified: true,
            certificate_time_nanos,
            root_key_digest: Sha256Digest::compute(&agent.read_root_key()).to_string(),
            certificate_hex: hex_bytes(&response.certificate),
            hash_tree_hex: hex_bytes(&encoded_hash_tree),
        },
    })
}

fn decode_mixed_hash_tree(raw: RegistryMixedHashTree) -> Result<HashTree, RegistryFetchError> {
    let mut nodes = 0;
    decode_mixed_hash_tree_node(raw, 0, &mut nodes)
}

fn decode_mixed_hash_tree_node(
    raw: RegistryMixedHashTree,
    depth: usize,
    nodes: &mut usize,
) -> Result<HashTree, RegistryFetchError> {
    if depth > MAX_MIXED_HASH_TREE_DEPTH {
        return Err(invalid_certified_registry(format!(
            "mixed hash tree exceeds the maximum depth of {MAX_MIXED_HASH_TREE_DEPTH}"
        )));
    }
    *nodes = nodes.saturating_add(1);
    if *nodes > MAX_MIXED_HASH_TREE_NODES {
        return Err(invalid_certified_registry(format!(
            "mixed hash tree exceeds the maximum node count of {MAX_MIXED_HASH_TREE_NODES}"
        )));
    }

    match raw
        .tree
        .ok_or_else(|| invalid_certified_registry("mixed hash tree node is empty"))?
    {
        Tree::Empty(()) => Ok(empty()),
        Tree::Fork(branch) => {
            let left = required_child(branch.left_tree, "fork.left_tree", depth, nodes)?;
            let right = required_child(branch.right_tree, "fork.right_tree", depth, nodes)?;
            Ok(fork(left, right))
        }
        Tree::Labeled(branch) => {
            let subtree = required_child(branch.subtree, "labeled.subtree", depth, nodes)?;
            Ok(label(branch.label, subtree))
        }
        Tree::LeafData(value) => Ok(leaf(value)),
        Tree::PrunedDigest(value) => {
            let digest: [u8; SHA_256_BYTES] = value.try_into().map_err(|value: Vec<u8>| {
                invalid_certified_registry(format!(
                    "pruned digest is {} bytes; expected {SHA_256_BYTES}",
                    value.len()
                ))
            })?;
            Ok(pruned(digest))
        }
    }
}

fn required_child(
    child: Option<Box<RegistryMixedHashTree>>,
    field: &str,
    depth: usize,
    nodes: &mut usize,
) -> Result<HashTree, RegistryFetchError> {
    let child = child
        .ok_or_else(|| invalid_certified_registry(format!("mixed hash tree {field} is missing")))?;
    decode_mixed_hash_tree_node(*child, depth.saturating_add(1), nodes)
}

pub(super) fn required_leb128_leaf(
    hash_tree: &HashTree,
    label: &[u8],
    field: &str,
) -> Result<u64, RegistryFetchError> {
    match hash_tree.lookup_path([label]) {
        LookupResult::Found(value) => decode_canonical_unsigned_leb128(field, value),
        LookupResult::Absent => Err(invalid_certified_registry(format!(
            "{field} leaf is absent"
        ))),
        LookupResult::Unknown => Err(invalid_certified_registry(format!(
            "{field} leaf is not proven by the partial tree"
        ))),
        LookupResult::Error => Err(invalid_certified_registry(format!(
            "{field} path does not identify a leaf"
        ))),
    }
}

fn decode_canonical_unsigned_leb128(field: &str, bytes: &[u8]) -> Result<u64, RegistryFetchError> {
    decode_canonical_unsigned_u64(bytes)
        .map_err(|error| invalid_certified_registry(format!("{field}: {error}")))
}

fn map_certified_data_error(error: CertifiedDataError) -> RegistryFetchError {
    match error {
        CertifiedDataError::Authentication { reason } => {
            RegistryFetchError::CertificateAuthentication { reason }
        }
        CertifiedDataError::Invalid { reason } => invalid_certified_registry(reason),
    }
}

pub(super) fn invalid_certified_registry(reason: impl Into<String>) -> RegistryFetchError {
    RegistryFetchError::InvalidCertifiedRegistry {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests;
