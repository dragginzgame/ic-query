use super::*;
use crate::subnet::{
    configuration::{SubnetConfigurationEvidence, SubnetConfigurationReport},
    distribution::DistributionSummary,
    metrics::SubnetMetricsEvidence,
};
use std::{
    cell::{Cell, RefCell},
    future::ready,
};
fn id(n: u8) -> Principal {
    Principal::from_slice(&[n])
}
fn distribution(id: Principal, complete: bool) -> SubnetDistributionReport {
    SubnetDistributionReport {
        schema: 1,
        endpoint: "https://ic0.app",
        source_canister: "rwlgt-iiaaa-aaaaa-aaaaa-cai",
        version_method: "get_latest_version",
        requested_registry_version: u64::from(id.as_slice()[0]),
        version_observed_at_unix_seconds: 100,
        started_at_unix_seconds: 100,
        elapsed: Duration::ZERO,
        query_signature_verified: true,
        result_certified: false,
        overall_deadline_reached: false,
        complete,
        configuration: SubnetConfigurationReport {
            schema: 1,
            subnet_id: id,
            registered_node_ids: vec![],
            replica_version_id: String::new(),
            registered_subnet_type: -1,
            is_halted: false,
            halt_at_cup_height: false,
            evidence: SubnetConfigurationEvidence {
                endpoint: "https://ic0.app",
                source_canister: "rwlgt-iiaaa-aaaaa-aaaaa-cai",
                source_method: "get_value",
                source_key: format!("subnet_record_{id}"),
                observed_at_unix_seconds: 100,
                record_version: 1,
                record_timestamp_nanoseconds: 0,
                query_signature_verified: true,
                result_certified: false,
                membership_complete: true,
            },
        },
        nodes: vec![],
        operators: vec![],
        data_centers: vec![],
        relationships: vec![],
        summary: DistributionSummary {
            selected_nodes: 0,
            provider_resolved: 0,
            provider_unknown: 0,
            data_center_references_resolved: 0,
            location_resolved: 0,
            location_unknown: 0,
            providers: vec![],
            data_centers: vec![],
            regions: vec![],
            region_unknown: 0,
            owners: vec![],
            owner_unknown: 0,
        },
    }
}
fn metric(id: Principal) -> SubnetMetricsReport {
    SubnetMetricsReport {
        schema: 1,
        subnet_id: id,
        num_canisters: 0,
        canister_state_bytes: u64::MAX,
        consumed_cycles_total: u128::MAX,
        update_transactions_total: u64::MAX,
        evidence: SubnetMetricsEvidence {
            endpoint: "https://ic0.app",
            source_path: format!("/api/v3/subnet/{id}/read_state"),
            state_tree_path: format!("/subnet/{id}/metrics"),
            observed_at_unix_seconds: 99,
            state_certificate_verified: true,
        },
    }
}
#[tokio::test]
async fn selection_order_versions_and_raw_values() {
    let events = RefCell::new(vec![]);
    let report = collect(
        id(1),
        id(2),
        |id| {
            events.borrow_mut().push(("distribution", id));
            ready(Ok(distribution(id, true)))
        },
        |id| {
            events.borrow_mut().push(("metrics", id));
            ready(Ok(metric(id)))
        },
    )
    .await
    .unwrap();
    assert_eq!(
        *events.borrow(),
        vec![
            ("distribution", id(1)),
            ("metrics", id(1)),
            ("distribution", id(2)),
            ("metrics", id(2))
        ]
    );
    assert!(report.collection_complete && !report.atomic_snapshot);
    assert_eq!(report.registry_versions_agree, Some(false));
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(
        json["left"]["metrics"]["report"]["consumed_cycles_total"],
        u128::MAX.to_string()
    );
    assert_eq!(
        json["right"]["metrics"]["report"]["canister_state_bytes"],
        u64::MAX
    );
    let duplicate = collect(
        id(1),
        id(1),
        |_| {
            events.borrow_mut().push(("unexpected", id(1)));
            ready(Ok(distribution(id(1), true)))
        },
        |_| ready(Ok(metric(id(1)))),
    )
    .await;
    assert!(matches!(duplicate, Err(ComparisonError::SameSubnet)));
    assert_eq!(events.borrow().len(), 4);
}
#[tokio::test]
async fn every_failure_preserves_other_inputs_and_all_failures_are_explicit() {
    for mask in 1..16 {
        let calls = Cell::new(0);
        let report = collect(
            id(1),
            id(2),
            |id| {
                let n = calls.get();
                calls.set(n + 1);
                ready(if mask & (1 << n) != 0 {
                    Err(DistributionError::InitialDeadline)
                } else {
                    Ok(distribution(id, true))
                })
            },
            |id| {
                let n = calls.get();
                calls.set(n + 1);
                ready(if mask & (1 << n) != 0 {
                    Err(SubnetMetricsError::Read(ReadError::Deadline))
                } else {
                    Ok(metric(id))
                })
            },
        )
        .await
        .unwrap();
        assert_eq!(calls.get(), 4);
        assert!(!report.collection_complete);
        assert_eq!(report.left.distribution.report().is_some(), mask & 1 == 0);
        assert_eq!(report.left.metrics.report().is_some(), mask & 2 == 0);
        assert_eq!(report.right.distribution.report().is_some(), mask & 4 == 0);
        assert_eq!(report.right.metrics.report().is_some(), mask & 8 == 0);
        assert!(serde_json::to_string(&report).is_ok());
    }
    let report = collect(
        id(1),
        id(2),
        |id| ready(Ok(distribution(id, false))),
        |id| ready(Ok(metric(id))),
    )
    .await
    .unwrap();
    assert!(!report.collection_complete);
    assert!(report.left.distribution.report().is_some());
}
#[tokio::test(start_paused = true)]
async fn cancellation_at_every_acquisition_starts_no_later_work() {
    for stop in 1..=4 {
        let calls = Cell::new(0);
        let work = collect(
            id(1),
            id(2),
            |id| {
                calls.set(calls.get() + 1);
                async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    Ok(distribution(id, true))
                }
            },
            |id| {
                calls.set(calls.get() + 1);
                async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    Ok(metric(id))
                }
            },
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(stop * 10 - 1), work)
                .await
                .is_err()
        );
        assert_eq!(calls.get(), stop);
        tokio::time::sleep(Duration::from_secs(100)).await;
        assert_eq!(calls.get(), stop);
    }
}
