use super::{RegistryQueryCounter, SubnetCatalogProgressPhase};
use ic_agent::{Agent, AgentError, agent_error::TransportError};
use std::{future::Future, time::Duration};

pub(super) async fn query(
    agent: &Agent,
    canister: &candid::Principal,
    method: &'static str,
    arg: Vec<u8>,
    counter: Option<&RegistryQueryCounter>,
) -> Result<Vec<u8>, AgentError> {
    retry_read(method, counter, || {
        agent.query(canister, method).with_arg(arg.clone()).call()
    })
    .await
}

async fn retry_read<F, Fut>(
    method: &'static str,
    counter: Option<&RegistryQueryCounter>,
    mut read: F,
) -> Result<Vec<u8>, AgentError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Vec<u8>, AgentError>>,
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

fn transient(error: &AgentError) -> bool {
    match error {
        AgentError::TransportError(TransportError::Reqwest(error)) => {
            error.is_connect() || error.is_timeout()
        }
        AgentError::HttpError(payload) => matches!(payload.status, 502 | 504),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_agent::agent_error::HttpErrorPayload;
    use std::task::Context;

    fn http(status: u16) -> AgentError {
        AgentError::HttpError(HttpErrorPayload {
            status,
            content_type: None,
            content: Vec::new(),
        })
    }

    #[test]
    fn retry_classification_is_narrow() {
        assert!(transient(&http(502)));
        assert!(transient(&http(504)));
        for status in [400, 401, 403, 404, 429, 500, 503] {
            assert!(!transient(&http(status)));
        }
        assert!(!transient(&AgentError::TransportError(
            TransportError::Generic("timeout".into())
        )));
    }

    #[test]
    fn cancellation_drops_retry_backoff_and_permanent_errors_do_not_retry() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let counter = RegistryQueryCounter::with_acquisition(
                "https://example.com".into(),
                std::sync::Arc::default(),
            );
            let mut future = Box::pin(retry_read("get_value", Some(&counter), || {
                std::future::ready(Err(http(502)))
            }));
            let waker = futures::task::noop_waker();
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(&waker))
                    .is_pending()
            );
            assert_eq!(counter.call_count(), 1);
            drop(future);
            let error = retry_read("get_value", Some(&counter), || {
                std::future::ready(Err(http(403)))
            })
            .await
            .unwrap_err();
            assert!(matches!(error, AgentError::HttpError(payload) if payload.status == 403));
            assert_eq!(counter.call_count(), 2);
        });
    }

    #[test]
    fn retries_count_attempts_and_preserve_final_error() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let counter = RegistryQueryCounter::with_acquisition(
                "https://example.com".into(),
                std::sync::Arc::default(),
            );
            let error = retry_read("get_changes_since", Some(&counter), || {
                std::future::ready(Err(http(502)))
            })
            .await
            .unwrap_err();
            assert!(matches!(error, AgentError::HttpError(payload) if payload.status == 502));
            assert_eq!(counter.call_count(), 3);
            let mut calls = 0;
            let result = retry_read("get_value", Some(&counter), || {
                calls += 1;
                std::future::ready(if calls == 1 {
                    Err(http(504))
                } else {
                    Ok(vec![42])
                })
            })
            .await
            .unwrap();
            assert_eq!(result, vec![42]);
            assert_eq!(counter.call_count(), 5);
        });
    }
}
