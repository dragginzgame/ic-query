//! Module: ic_registry::transport::chunk
//!
//! Responsibility: retrieve and hash-verify explicitly bounded Registry value chunks.
//! Does not own: Registry delta parsing, report projection, or cache policy.
//! Boundary: no caller can reconstruct an unbounded large Registry value.

use super::{RegistryQueryCounter, hex_bytes};
use crate::ic_registry::{
    RegistryFetchError,
    wire::{RegistryChunk, RegistryGetChunkRequest},
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::agent::Agent;
use ic_platform::candid::contract::Principal;
use std::collections::BTreeMap;

const REGISTRY_CHUNK_SHA256_BYTES: usize = 32;
/// Maximum chunk references accepted in one bounded collection.
pub const MAX_REGISTRY_CHUNK_REFERENCES: usize = 64;
/// Maximum decoded content bytes accepted from one Registry chunk.
pub const MAX_REGISTRY_CHUNK_BYTES: usize = 1_800_000;
/// Maximum bytes reconstructed for one Registry value.
pub const MAX_REGISTRY_RECONSTRUCTED_VALUE_BYTES: usize = 10 * 1_024 * 1_024;
/// Maximum encoded response bytes accepted across Registry chunk calls.
pub const MAX_REGISTRY_CHUNK_RESPONSE_BYTES: usize = 32 * 1_024 * 1_024;

///
/// RegistryChunkLimits
///
/// Explicit call, response, and reconstructed-value ceilings for one collection.
///

#[derive(Clone, Copy)]
pub(in crate::ic_registry) struct RegistryChunkLimits {
    pub(in crate::ic_registry) references: usize,
    pub(in crate::ic_registry) chunk_bytes: usize,
    pub(in crate::ic_registry) value_bytes: usize,
    pub(in crate::ic_registry) total_value_bytes: usize,
    pub(in crate::ic_registry) response_bytes: usize,
}

impl RegistryChunkLimits {
    pub(in crate::ic_registry) const fn ordinary_value() -> Self {
        Self {
            references: MAX_REGISTRY_CHUNK_REFERENCES,
            chunk_bytes: MAX_REGISTRY_CHUNK_BYTES,
            value_bytes: MAX_REGISTRY_RECONSTRUCTED_VALUE_BYTES,
            total_value_bytes: MAX_REGISTRY_RECONSTRUCTED_VALUE_BYTES,
            response_bytes: MAX_REGISTRY_CHUNK_RESPONSE_BYTES,
        }
    }
}

///
/// RegistryChunkBudget
///
/// Mutable accounting and content-addressed reuse for one bounded chunk collection.
///

pub(in crate::ic_registry) struct RegistryChunkBudget {
    limits: RegistryChunkLimits,
    reference_count: usize,
    query_call_count: usize,
    response_bytes: usize,
    reconstructed_value_bytes: usize,
    cache: BTreeMap<[u8; REGISTRY_CHUNK_SHA256_BYTES], Vec<u8>>,
}

impl RegistryChunkBudget {
    pub(in crate::ic_registry) fn new(
        limits: RegistryChunkLimits,
        initial_value_bytes: usize,
    ) -> Result<Self, RegistryFetchError> {
        enforce_limit(
            "reconstructed value bytes",
            initial_value_bytes,
            limits.total_value_bytes,
        )?;
        Ok(Self {
            limits,
            reference_count: 0,
            query_call_count: 0,
            response_bytes: 0,
            reconstructed_value_bytes: initial_value_bytes,
            cache: BTreeMap::new(),
        })
    }

    fn validated_hashes(
        &mut self,
        hashes: &[Vec<u8>],
    ) -> Result<Vec<[u8; REGISTRY_CHUNK_SHA256_BYTES]>, RegistryFetchError> {
        let reference_count = checked_total(
            "reference count",
            self.reference_count,
            hashes.len(),
            self.limits.references,
        )?;
        let hashes = validated_chunk_hashes(hashes)?;
        self.reference_count = reference_count;
        Ok(hashes)
    }

    fn append_cached_chunk(
        &mut self,
        value: &mut Vec<u8>,
        hash: &[u8; REGISTRY_CHUNK_SHA256_BYTES],
    ) -> Result<bool, RegistryFetchError> {
        let Some(content) = self.cache.get(hash) else {
            return Ok(false);
        };
        append_reconstructed_bytes(
            value,
            content,
            &mut self.reconstructed_value_bytes,
            self.limits,
        )?;
        Ok(true)
    }

    fn record_query_response(&mut self, response_bytes: usize) -> Result<(), RegistryFetchError> {
        let query_call_count = checked_total(
            "query call count",
            self.query_call_count,
            1,
            self.limits.references,
        )?;
        let total_response_bytes = checked_total(
            "response bytes",
            self.response_bytes,
            response_bytes,
            self.limits.response_bytes,
        )?;
        self.query_call_count = query_call_count;
        self.response_bytes = total_response_bytes;
        Ok(())
    }

    fn append_fetched_chunk(
        &mut self,
        value: &mut Vec<u8>,
        expected_sha256: [u8; REGISTRY_CHUNK_SHA256_BYTES],
        chunk_content: Vec<u8>,
    ) -> Result<(), RegistryFetchError> {
        enforce_limit(
            "content bytes",
            chunk_content.len(),
            self.limits.chunk_bytes,
        )?;
        validate_chunk_hash(&expected_sha256, &chunk_content)?;
        append_reconstructed_bytes(
            value,
            &chunk_content,
            &mut self.reconstructed_value_bytes,
            self.limits,
        )?;
        self.cache.insert(expected_sha256, chunk_content);
        Ok(())
    }
}

pub(in crate::ic_registry::transport) fn validated_chunk_hashes(
    hashes: &[Vec<u8>],
) -> Result<Vec<[u8; REGISTRY_CHUNK_SHA256_BYTES]>, RegistryFetchError> {
    if hashes.is_empty() {
        return Err(RegistryFetchError::EmptyRegistryChunkList);
    }
    hashes
        .iter()
        .map(|hash| {
            hash.as_slice()
                .try_into()
                .map_err(|_| RegistryFetchError::InvalidRegistryChunkDigest {
                    actual_bytes: hash.len(),
                })
        })
        .collect()
}

pub(in crate::ic_registry::transport) async fn get_large_registry_value(
    agent: &Agent,
    registry_canister: &Principal,
    chunk_sha256s: &[Vec<u8>],
    counter: Option<&RegistryQueryCounter>,
    budget: &mut RegistryChunkBudget,
) -> Result<Vec<u8>, RegistryFetchError> {
    let hashes = budget.validated_hashes(chunk_sha256s)?;
    let mut value = Vec::new();
    for hash in hashes {
        if budget.append_cached_chunk(&mut value, &hash)? {
            continue;
        }
        let (chunk_content, response_bytes) =
            get_registry_chunk(agent, registry_canister, &hash, counter).await?;
        budget.record_query_response(response_bytes)?;
        budget.append_fetched_chunk(&mut value, hash, chunk_content)?;
    }
    enforce_limit("value bytes", value.len(), budget.limits.value_bytes)?;
    Ok(value)
}

async fn get_registry_chunk(
    agent: &Agent,
    registry_canister: &Principal,
    content_sha256: &[u8],
    counter: Option<&RegistryQueryCounter>,
) -> Result<(Vec<u8>, usize), RegistryFetchError> {
    let request = RegistryGetChunkRequest {
        content_sha256: Some(content_sha256.to_vec()),
    };
    let arg = ic_platform::candid::contract::encode_args((&request,)).map_err(|err| {
        RegistryFetchError::CandidEncode {
            message: "RegistryGetChunkRequest",
            reason: err.to_string(),
        }
    })?;
    let bytes = super::query::query(agent, registry_canister, "get_chunk", arg, counter)
        .await
        .map_err(|err| RegistryFetchError::AgentCall {
            method: "get_chunk",
            reason: err.to_string(),
        })?;
    let response_bytes = bytes.len();
    let result = crate::candid_decode::decode_reply::<Result<RegistryChunk, String>>(&bytes)
        .map_err(|err| RegistryFetchError::CandidDecode {
            message: "Result<RegistryChunk, String>",
            reason: err.to_string(),
        })?;
    match result {
        Ok(chunk) => chunk
            .content
            .map(|content| (content, response_bytes))
            .ok_or_else(|| RegistryFetchError::MissingChunkContent {
                sha256: hex_bytes(content_sha256),
            }),
        Err(reason) => Err(RegistryFetchError::RegistryChunkRejected {
            sha256: hex_bytes(content_sha256),
            reason,
        }),
    }
}

fn append_reconstructed_bytes(
    value: &mut Vec<u8>,
    content: &[u8],
    total_value_bytes: &mut usize,
    limits: RegistryChunkLimits,
) -> Result<(), RegistryFetchError> {
    checked_total(
        "value bytes",
        value.len(),
        content.len(),
        limits.value_bytes,
    )?;
    let reconstructed_value_bytes = checked_total(
        "reconstructed value bytes",
        *total_value_bytes,
        content.len(),
        limits.total_value_bytes,
    )?;
    value.extend_from_slice(content);
    *total_value_bytes = reconstructed_value_bytes;
    Ok(())
}

fn validate_chunk_hash(
    expected_sha256: &[u8; REGISTRY_CHUNK_SHA256_BYTES],
    chunk_content: &[u8],
) -> Result<(), RegistryFetchError> {
    let actual_sha256 = sha256_digest(chunk_content);
    if &actual_sha256 != expected_sha256 {
        return Err(RegistryFetchError::ChunkHashMismatch {
            sha256: hex_bytes(expected_sha256),
            actual_sha256: hex_bytes(&actual_sha256),
        });
    }
    Ok(())
}

fn checked_total(
    field: &'static str,
    current: usize,
    increment: usize,
    maximum: usize,
) -> Result<usize, RegistryFetchError> {
    let total = current
        .checked_add(increment)
        .ok_or(RegistryFetchError::RegistryChunkLimit {
            field,
            maximum,
            actual: usize::MAX,
        })?;
    enforce_limit(field, total, maximum)?;
    Ok(total)
}

const fn enforce_limit(
    field: &'static str,
    actual: usize,
    maximum: usize,
) -> Result<(), RegistryFetchError> {
    if actual > maximum {
        Err(RegistryFetchError::RegistryChunkLimit {
            field,
            maximum,
            actual,
        })
    } else {
        Ok(())
    }
}

pub(in crate::ic_registry) fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    *Sha256Digest::compute(bytes).as_bytes()
}

#[cfg(test)]
mod tests;
