//! Shared Candid query mechanics; callers own agents, endpoints and diagnostic labels.

use candid::{CandidType, Deserialize, Encode, Principal};
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

/// Query a method accepting an empty Candid argument list.
pub async fn query_no_args<T, E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
) -> Result<T, E>
where
    E: CandidCallError,
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = Encode!().map_err(|err| E::candid_encode(method, err.to_string()))?;
    query_encoded(agent, canister, method, arg).await
}

/// Query a method with one argument and one bounded decoded reply.
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
    let bytes = query_bytes(agent, canister, method, request_message, arg).await?;
    crate::candid_decode::decode_reply(&bytes)
        .map_err(|err| E::candid_decode(response_message, err.to_string()))
}

/// Queries one Candid method while leaving response decoding to the caller.
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
    let arg = candid::encode_one(arg)
        .map_err(|err| E::candid_encode(request_message, err.to_string()))?;
    query_encoded_bytes(agent, canister, method, arg).await
}

async fn query_encoded<T, E>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    arg: Vec<u8>,
) -> Result<T, E>
where
    E: CandidCallError,
    T: for<'de> Deserialize<'de> + CandidType,
{
    let bytes = query_encoded_bytes(agent, canister, method, arg).await?;
    crate::candid_decode::decode_reply(&bytes)
        .map_err(|err| E::candid_decode(method, err.to_string()))
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

#[cfg(all(test, feature = "sns-host"))]
mod tests {
    use super::*;
    use crate::sns::SnsHostError;
    use ciborium::value::Value;
    use ic_testkit::testing::read_http_request;
    use std::{io::Write, net::TcpListener, thread, time::Duration};

    #[tokio::test(flavor = "current_thread")]
    async fn query_preserves_one_argument_reply_bytes_and_family_error_labels() {
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
                let request =
                    read_http_request(&mut stream, Duration::from_secs(10), 64 * 1024, 64 * 1024)
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
            let result = query_canister::<_, u64, SnsHostError>(
                &agent,
                &Principal::anonymous(),
                "fixture_query",
                "FixtureRequest",
                "FixtureResponse",
                &7_u64,
            )
            .await;
            match expected {
                Some(expected) => assert_eq!(result.unwrap(), expected),
                None => assert!(matches!(
                    result,
                    Err(SnsHostError::CandidDecode {
                        message: "FixtureResponse",
                        ..
                    })
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
            let argument = &content
                .iter()
                .find(|(key, _)| key.as_text() == Some("arg"))
                .unwrap()
                .1;
            assert_eq!(
                argument.as_bytes().unwrap(),
                &candid::encode_one(7_u64).unwrap()
            );
            let method = &content
                .iter()
                .find(|(key, _)| key.as_text() == Some("method_name"))
                .unwrap()
                .1;
            assert_eq!(method.as_text(), Some("fixture_query"));
        }
    }
}
