use super::SnsHostError;
use candid::{CandidType, Deserialize, Encode, Principal};
use ic_agent::Agent;

pub(super) async fn query_canister<Arg, Response>(
    agent: &Agent,
    canister: &Principal,
    method: &'static str,
    request_message: &'static str,
    response_message: &'static str,
    arg: &Arg,
) -> Result<Response, SnsHostError>
where
    Arg: CandidType + Sync,
    Response: for<'de> Deserialize<'de> + CandidType,
{
    let arg = candid::encode_one(arg).map_err(|err| SnsHostError::CandidEncode {
        message: request_message,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: response_message,
        reason: err.to_string(),
    })
}

pub(super) async fn query_ledger<T>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
) -> Result<T, SnsHostError>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = Encode!().map_err(|err| SnsHostError::CandidEncode {
        message: method,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(ledger_canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: method,
        reason: err.to_string(),
    })
}

pub(super) fn sns_agent(endpoint: &str) -> Result<Agent, SnsHostError> {
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| SnsHostError::AgentBuild {
            endpoint: endpoint.to_string(),
            reason: err.to_string(),
        })
}

pub(super) fn principal_from_text(
    value: &str,
    field: &'static str,
) -> Result<Principal, SnsHostError> {
    Principal::from_text(value).map_err(|err| SnsHostError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}
