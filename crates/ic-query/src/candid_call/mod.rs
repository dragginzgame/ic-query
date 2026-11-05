//! Shared Candid query mechanics; callers own agents, endpoints and diagnostic labels.

use ic_platform::agent::Agent;
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
    fn agent_call(method: &'static str, error: crate::agent_transport::QueryCallError) -> Self;

    /// Project a bounded reply decoding failure.
    fn candid_decode(message: &'static str, reason: String) -> Self;
}

// These families expose the same error variants while retaining distinct public error types.
macro_rules! impl_candid_call_error {
    ($error:ty) => {
        impl $crate::candid_call::CandidCallError for $error {
            fn candid_encode(message: &'static str, reason: String) -> Self {
                Self::CandidEncode { message, reason }
            }

            fn agent_call(
                method: &'static str,
                error: $crate::agent_transport::QueryCallError,
            ) -> Self {
                Self::AgentCall {
                    method,
                    reason: error.to_string(),
                }
            }

            fn candid_decode(message: &'static str, reason: String) -> Self {
                Self::CandidDecode { message, reason }
            }
        }
    };
}

pub(crate) use impl_candid_call_error;

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
#[cfg(feature = "icrc-host")]
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
        feature = "cmc-host",
        feature = "nns-host",
        feature = "sns-host"
    )
))]
mod tests;
