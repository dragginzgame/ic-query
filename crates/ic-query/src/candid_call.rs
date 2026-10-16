//! Shared Candid query mechanics; callers own agents, endpoints and diagnostic labels.

use candid::{CandidType, Deserialize, Principal};
use ic_agent::{Agent, AgentError};

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
    Args: candid::utils::ArgumentEncoder,
    E: CandidCallError,
{
    candid::encode_args(args).map_err(|err| E::candid_encode(message, err.to_string()))
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
    agent
        .query(canister, method)
        .with_arg(arg)
        .call()
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
    use ciborium::value::Value;
    use ic_testkit::testing::read_http_request;
    use std::{io::Write, net::TcpListener, thread, time::Duration};

    #[tokio::test(flavor = "current_thread")]
    async fn query_preserves_argument_lists_reply_bytes_and_family_error_labels() {
        for (argument, response_message) in
            [(Some(7_u64), "FixtureResponse"), (None, "EmptyResponse")]
        {
            for (reply, expected) in [
                (candid::encode_one(42_u64).unwrap(), Some(42)),
                (vec![0xff], None),
            ] {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let endpoint = format!("http://{}", listener.local_addr().unwrap());
                let response = Value::Map(vec![
                    (Value::Text("status".into()), Value::Text("replied".into())),
                    (
                        Value::Text("reply".into()),
                        Value::Map(vec![(Value::Text("arg".into()), Value::Bytes(reply))]),
                    ),
                ]);
                let mut body = Vec::new();
                ciborium::ser::into_writer(&response, &mut body).unwrap();
                let server = thread::spawn(move || {
                    let (mut stream, _) = listener.accept().unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(10)))
                        .unwrap();
                    let request = read_http_request(
                        &mut stream,
                        Duration::from_secs(10),
                        64 * 1024,
                        64 * 1024,
                    )
                    .unwrap();
                    let body_start = request
                        .windows(4)
                        .position(|bytes| bytes == b"\r\n\r\n")
                        .unwrap()
                        + 4;
                    let request: Value = ciborium::de::from_reader(&request[body_start..]).unwrap();
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                    stream.write_all(&body).unwrap();
                    request
                });
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
                let Value::Tag(55_799, request) = server.join().unwrap() else {
                    panic!("expected self-described CBOR request");
                };
                let content = request
                    .as_map()
                    .unwrap()
                    .iter()
                    .find(|(key, _)| key.as_text() == Some("content"))
                    .unwrap()
                    .1
                    .as_map()
                    .unwrap();
                let captured_argument = &content
                    .iter()
                    .find(|(key, _)| key.as_text() == Some("arg"))
                    .unwrap()
                    .1;
                assert_eq!(captured_argument.as_bytes().unwrap(), &encoded_argument);
                let method = &content
                    .iter()
                    .find(|(key, _)| key.as_text() == Some("method_name"))
                    .unwrap()
                    .1;
                assert_eq!(method.as_text(), Some("fixture_query"));
            }
        }
    }
}
