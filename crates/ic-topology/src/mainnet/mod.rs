//! Shared admission and limits for the implemented mainnet reports.

use std::future::Future;
use std::time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH};

use ic_platform::agent::{Agent, AgentError, AnonymousIdentity};
use thiserror::Error;

pub(crate) const ENDPOINT: &str = "https://ic0.app";
const DEADLINE: Duration = Duration::from_secs(20);
const HTTP_BODY_BYTES: usize = 1024 * 1024;
const INGRESS_EXPIRY: Duration = Duration::from_secs(300);

/// Shared failures from bounded mainnet reads and observation timing.
#[derive(Debug, Error)]
pub enum ReadError {
    /// Agent construction, transport, certificate or query verification failed.
    #[error("mainnet transport or verification failed: {0}")]
    Transport(#[source] Box<AgentError>),
    /// The asynchronous operation exceeded its overall deadline.
    #[error("mainnet read exceeded its 20-second deadline")]
    Deadline,
    /// The local clock could not supply a Unix observation time.
    #[error("unable to record observation time: {0}")]
    Clock(#[source] SystemTimeError),
}

pub(crate) fn agent() -> Result<Agent, ReadError> {
    Agent::builder()
        .with_url(ENDPOINT)
        .with_identity(AnonymousIdentity)
        .with_verify_query_signatures(true)
        .with_ingress_expiry(INGRESS_EXPIRY)
        .with_max_tcp_error_retries(0)
        .with_max_response_body_size(HTTP_BODY_BYTES)
        .build()
        .map_err(|error| ReadError::Transport(Box::new(error)))
}

pub(crate) async fn read<T>(
    operation: impl Future<Output = Result<T, AgentError>>,
) -> Result<T, ReadError> {
    tokio::time::timeout(DEADLINE, operation)
        .await
        .map_err(|_| ReadError::Deadline)?
        .map_err(|error| ReadError::Transport(Box::new(error)))
}

pub(crate) fn observed_at() -> Result<u64, ReadError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(ReadError::Clock)
}

#[cfg(test)]
mod tests;
