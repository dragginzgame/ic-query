use super::{chunk::get_large_registry_value, decode_message};
use crate::ic_registry::{
    RegistryFetchError,
    proto::{
        RegistryErrorCode, RegistryGetValueRequest, RegistryGetValueResponse, UInt64Value,
        registry_get_value_response,
    },
    wire::RegistryValueContent,
};
use ic_agent::Agent;
use prost::Message;

pub(in crate::ic_registry) async fn get_registry_value(
    agent: &Agent,
    registry_canister: &candid::Principal,
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
            get_large_registry_value(agent, registry_canister, &keys.chunk_content_sha256s).await
        }
    }
}

pub(in crate::ic_registry) fn registry_value_content_from_response(
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
