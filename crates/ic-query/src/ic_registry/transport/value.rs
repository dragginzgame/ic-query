use super::{
    RegistryQueryCounter,
    chunk::{RegistryChunkBudget, RegistryChunkLimits, get_large_registry_value},
    decode_message,
};
use crate::ic_registry::{
    RegistryFetchError,
    proto::{
        RegistryErrorCode, RegistryGetValueRequest, RegistryGetValueResponse, UInt64Value,
        registry_get_value_response,
    },
    wire::{
        RegistryValueContent, RegistryValueEncoding, RegistryVersionedValue,
        RegistryVersionedValueFailure,
    },
};
use ic_agent::Agent;
use prost::Message;

#[cfg(feature = "nns-topology-host")]
pub(in crate::ic_registry) async fn get_registry_value(
    agent: &Agent,
    registry_canister: &candid::Principal,
    key: &str,
    version: u64,
) -> Result<Vec<u8>, RegistryFetchError> {
    get_registry_versioned_value_inner(agent, registry_canister, key, version, None)
        .await
        .map(|value| value.value)
        .map_err(|failure| failure.source)
}

pub(in crate::ic_registry) async fn get_registry_versioned_value_counted(
    agent: &Agent,
    registry_canister: &candid::Principal,
    key: &str,
    version: u64,
    counter: &RegistryQueryCounter,
) -> Result<RegistryVersionedValue, RegistryVersionedValueFailure> {
    get_registry_versioned_value_inner(agent, registry_canister, key, version, Some(counter)).await
}

async fn get_registry_versioned_value_inner(
    agent: &Agent,
    registry_canister: &candid::Principal,
    key: &str,
    version: u64,
    counter: Option<&RegistryQueryCounter>,
) -> Result<RegistryVersionedValue, RegistryVersionedValueFailure> {
    let request = RegistryGetValueRequest {
        version: Some(UInt64Value { value: version }),
        key: key.as_bytes().to_vec(),
    };
    let mut arg = Vec::new();
    request.encode(&mut arg).map_err(|err| {
        value_failure(RegistryFetchError::ProtobufEncode {
            message: "RegistryGetValueRequest",
            reason: err.to_string(),
        })
    })?;
    if let Some(counter) = counter {
        counter.record_call();
    }
    let bytes = agent
        .query(registry_canister, "get_value")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| {
            value_failure(RegistryFetchError::AgentCall {
                method: "get_value",
                reason: err.to_string(),
            })
        })?;
    let response = decode_message::<RegistryGetValueResponse>("RegistryGetValueResponse", &bytes)
        .map_err(value_failure)?;
    let returned_version = response.version;
    let timestamp_nanoseconds = response.timestamp_nanoseconds;
    match registry_value_content_from_response(key, response).map_err(|source| {
        RegistryVersionedValueFailure {
            source,
            returned_version: Some(returned_version),
        }
    })? {
        RegistryValueContent::Value(value) => Ok(RegistryVersionedValue {
            value,
            version: returned_version,
            timestamp_nanoseconds,
            encoding: RegistryValueEncoding::Inline,
        }),
        RegistryValueContent::LargeValueChunkKeys(keys) => {
            let mut budget = RegistryChunkBudget::new(RegistryChunkLimits::ordinary_value(), 0)
                .map_err(|source| RegistryVersionedValueFailure {
                    source,
                    returned_version: Some(returned_version),
                })?;
            let value = get_large_registry_value(
                agent,
                registry_canister,
                &keys.chunk_content_sha256s,
                counter,
                &mut budget,
            )
            .await
            .map_err(|source| RegistryVersionedValueFailure {
                source,
                returned_version: Some(returned_version),
            })?;
            Ok(RegistryVersionedValue {
                value,
                version: returned_version,
                timestamp_nanoseconds,
                encoding: RegistryValueEncoding::Chunked,
            })
        }
    }
}

const fn value_failure(source: RegistryFetchError) -> RegistryVersionedValueFailure {
    RegistryVersionedValueFailure {
        source,
        returned_version: None,
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
