//! Private raw query dispatch with an owned error identity.
//!
//! Callers retain codecs, retries, diagnostics and trust policy. Authentication,
//! updates and read-state calls retain their separate existing paths.

mod error;
pub use error::QueryCallError;
#[cfg(any(test, feature = "icrc-host", feature = "subnet-catalog-host"))]
pub use error::QueryFailure;
use ic_platform::agent::Agent;
use ic_platform::principal::ExternalPrincipal;

/// Dispatch one query through the caller's configured Agent, without adding retries.
pub async fn query(
    agent: &Agent,
    canister: &ExternalPrincipal,
    method: &str,
    argument: Vec<u8>,
) -> Result<Vec<u8>, QueryCallError> {
    agent
        .query(canister, method)
        .with_arg(argument)
        .call()
        .await
        .map_err(QueryCallError::from)
}

#[cfg(test)]
mod tests;
