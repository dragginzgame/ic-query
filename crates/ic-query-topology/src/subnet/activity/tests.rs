use std::future::ready;
use std::time::Duration;

use ic_platform_types::Principal;
use tokio::time::Instant;

use super::{MessageChange, SubnetActivityError, sample};
use crate::ReadError;
use crate::subnet::metrics::{SubnetMetricsError, SubnetMetricsEvidence, SubnetMetricsReport};

fn observation(total: u64, time: u64) -> SubnetMetricsReport {
    SubnetMetricsReport {
        schema: 1,
        subnet_id: Principal::from_slice(&[1]),
        num_canisters: 1,
        canister_state_bytes: 2,
        consumed_cycles_total: u128::MAX,
        update_transactions_total: total,
        evidence: SubnetMetricsEvidence {
            endpoint: "fixture",
            source_path: "/read_state".into(),
            state_tree_path: "/subnet/fixture/metrics".into(),
            observed_at_unix_seconds: time,
            state_certificate_verified: true,
        },
    }
}

#[tokio::test(start_paused = true)]
async fn invalid_intervals_fail_without_reading_or_waiting() {
    for interval in [0, 61, u64::MAX] {
        let start = Instant::now();
        let mut calls = 0;
        let result = sample(interval, || {
            calls += 1;
            ready(Ok(observation(0, 0)))
        })
        .await;
        assert!(matches!(result, Err(SubnetActivityError::InvalidInterval)));
        assert_eq!(calls, 0);
        assert_eq!(Instant::now(), start);
    }
}

#[tokio::test(start_paused = true)]
async fn exactly_two_reads_wait_after_the_first_completes_at_both_interval_bounds() {
    for interval in [1, 60] {
        let mut starts = Vec::new();
        let start = Instant::now();
        let report = sample(interval, || {
            starts.push(Instant::now());
            async {
                tokio::time::sleep(Duration::from_secs(3)).await;
                Ok(observation(1, 0))
            }
        })
        .await
        .unwrap();
        assert_eq!(starts, [start, start + Duration::from_secs(3 + interval)]);
        assert_eq!(start.elapsed(), Duration::from_secs(6 + interval));
        assert_eq!(report.interval_seconds, interval);
    }
}

#[tokio::test(start_paused = true)]
async fn comparison_preserves_extremes_raw_evidence_and_wall_clock_reversal() {
    for (first, second, expected) in [
        (0, u64::MAX, MessageChange::Increased { by: u64::MAX }),
        (7, 7, MessageChange::Unchanged),
        (u64::MAX, 0, MessageChange::Decreased),
    ] {
        let mut observations = [observation(first, 100), observation(second, 99)].into_iter();
        let report = sample(1, || ready(Ok(observations.next().unwrap())))
            .await
            .unwrap();
        assert_eq!(report.message_change, expected);
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(json["schema"], 1);
        assert_eq!(json["before"]["update_transactions_total"], first);
        assert_eq!(json["after"]["update_transactions_total"], second);
        assert_eq!(json["before"]["evidence"]["observed_at_unix_seconds"], 100);
        assert_eq!(json["after"]["evidence"]["observed_at_unix_seconds"], 99);
        assert_eq!(
            json["after"]["consumed_cycles_total"],
            u128::MAX.to_string()
        );
        assert_eq!(
            json["message_change"],
            serde_json::to_value(expected).unwrap()
        );
        assert!(observations.next().is_none());
    }
}

#[tokio::test(start_paused = true)]
async fn first_failure_keeps_typed_cause_and_stops_without_wait_or_retry() {
    let start = Instant::now();
    let mut calls = 0;
    let result = sample(10, || {
        calls += 1;
        ready(Err(SubnetMetricsError::Read(ReadError::Deadline)))
    })
    .await;
    assert!(matches!(
        result,
        Err(SubnetActivityError::First(SubnetMetricsError::Read(
            ReadError::Deadline
        )))
    ));
    assert_eq!(calls, 1);
    assert_eq!(Instant::now(), start);
}

#[tokio::test(start_paused = true)]
async fn second_failure_retains_first_observation_and_never_retries() {
    let mut calls = 0;
    let result = sample(10, || {
        calls += 1;
        ready(if calls == 1 {
            Ok(observation(123, 456))
        } else {
            Err(SubnetMetricsError::Read(ReadError::Deadline))
        })
    })
    .await;
    match result {
        Err(SubnetActivityError::Second { before, source }) => {
            assert_eq!(before.update_transactions_total, 123);
            assert_eq!(before.evidence.observed_at_unix_seconds, 456);
            assert!(matches!(
                source,
                SubnetMetricsError::Read(ReadError::Deadline)
            ));
        }
        other => panic!("unexpected result: {other:?}"),
    }
    assert_eq!(calls, 2);
}

#[tokio::test(start_paused = true)]
async fn cancellation_during_wait_does_not_start_a_second_read() {
    let mut calls = 0;
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        sample(60, || {
            calls += 1;
            ready(Ok(observation(0, 0)))
        }),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(calls, 1);
    tokio::time::advance(Duration::from_secs(100)).await;
    assert_eq!(calls, 1);
}
