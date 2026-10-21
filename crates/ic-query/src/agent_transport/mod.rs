//! Private raw query dispatch and structural Agent error inspection.
//!
//! Callers retain codecs, retries, diagnostics and trust policy. Authentication,
//! updates and read-state calls retain their separate existing paths.

#[cfg(feature = "icrc-host")]
use ic_platform::agent::RejectResponse;
#[cfg(feature = "subnet-catalog-host")]
use ic_platform::agent::TransportError;
use ic_platform::agent::{Agent, AgentError};
use ic_platform::principal::ExternalPrincipal;

/// Dispatch one query through the caller's configured Agent, without adding retries.
pub async fn query(
    agent: &Agent,
    canister: &ExternalPrincipal,
    method: &str,
    argument: Vec<u8>,
) -> Result<Vec<u8>, AgentError> {
    agent
        .query(canister, method)
        .with_arg(argument)
        .call()
        .await
}

/// Inspect transport facts; the Registry caller decides whether to retry.
#[cfg(feature = "subnet-catalog-host")]
pub fn is_connect_or_timeout(error: &AgentError) -> bool {
    matches!(error, AgentError::TransportError(TransportError::Reqwest(source))
        if source.is_connect() || source.is_timeout())
}

/// Preserve HTTP status without classifying it as retryable.
#[cfg(feature = "subnet-catalog-host")]
pub const fn http_status(error: &AgentError) -> Option<u16> {
    match error {
        AgentError::HttpError(payload) => Some(payload.status),
        _ => None,
    }
}

/// Read structured rejection fields without interpreting text or authorizing retry.
#[cfg(feature = "icrc-host")]
pub const fn rejection(error: &AgentError) -> Option<&RejectResponse> {
    match error {
        AgentError::CertifiedReject { reject, .. }
        | AgentError::UncertifiedReject { reject, .. } => Some(reject),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
