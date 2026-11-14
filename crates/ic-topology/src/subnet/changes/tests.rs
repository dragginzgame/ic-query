use std::cell::Cell;
use std::future::ready;

use super::*;
use crate::subnet::configuration::SubnetConfigurationEvidence;

fn id(byte: u8) -> Principal {
    Principal::from_slice(&[byte])
}

fn configuration(version: u64, nodes: &[u8]) -> SubnetConfigurationReport {
    SubnetConfigurationReport {
        schema: 1,
        subnet_id: id(250),
        registered_node_ids: nodes.iter().copied().map(id).collect(),
        replica_version_id: "configured".into(),
        registered_subnet_type: 2,
        is_halted: false,
        halt_at_cup_height: false,
        evidence: SubnetConfigurationEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "subnet_record_fixture".into(),
            observed_at_unix_seconds: 123,
            record_version: version,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: true,
            result_certified: false,
            membership_complete: true,
        },
    }
}

#[test]
fn changes_keep_raw_fields_and_both_sources_without_promoting_assurance() {
    let before = configuration(8, &[1, 2]);
    let mut after = configuration(9, &[2, 3]);
    after.replica_version_id = "next\n\u{1b}[31m".into();
    after.registered_subnet_type = -1;
    after.is_halted = true;
    after.halt_at_cup_height = true;
    after.evidence.endpoint = "second-fixture";
    after.evidence.observed_at_unix_seconds = 100; // Wall clocks are not event ordering.
    after.evidence.query_signature_verified = false;
    let report = compare(before, after).unwrap();
    assert!(report.has_selected_changes());
    assert_eq!(report.registered_nodes_added, [id(3)]);
    assert_eq!(report.registered_nodes_removed, [id(1)]);
    assert!(report.replica_version_changed && report.subnet_type_changed);
    assert!(report.is_halted_changed && report.halt_at_cup_height_changed);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema"], 1);
    assert!(json["requested_interval_seconds"].is_null());
    assert_eq!(json["after"]["replica_version_id"], "next\n\u{1b}[31m");
    assert_eq!(json["after"]["registered_subnet_type"], -1);
    assert_eq!(
        json["before"]["evidence"]["record_timestamp_nanoseconds"],
        u64::MAX
    );
    assert_eq!(json["after"]["evidence"]["observed_at_unix_seconds"], 100);
    assert_eq!(json["after"]["evidence"]["query_signature_verified"], false);
    assert_eq!(json["before"]["evidence"]["result_certified"], false);
    assert_eq!(json["after"]["evidence"]["endpoint"], "second-fixture");
}

#[test]
fn equal_selected_fields_allow_equal_or_newer_versions_and_empty_membership() {
    for version in [8, 9] {
        let report = compare(configuration(8, &[]), configuration(version, &[])).unwrap();
        assert!(!report.has_selected_changes());
        assert_eq!(report.after.evidence.record_version, version);
    }
    let report = compare(configuration(8, &[]), configuration(9, &[1])).unwrap();
    assert_eq!(report.registered_nodes_added, [id(1)]);
    let report = compare(configuration(8, &[1]), configuration(9, &[])).unwrap();
    assert_eq!(report.registered_nodes_removed, [id(1)]);
}

#[test]
fn refusals_retain_both_inputs_and_distinguish_regression_from_conflict() {
    for (mut after, reason) in [
        (configuration(7, &[1]), ComparisonRefusal::VersionRegression),
        (
            configuration(8, &[2]),
            ComparisonRefusal::ConflictingVersion,
        ),
        (configuration(9, &[1]), ComparisonRefusal::DifferentSubnets),
        (configuration(9, &[1]), ComparisonRefusal::Schema),
    ] {
        if reason == ComparisonRefusal::DifferentSubnets {
            after.subnet_id = id(249);
        }
        if reason == ComparisonRefusal::Schema {
            after.schema = 2;
        }
        let error = compare(configuration(8, &[1]), after).unwrap_err();
        let SubnetChangesError::Comparison {
            before,
            after,
            reason: found,
        } = error
        else {
            panic!("comparison refusal")
        };
        assert_eq!(found, reason);
        assert_eq!(before.evidence.record_version, 8);
        assert!(!after.evidence.result_certified);
    }
}

