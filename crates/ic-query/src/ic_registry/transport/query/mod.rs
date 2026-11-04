use super::{RegistryQueryCounter, SubnetCatalogProgressPhase};
use crate::agent_transport::{QueryCallError, QueryFailure};
use ic_platform::agent::Agent;
use std::{future::Future, time::Duration};

pub(super) async fn query(
    agent: &Agent,
    canister: &ic_platform::candid::contract::Principal,
    method: &'static str,
    arg: Vec<u8>,
    counter: Option<&RegistryQueryCounter>,
) -> Result<Vec<u8>, QueryCallError> {
    retry_read(method, counter, || {
        crate::agent_transport::query(agent, canister, method, arg.clone())
    })
    .await
}

async fn retry_read<F, Fut>(
    method: &'static str,
    counter: Option<&RegistryQueryCounter>,
    mut read: F,
) -> Result<Vec<u8>, QueryCallError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Vec<u8>, QueryCallError>>,
{
    for attempt in 1..=3 {
        if let Some(counter) = counter {
            counter.record_call();
        }
        let result = read().await;
        // Scope the added retry policy to catalog acquisition. ic-agent already retries
        // HTTP 429/503; do not multiply that transport's retry budget here.
        if attempt == 3
            || counter.is_none_or(|counter| counter.acquisition.is_none())
            || !result.as_ref().is_err_and(transient)
        {
            return result;
        }
        let delay_millis = 250 * u64::from(attempt);
        if let Some(counter) = counter {
            counter.emit(SubnetCatalogProgressPhase::Retry {
                method,
                next_attempt: attempt + 1,
                delay_millis,
            });
        }
        tokio::time::sleep(Duration::from_millis(delay_millis)).await;
    }
    unreachable!("last attempt always returns")
}

fn transient(error: &QueryCallError) -> bool {
    matches!(
        error.failure(),
        QueryFailure::Transport { connect: true, .. }
            | QueryFailure::Transport { timeout: true, .. }
            | QueryFailure::Http { status: 502 | 504 }
    )
}

#[cfg(test)]
mod tests;
