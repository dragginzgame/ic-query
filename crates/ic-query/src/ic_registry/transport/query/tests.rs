use super::*;
use ic_platform::agent::{AgentError, HttpErrorPayload, TransportError};
use std::task::Context;

fn http(status: u16) -> QueryCallError {
    AgentError::HttpError(HttpErrorPayload {
        status,
        content_type: None,
        content: Vec::new(),
    })
    .into()
}

#[test]
fn retry_classification_is_narrow() {
    assert!(!transient(&AgentError::TimeoutWaitingForResponse().into()));
    assert!(transient(&http(502)));
    assert!(transient(&http(504)));
    for status in [400, 401, 403, 404, 429, 500, 503] {
        assert!(!transient(&http(status)));
    }
    assert!(!transient(
        &AgentError::TransportError(TransportError::Generic("timeout".into())).into()
    ));
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
        assert!(matches!(
            error.failure(),
            QueryFailure::Http { status: 403 }
        ));
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
        assert!(matches!(
            error.failure(),
            QueryFailure::Http { status: 502 }
        ));
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
