//! Shared Candid query mechanics; callers own agents, endpoints and diagnostic labels.

use ic_platform::agent::{Agent, AgentError};
use ic_platform::candid::contract::{CandidType, Principal, serde::Deserialize};

///
/// CandidCallError
///
/// Error adapter for bounded Candid queries with family-owned diagnostics.
///

pub trait CandidCallError: Sized {
    /// Project a request encoding failure.
    fn candid_encode(message: &'static str, reason: String) -> Self;

    /// Preserve the original Agent error for family-specific classification.
    fn agent_call(method: &'static str, error: AgentError) -> Self;

    /// Project a bounded reply decoding failure.
    fn candid_decode(message: &'static str, reason: String) -> Self;
}

/// Encode a Candid argument list before admitting a family-owned agent.
pub fn encode_request<Args, E>(args: Args, message: &'static str) -> Result<Vec<u8>, E>
where
    Args: ic_platform::candid::contract::ArgumentEncoder,
    E: CandidCallError,
{
    ic_platform::candid::contract::encode_args(args)
        .map_err(|err| E::candid_encode(message, err.to_string()))
}

/// Query a method accepting an empty Candid argument list.
#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
pub async fn query_no_args<T, E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
) -> Result<T, E>
where
    E: CandidCallError,
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = encode_request((), method)?;
    query_encoded(agent, canister, method, method, arg).await
}

/// Query a method with one argument and one bounded decoded reply.
#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
pub async fn query_canister<Arg, Response, E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, E>
where
    Arg: CandidType + Sync,
    E: CandidCallError,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = encode_request((arg,), request_message)?;
    query_encoded(agent, canister, method, response_message, arg).await
}

/// Queries one Candid method while leaving response decoding to the caller.
#[cfg(feature = "icrc-host")]
pub async fn query_bytes<Arg, E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    arg: &Arg,
) -> Result<Vec<u8>, E>
where
    Arg: CandidType + Sync,
    E: CandidCallError,
{
    let arg = encode_request((arg,), request_message)?;
    query_encoded_bytes(agent, canister, method, arg).await
}

/// Query pre-encoded arguments and decode one bounded reply with the caller's label.
pub async fn query_encoded<T, E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    response_message: &'static str,
    arg: Vec<u8>,
) -> Result<T, E>
where
    E: CandidCallError,
    T: for<'de> Deserialize<'de> + CandidType,
{
    let bytes = query_encoded_bytes(agent, canister, method, arg).await?;
    crate::candid_decode::decode_reply(&bytes)
        .map_err(|err| E::candid_decode(response_message, err.to_string()))
}

async fn query_encoded_bytes<E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    arg: Vec<u8>,
) -> Result<Vec<u8>, E>
where
    E: CandidCallError,
{
    crate::agent_transport::query(agent, canister, method, arg)
        .await
        .map_err(|err| E::agent_call(method, err))
}

#[cfg(all(
    test,
    any(
        feature = "cloud-engine-host",
        feature = "nns-host",
        feature = "sns-host"
    )
))]
mod tests {
    use super::*;
    #[cfg(all(
        not(feature = "nns-host"),
        not(feature = "sns-host"),
        feature = "cloud-engine-host"
    ))]
    use crate::cloud_engine::CloudEngineHostError as QueryError;
    #[cfg(feature = "nns-host")]
    use crate::nns::governance::NnsGovernanceError as QueryError;
    #[cfg(all(not(feature = "nns-host"), feature = "sns-host"))]
    use crate::sns::SnsHostError as QueryError;
    use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
    use serde_cbor::Value;
    use std::{collections::BTreeMap, time::Duration};

    #[tokio::test(flavor = "current_thread")]
    async fn query_preserves_argument_lists_reply_bytes_and_family_error_labels() {
        for (argument, response_message) in
            [(Some(7_u64), "FixtureResponse"), (None, "EmptyResponse")]
        {
            for (reply, expected) in [
                (
                    ic_platform::candid::contract::encode_one(42_u64).unwrap(),
                    Some(42),
                ),
                (vec![0xff], None),
            ] {
                let response = Value::Map(BTreeMap::from([
                    (Value::Text("status".into()), Value::Text("replied".into())),
                    (
                        Value::Text("reply".into()),
                        Value::Map(BTreeMap::from([(
                            Value::Text("arg".into()),
                            Value::Bytes(reply),
                        )])),
                    ),
                ]));
                let body = serde_cbor::to_vec(&response).unwrap();
                let server = HttpFixture::spawn(
                    &body,
                    "application/cbor",
                    HttpFixtureLimits {
                        timeout: Duration::from_secs(10),
                        max_header_bytes: 64 * 1024,
                        max_body_bytes: 64 * 1024,
                    },
                )
                .unwrap();
                let endpoint = server.endpoint();
                let agent = Agent::builder()
                    .with_url(endpoint)
                    .with_verify_query_signatures(false)
                    .build()
                    .unwrap();
                let encoded_argument = argument
                    .map_or_else(
                        || encode_request::<_, QueryError>((), "()"),
                        |value| encode_request((value,), "FixtureRequest"),
                    )
                    .unwrap();
                let result: Result<u64, QueryError> = query_encoded(
                    &agent,
                    &Principal::anonymous(),
                    "fixture_query",
                    response_message,
                    encoded_argument.clone(),
                )
                .await;
                match expected {
                    Some(expected) => assert_eq!(result.unwrap(), expected),
                    None => assert!(matches!(
                        result,
                        Err(QueryError::CandidDecode {
                            message,
                            ..
                        }) if message == response_message
                    )),
                }
                let raw = server.finish().unwrap();
                let body_start = raw
                    .windows(4)
                    .position(|bytes| bytes == b"\r\n\r\n")
                    .unwrap()
                    + 4;
                assert!(raw[body_start..].starts_with(&[0xd9, 0xd9, 0xf7]));
                let Value::Map(request) =
                    serde_cbor::from_slice::<Value>(&raw[body_start..]).unwrap()
                else {
                    panic!("expected CBOR request envelope");
                };
                let Value::Map(content) = &request[&Value::Text("content".into())] else {
                    panic!("expected CBOR request content");
                };
                assert_eq!(
                    content[&Value::Text("arg".into())],
                    Value::Bytes(encoded_argument)
                );
                assert_eq!(
                    content[&Value::Text("method_name".into())],
                    Value::Text("fixture_query".into())
                );
            }
        }
    }
}
