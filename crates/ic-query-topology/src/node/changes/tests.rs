use std::cell::{Cell, RefCell};
use std::future::ready;

use super::*;
use crate::node::NodeEvidence;

fn node(version: u64) -> NodeReport {
    let node_id = Principal::from_slice(&[1]);
    NodeReport {
        schema: 1,
        node_id,
        node_operator_id: Principal::from_slice(&[2]),
        chip_id: Some(vec![0, 255]),
        hostos_version_id: Some("configured".into()),
        evidence: NodeEvidence {
            endpoint: mainnet::ENDPOINT,
            source_canister: mainnet::REGISTRY,
            source_method: mainnet::REGISTRY_METHOD,
            source_key: crate::node::source_key(node_id),
            observed_at_unix_seconds: 100,
            record_version: version,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: true,
            result_certified: false,
            tee_attestation_verified: false,
        },
    }
}

fn assert_refused(before: NodeReport, after: NodeReport, expected: &ComparisonRefusal) {
    let before_json = serde_json::to_value(&before).unwrap();
    let after_json = serde_json::to_value(&after).unwrap();
    let NodeChangesError::Comparison {
        before,
        after,
        reason,
    } = compare(before, after).unwrap_err()
    else {
        panic!("expected comparison refusal");
    };
    assert_eq!(&reason, expected);
    assert_eq!(serde_json::to_value(before).unwrap(), before_json);
    assert_eq!(serde_json::to_value(after).unwrap(), after_json);
}

#[test]
fn optional_presence_transitions_and_raw_changes_are_not_collapsed() {
    for first in [None, Some(vec![]), Some(vec![0, 255]), Some(vec![255, 0])] {
        for second in [None, Some(vec![]), Some(vec![0, 255]), Some(vec![255, 0])] {
            let mut before = node(8);
            let mut after = node(9);
            before.chip_id = first.clone();
            after.chip_id = second.clone();
            let report = compare(before, after).unwrap();
            assert_eq!(report.chip_id_changed, first != second);
            assert_eq!(report.has_selected_changes(), first != second);
            assert!(!report.operator_changed && !report.hostos_version_changed);
            let json = serde_json::to_value(report).unwrap();
            assert_eq!(
                json["before"]["chip_id"],
                serde_json::to_value(&first).unwrap()
            );
            assert_eq!(
                json["after"]["chip_id"],
                serde_json::to_value(&second).unwrap()
            );
        }
    }
    for first in [None, Some(""), Some("configured"), Some("next\n\u{1b}[31m")] {
        for second in [None, Some(""), Some("configured"), Some("next\n\u{1b}[31m")] {
            let mut before = node(8);
            let mut after = node(9);
            before.hostos_version_id = first.map(str::to_owned);
            after.hostos_version_id = second.map(str::to_owned);
            let report = compare(before, after).unwrap();
            assert_eq!(report.hostos_version_changed, first != second);
            assert_eq!(report.has_selected_changes(), first != second);
        }
    }
}

#[test]
fn unchanged_fields_accept_newer_versions_and_keep_raw_evidence() {
    for version in [8, u64::MAX] {
        let mut after = node(version);
        after.evidence.endpoint = "second-observation";
        after.evidence.observed_at_unix_seconds = 1; // Clock changes are not Registry ordering.
        after.evidence.query_signature_verified = false;
        let report = compare(node(8), after).unwrap();
        assert!(!report.has_selected_changes());
        let json = serde_json::to_value(report).unwrap();
        assert!(json["requested_interval_seconds"].is_null());
        assert_eq!(json["after"]["evidence"]["record_version"], version);
        assert_eq!(json["after"]["evidence"]["observed_at_unix_seconds"], 1);
        assert_eq!(json["after"]["evidence"]["query_signature_verified"], false);
        assert_eq!(json["after"]["evidence"]["tee_attestation_verified"], false);
        assert_eq!(
            json["before"]["evidence"]["record_timestamp_nanoseconds"],
            u64::MAX
        );
    }
    let mut after = node(9);
    after.node_operator_id = Principal::from_slice(&[3]);
    let report = compare(node(8), after).unwrap();
    assert!(report.operator_changed && report.has_selected_changes());
}

