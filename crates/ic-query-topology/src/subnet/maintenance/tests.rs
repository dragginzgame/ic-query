use super::{
    super::{configuration::SubnetConfigurationEvidence, metrics::SubnetMetricsEvidence},
    *,
};
use std::{
    cell::{Cell, RefCell},
    future::ready,
};

fn id() -> Principal {
    Principal::from_slice(&[1])
}

fn config(v: u64) -> SubnetConfigurationReport {
    SubnetConfigurationReport {
        schema: 1,
        subnet_id: id(),
        registered_node_ids: vec![],
        replica_version_id: "".into(),
        registered_subnet_type: 2,
        is_halted: false,
        halt_at_cup_height: false,
        evidence: SubnetConfigurationEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "key".into(),
            observed_at_unix_seconds: 100 - v,
            record_version: v,
            record_timestamp_nanoseconds: 0,
            query_signature_verified: true,
            result_certified: false,
            membership_complete: true,
        },
    }
}

fn metric(n: u64) -> SubnetMetricsReport {
    SubnetMetricsReport {
        schema: 1,
        subnet_id: id(),
        num_canisters: 0,
        canister_state_bytes: 0,
        consumed_cycles_total: u128::MAX,
        update_transactions_total: n,
        evidence: SubnetMetricsEvidence {
            endpoint: "fixture",
            source_path: "path".into(),
            state_tree_path: "metrics".into(),
            observed_at_unix_seconds: 0,
            state_certificate_verified: true,
        },
    }
}

#[tokio::test(start_paused = true)]
async fn ordering_wait_and_exact_raw_values() {
    let events = RefCell::new(vec![]);
    let n = Cell::new(0);
    let start = Instant::now();
    let report = collect(
        id(),
        10,
        Some("".into()),
        || {
            events.borrow_mut().push(("c", start.elapsed().as_secs()));
            n.set(n.get() + 1);
            let v = n.get();
            async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(config(v))
            }
        },
        || {
            events.borrow_mut().push(("m", start.elapsed().as_secs()));
            let v = n.get();
            async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
                Ok(metric(v))
            }
        },
    )
    .await
    .unwrap();
    assert_eq!(*events.borrow(), [("c", 0), ("m", 2), ("c", 15), ("m", 17)]);
    assert_eq!(report.elapsed, Duration::from_secs(20));
    assert!(report.comparisons_available());
    assert_eq!(report.expected_before, VersionMatch::Matches);
    assert_eq!(
        report.message_change,
        Some(MessageChange::Increased { by: 1 })
    );
    let json = serde_json::to_value(report).unwrap();
    assert_eq!(
        json["metrics_before"]["report"]["consumed_cycles_total"],
        u128::MAX.to_string()
    );
    assert_eq!(json["atomic_snapshot"], false);
}

#[tokio::test(start_paused = true)]
async fn invalid_inputs_and_anchor_failure_do_no_later_work() {
    for (interval, expected) in [(0, None), (61, None), (1, Some("é".repeat(129)))] {
        let result = collect(
            id(),
            interval,
            expected,
            || async { panic!("IO") },
            || async { panic!("IO") },
        )
        .await;
        assert!(result.is_err());
    }
    let result = collect(
        id(),
        1,
        None,
        || ready(Err(SubnetConfigurationError::MissingContent)),
        || async { panic!("later IO") },
    )
    .await;
    assert!(matches!(result, Err(MaintenanceError::Initial(_))));
}

#[tokio::test(start_paused = true)]
async fn every_later_failure_retains_independent_evidence() {
    for failed in 1..4 {
        let call = Cell::new(0);
        let report = collect(
            id(),
            1,
            None,
            || {
                let i = call.get();
                call.set(i + 1);
                ready(if i == failed {
                    Err(SubnetConfigurationError::Read(ReadError::Deadline))
                } else {
                    Ok(config(i))
                })
            },
            || {
                let i = call.get();
                call.set(i + 1);
                ready(if i == failed {
                    Err(SubnetMetricsError::Read(ReadError::Deadline))
                } else {
                    Ok(metric(i))
                })
            },
        )
        .await
        .unwrap();
        assert_eq!(call.get(), 4);
        assert!(!report.collection_complete);
        assert!(!report.comparisons_available());
        assert_eq!(report.metrics_after.report().is_some(), failed != 3);
        assert_eq!(report.message_change.is_some(), failed == 2);
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("read_deadline"));
        if failed == 2 {
            assert_eq!(report.expected_after, VersionMatch::Unavailable);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn refusal_does_not_discard_activity_or_inputs() {
    let n = Cell::new(3);
    let report = collect(
        id(),
        1,
        Some("v".into()),
        || {
            n.set(n.get() - 1);
            ready(Ok(config(n.get())))
        },
        || ready(Ok(metric(u64::MAX))),
    )
    .await
    .unwrap();
    assert!(report.collection_complete);
    assert!(!report.comparisons_available());
    assert!(matches!(
        report.configuration,
        ConfigurationOutcome::Refused {
            reason: ComparisonRefusal::VersionRegression,
            ..
        }
    ));
    assert_eq!(report.message_change, Some(MessageChange::Unchanged));
    assert_eq!(report.expected_before, VersionMatch::Differs);
    for expected in [None, Some(""), Some(" ")] {
        assert_eq!(
            assess(Some(&config(1)), expected),
            match expected {
                None => VersionMatch::NotRequested,
                Some("") => VersionMatch::Matches,
                _ => VersionMatch::Differs,
            }
        );
    }
    assert_eq!(
        activity::message_change(u64::MAX, 0),
        MessageChange::Decreased
    );
}

#[tokio::test(start_paused = true)]
async fn cancellation_at_each_read_and_wait_starts_no_later_work() {
    for blocked in 0..4 {
        let count = Cell::new(0);
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            collect(
                id(),
                1,
                None,
                || {
                    let i = count.get();
                    count.set(i + 1);
                    async move {
                        if i == blocked {
                            std::future::pending::<()>().await;
                        }
                        Ok(config(i))
                    }
                },
                || {
                    let i = count.get();
                    count.set(i + 1);
                    async move {
                        if i == blocked {
                            std::future::pending::<()>().await;
                        }
                        Ok(metric(i))
                    }
                },
            ),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(count.get(), blocked + 1);
    }
    let count = Cell::new(0);
    let result = tokio::time::timeout(
        Duration::from_secs(1),
        collect(
            id(),
            60,
            None,
            || {
                count.set(count.get() + 1);
                ready(Ok(config(1)))
            },
            || {
                count.set(count.get() + 1);
                ready(Ok(metric(0)))
            },
        ),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(count.get(), 2);
}
