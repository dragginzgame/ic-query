use std::future;

use ic_platform::agent::AgentError;

use super::{ReadError, read};

#[tokio::test(start_paused = true)]
async fn stalled_read_reaches_deadline_without_waiting_on_wall_time() {
    let result = read(future::pending::<Result<(), AgentError>>()).await;
    assert!(matches!(result, Err(ReadError::Deadline)));
}

#[tokio::test]
async fn transport_failure_preserves_the_supplier_error() {
    let result = read(future::ready(Err::<(), _>(AgentError::MessageError(
        "fixture".into(),
    ))))
    .await;
    match result {
        Err(ReadError::Transport(error)) => {
            assert!(matches!(*error, AgentError::MessageError(message) if message == "fixture"));
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[tokio::test]
async fn successful_read_preserves_the_observation() {
    assert_eq!(
        read(future::ready(Ok(vec![1, 2, 3]))).await.unwrap(),
        [1, 2, 3]
    );
}
