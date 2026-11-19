use std::future;

use ic_platform::native::ReadError as NativeError;

use super::{ReadError, read};

#[test]
fn ingress_expiry_and_time_windows_are_bounded_without_overflow() {
    let (expiry, policy) = super::context_at(std::time::Duration::from_secs(1)).unwrap();
    assert_eq!(expiry, 301_000_000_000);
    assert_eq!(policy.certificate.reference_ns, 1_000_000_000);
    assert_eq!(policy.certificate.max_age_ns, 300_000_000_000);
    assert_eq!(policy.delegation.max_future_skew_ns, 60_000_000_000);
    for now in [
        std::time::Duration::from_nanos(u64::MAX),
        std::time::Duration::MAX,
    ] {
        assert!(matches!(super::context_at(now), Err(ReadError::ClockRange)));
    }
}

#[test]
fn pinned_mainnet_trust_inputs_construct_a_client() {
    assert!(super::client().is_ok());
}

#[tokio::test]
async fn structured_rejection_preserves_code_and_text() {
    let cause = NativeError::Rejected {
        code: 5,
        message: "HTTP 504 timeout is source text".into(),
        error_code: Some("IC0536".into()),
    };
    let Err(ReadError::Transport(error)) = read(future::ready(Err::<(), _>(cause))).await else {
        panic!("lost structured supplier cause");
    };
    assert!(
        matches!(*error, NativeError::Rejected { code: 5, message, error_code }
        if message == "HTTP 504 timeout is source text" && error_code.as_deref() == Some("IC0536"))
    );
}

#[tokio::test(start_paused = true)]
async fn stalled_read_reaches_deadline_without_waiting_on_wall_time() {
    let result = read(future::pending::<Result<(), NativeError>>()).await;
    assert!(matches!(result, Err(ReadError::Deadline)));
}

#[tokio::test]
async fn transport_failure_preserves_the_supplier_error() {
    let result = read(future::ready(Err::<(), _>(NativeError::Packet))).await;

    match result {
        Err(ReadError::Transport(error)) => {
            assert!(matches!(*error, NativeError::Packet));
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
