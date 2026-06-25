use super::hex_bytes;
use crate::ic_registry::{
    RegistryFetchError,
    wire::{RegistryChunk, RegistryGetChunkRequest},
};
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;
use sha2::{Digest, Sha256};

pub(in crate::ic_registry::transport) async fn get_large_registry_value(
    agent: &Agent,
    registry_canister: &Principal,
    chunk_sha256s: &[Vec<u8>],
) -> Result<Vec<u8>, RegistryFetchError> {
    let mut value = Vec::new();
    for chunk_sha256 in chunk_sha256s {
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

pub(in crate::ic_registry) fn append_validated_chunk(
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

pub(in crate::ic_registry) fn sha256_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