#[test]
fn identity_schema_source_and_version_refusals_retain_both_inputs() {
    let mut after = node(9);
    after.node_id = Principal::from_slice(&[3]);
    assert_refused(node(8), after, &ComparisonRefusal::DifferentNodes);
    let mut before = node(8);
    before.schema = 2;
    assert_refused(before, node(9), &ComparisonRefusal::Schema);
    assert_refused(node(9), node(8), &ComparisonRefusal::VersionRegression);
    for field in 0..3 {
        let mut after = node(8);
        match field {
            0 => after.chip_id = None,
            1 => after.hostos_version_id = None,
            _ => after.node_operator_id = Principal::from_slice(&[3]),
        }
        assert_refused(node(8), after, &ComparisonRefusal::ConflictingVersion);
    }
    for first in [false, true] {
        for field in 0..3 {
            let mut before = node(8);
            let mut after = node(9);
            let target = if first { &mut before } else { &mut after };
            match field {
                0 => target.evidence.source_canister = "another-registry",
                1 => target.evidence.source_method = "another-method",
                _ => target.evidence.source_key = "node_record_other".into(),
            }
            assert_refused(before, after, &ComparisonRefusal::SourceContext);
        }
    }
}

#[test]
fn supplied_records_obey_existing_field_limits_and_operator_admission() {
    let mut before = node(8);
    let mut after = node(9);
    for report in [&mut before, &mut after] {
        report.chip_id = Some(vec![0; 256]);
        report.hostos_version_id = Some("é".repeat(128));
    }
    assert!(!compare(before, after).unwrap().has_selected_changes());
    for first in [false, true] {
        for field in 0..3 {
            let mut before = node(8);
            let mut after = node(9);
            let target = if first { &mut before } else { &mut after };
            match field {
                0 => target.chip_id = Some(vec![0; 257]),
                1 => target.hostos_version_id = Some(format!("{}x", "é".repeat(128))),
                _ => target.node_operator_id = Principal::from_slice(&[]),
            }
            assert_refused(before, after, &ComparisonRefusal::InvalidRecord);
        }
    }
}

#[tokio::test(start_paused = true)]
async fn sampling_waits_after_first_completion_and_accepts_interval_boundaries() {
    for interval in [1, 60] {
        let calls = Cell::new(0);
        let starts = RefCell::new(Vec::new());
        let started = tokio::time::Instant::now();
        let report = sample(interval, || {
            let call = calls.get();
            calls.set(call + 1);
            starts.borrow_mut().push(started.elapsed());
            async move {
                tokio::time::sleep(Duration::from_secs(if call == 0 { 3 } else { 5 })).await;
                Ok(node(8 + call))
            }
        })
        .await
        .unwrap();
        assert_eq!(calls.get(), 2);
        assert_eq!(
            *starts.borrow(),
            [Duration::ZERO, Duration::from_secs(3 + interval)]
        );
        assert_eq!(started.elapsed(), Duration::from_secs(8 + interval));
        assert_eq!(report.requested_interval_seconds, Some(interval));
    }
}

#[tokio::test(start_paused = true)]
async fn invalid_waits_and_initial_errors_stop_before_later_reads() {
    let calls = Cell::new(0);
    for interval in [0, 61, u64::MAX] {
        let result = sample(interval, || {
            calls.set(calls.get() + 1);
            ready(Ok(node(8)))
        })
        .await;
        assert!(matches!(result, Err(NodeChangesError::InvalidInterval)));
    }
    assert_eq!(calls.get(), 0);
    let result = sample(1, || {
        calls.set(calls.get() + 1);
        ready(Err(NodeError::MissingContent))
    })
    .await;
    assert!(matches!(
        result,
        Err(NodeChangesError::First(NodeError::MissingContent))
    ));
    assert_eq!(calls.get(), 1);
}

#[tokio::test(start_paused = true)]
async fn second_failure_preserves_first_and_cancellation_starts_no_more_reads() {
    let calls = Cell::new(0);
    let result = sample(1, || {
        let call = calls.get();
        calls.set(call + 1);
        ready(if call == 0 {
            Ok(node(8))
        } else {
            Err(NodeError::MissingOperator)
        })
    })
    .await;
    let Err(NodeChangesError::Second {
        before,
        source: NodeError::MissingOperator,
    }) = result
    else {
        panic!("expected retained observation");
    };
    assert_eq!(before.evidence.record_version, 8);
    assert_eq!(calls.get(), 2);
    for timeout in [1, 3] {
        calls.set(0);
        let result = tokio::time::timeout(
            Duration::from_secs(timeout),
            sample(2, || {
                let call = calls.get();
                calls.set(call + 1);
                async move {
                    if call == 1 {
                        tokio::time::sleep(Duration::from_secs(10)).await;
                    }
                    Ok(node(8 + call))
                }
            }),
        )
        .await;
        assert!(result.is_err());
        let expected = if timeout == 1 { 1 } else { 2 };
        assert_eq!(calls.get(), expected);
        tokio::time::sleep(Duration::from_secs(100)).await;
        assert_eq!(calls.get(), expected);
    }
}