#[test]
fn public_comparison_rejects_noncanonical_or_incomplete_membership_on_either_side() {
    for bad_before in [true, false] {
        for variant in 0..4 {
            let mut before = configuration(8, &[1]);
            let mut after = configuration(9, &[1]);
            let bad = if bad_before { &mut before } else { &mut after };
            match variant {
                0 => bad.registered_node_ids = vec![id(1), id(1)],
                1 => bad.registered_node_ids = vec![id(2), id(1)],
                2 => {
                    bad.registered_node_ids = (0_u16..1025)
                        .map(|value| Principal::from_slice(&value.to_be_bytes()))
                        .collect()
                }
                _ => bad.evidence.membership_complete = false,
            }
            assert!(matches!(
                compare(before, after),
                Err(SubnetChangesError::Comparison {
                    reason: ComparisonRefusal::InvalidMembership,
                    ..
                })
            ));
        }
    }
    let mut before = configuration(8, &[]);
    let mut after = configuration(9, &[]);
    before.registered_node_ids = (0_u16..1024)
        .map(|value| Principal::from_slice(&value.to_be_bytes()))
        .collect();
    after.registered_node_ids = before.registered_node_ids.clone();
    assert!(!compare(before, after).unwrap().has_selected_changes());
}

#[tokio::test(start_paused = true)]
async fn sampling_waits_after_first_completion_and_calls_exactly_twice() {
    for interval in [1, 60] {
        let calls = Cell::new(0);
        let start = tokio::time::Instant::now();
        let report = sample(interval, || {
            let index = calls.get();
            calls.set(index + 1);
            if index == 1 {
                assert_eq!(start.elapsed(), Duration::from_secs(interval + 2));
            }
            async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok(configuration(8 + index, &[1]))
            }
        })
        .await
        .unwrap();
        assert_eq!(calls.get(), 2);
        assert_eq!(start.elapsed(), Duration::from_secs(interval + 4));
        assert_eq!(report.requested_interval_seconds, Some(interval));
        assert!(!report.has_selected_changes());
    }
}

#[tokio::test]
async fn invalid_intervals_and_initial_failures_do_not_start_followup_reads() {
    for interval in [0, 61, u64::MAX] {
        assert!(matches!(
            sample(interval, || async { panic!("no read") }).await,
            Err(SubnetChangesError::InvalidInterval)
        ));
    }
    let calls = Cell::new(0);
    let error = sample(1, || {
        calls.set(calls.get() + 1);
        ready(Err(SubnetConfigurationError::MissingContent))
    })
    .await
    .unwrap_err();
    assert_eq!(calls.get(), 1);
    assert!(matches!(
        error,
        SubnetChangesError::First(SubnetConfigurationError::MissingContent)
    ));
}

#[tokio::test(start_paused = true)]
async fn second_failure_keeps_first_observation_and_cancellation_stops_sampling() {
    let calls = Cell::new(0);
    let error = sample(1, || {
        let index = calls.get();
        calls.set(index + 1);
        ready(if index == 0 {
            Ok(configuration(8, &[1]))
        } else {
            Err(SubnetConfigurationError::MissingContent)
        })
    })
    .await
    .unwrap_err();
    assert_eq!(calls.get(), 2);
    assert!(
        matches!(error, SubnetChangesError::Second { before, source: SubnetConfigurationError::MissingContent } if before.evidence.record_version == 8)
    );
    calls.set(0);
    let work = sample(60, || {
        calls.set(calls.get() + 1);
        ready(Ok(configuration(8, &[1])))
    });
    assert!(
        tokio::time::timeout(Duration::from_secs(1), work)
            .await
            .is_err()
    );
    assert_eq!(calls.get(), 1);
}
