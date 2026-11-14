use std::cell::{Cell, RefCell};
use std::future::{pending, ready};

use super::*;
use crate::node::NodeEvidence;
use crate::subnet::configuration::SubnetConfigurationEvidence;

fn id(byte: u8) -> Principal {
    Principal::from_slice(&[byte])
}

fn configuration(count: u8) -> SubnetConfigurationReport {
    SubnetConfigurationReport {
        schema: 1,
        subnet_id: id(250),
        registered_node_ids: (0..count).map(id).collect(),
        replica_version_id: "configured".into(),
        registered_subnet_type: 2,
        is_halted: false,
        halt_at_cup_height: true,
        evidence: SubnetConfigurationEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "subnet_record_fixture".into(),
            observed_at_unix_seconds: 123,
            record_version: 456,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: true,
            result_certified: false,
            membership_complete: true,
        },
    }
}

fn node(node_id: Principal) -> NodeReport {
    NodeReport {
        schema: 1,
        node_id,
        node_operator_id: id(240),
        chip_id: Some(vec![]),
        hostos_version_id: None,
        evidence: NodeEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: format!("node_record_{node_id}"),
            observed_at_unix_seconds: 124,
            record_version: 789,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: true,
            result_certified: false,
            tee_attestation_verified: false,
        },
    }
}

struct InFlight<'a>(&'a Cell<usize>);

impl Drop for InFlight<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

#[tokio::test(start_paused = true)]
async fn maximum_inventory_is_scoped_ordered_bounded_and_keeps_independent_evidence() {
    let active = Cell::new(0);
    let peak = Cell::new(0);
    let calls = RefCell::new(Vec::new());
    let report = collect(id(250), 122, ready(Ok(configuration(128))), |node_id| {
        let active = &active;
        let peak = &peak;
        let calls = &calls;
        async move {
            calls.borrow_mut().push(node_id);
            active.set(active.get() + 1);
            peak.set(peak.get().max(active.get()));
            let _guard = InFlight(active);
            // Different completion order must not reorder rows.
            tokio::time::sleep(Duration::from_millis(u64::from(
                128 - node_id.as_slice()[0],
            )))
            .await;
            Ok(node(node_id))
        }
    })
    .await
    .unwrap();
    assert_eq!(peak.get(), CONCURRENCY);
    assert_eq!(active.get(), 0);
    let mut calls = calls.into_inner();
    calls.sort_unstable();
    assert_eq!(calls, report.configuration.registered_node_ids);
    assert_eq!(
        report
            .nodes
            .iter()
            .map(|row| row.node_id)
            .collect::<Vec<_>>(),
        calls
    );
    assert!(report.all_node_reads_succeeded);
    assert!(!report.overall_deadline_reached);
    assert!(!report.atomic_snapshot);
    assert_eq!(report.started_at_unix_seconds, 122);
    assert_eq!(report.configuration.evidence.record_version, 456);
    for row in &report.nodes {
        assert!(row.started_after <= row.finished_after);
        assert!(row.finished_after.unwrap() <= report.elapsed);
        let NodeReadOutcome::Read { report } = &row.outcome else {
            panic!("expected node record")
        };
        assert_eq!(report.evidence.record_version, 789);
        assert_eq!(report.evidence.observed_at_unix_seconds, 124);
        assert!(!report.evidence.result_certified);
        assert!(!report.evidence.tee_attestation_verified);
        assert_eq!(report.chip_id, Some(vec![]));
        assert_eq!(report.hostos_version_id, None);
    }
}

#[tokio::test]
async fn configuration_refusals_do_not_start_nodes_and_excess_membership_is_retained() {
    let reader = |_| -> std::future::Ready<Result<NodeReport, NodeError>> {
        panic!("node read must not start")
    };
    let error = collect(
        id(250),
        1,
        ready(Err(SubnetConfigurationError::MissingContent)),
        reader,
    )
    .await
    .unwrap_err();
    assert!(matches!(
        error,
        SubnetNodesError::Configuration(SubnetConfigurationError::MissingContent)
    ));
    let error = collect(id(249), 1, ready(Ok(configuration(1))), reader)
        .await
        .unwrap_err();
    assert!(
        matches!(error, SubnetNodesError::IdentityMismatch { requested, observed } if requested == id(249) && observed == id(250))
    );
    let error = collect(id(250), 1, ready(Ok(configuration(129))), reader)
        .await
        .unwrap_err();
    assert!(
        matches!(error, SubnetNodesError::NodeLimit { configuration } if configuration.registered_node_ids.len() == 129)
    );
}

#[tokio::test(start_paused = true)]
async fn initial_configuration_obeys_overall_deadline() {
    let start = Instant::now();
    let error = collect(id(250), 1, pending(), |_| async { panic!("no node reads") })
        .await
        .unwrap_err();
    assert!(matches!(error, SubnetNodesError::ConfigurationDeadline));
    assert_eq!(start.elapsed(), DEADLINE);
}

#[tokio::test]
async fn empty_membership_has_no_reads_and_no_global_completeness_claim() {
    let report = collect(id(250), 1, ready(Ok(configuration(0))), |_| async {
        panic!("no node reads")
    })
    .await
    .unwrap();
    assert!(report.nodes.is_empty());
    assert!(report.all_node_reads_succeeded);
    assert!(!report.atomic_snapshot);
    assert!(!report.overall_deadline_reached);
}

#[tokio::test]
async fn partial_failures_keep_node_identity_typed_causes_and_raw_registry_fields() {
    let calls = Cell::new(0);
    let report = collect(id(250), 1, ready(Ok(configuration(6))), |node_id| {
        calls.set(calls.get() + 1);
        ready(match node_id.as_slice()[0] {
            0 => Err(NodeError::RegistryRejected {
                code: 3,
                reason: "missing\n\u{1b}[31m".into(),
                key: vec![0, 255],
            }),
            1 => Err(NodeError::MissingOperator),
            2 => Err(NodeError::Read(ReadError::Deadline)),
            3 => Ok(node(id(249))),
            _ => Ok(node(node_id)),
        })
    })
    .await
    .unwrap();
    assert_eq!(calls.get(), 6); // A failure neither aborts peers nor adds retries.
    assert!(!report.all_node_reads_succeeded);
    assert!(!report.overall_deadline_reached);
    assert!(
        matches!(&report.nodes[0].outcome, NodeReadOutcome::Failed { error: NodeError::RegistryRejected { code: 3, key, .. } } if key == &[0,255])
    );
    assert!(
        matches!(&report.nodes[3].outcome, NodeReadOutcome::IdentityMismatch { observed_node_id } if *observed_node_id == id(249))
    );
    assert!(report.nodes.iter().all(|row| row.finished_after.is_some()));
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["nodes"][0]["node_id"], id(0).to_string());
    assert_eq!(
        json["nodes"][0]["outcome"]["error"]["reason"],
        "missing\n\u{1b}[31m"
    );
    assert_eq!(
        json["nodes"][0]["outcome"]["error"]["kind"],
        "registry_rejected"
    );
    assert_eq!(
        json["nodes"][1]["outcome"]["error"]["kind"],
        "missing_operator"
    );
    assert_eq!(
        json["nodes"][2]["outcome"]["error"]["kind"],
        "read_deadline"
    );
    assert_eq!(
        json["nodes"][4]["outcome"]["report"]["chip_id"],
        serde_json::json!([])
    );
}

#[tokio::test(start_paused = true)]
async fn deadline_includes_configuration_retains_finished_rows_and_cancels_only_started_reads() {
    let active = Cell::new(0);
    let calls = Cell::new(0);
    let report = collect(
        id(250),
        1,
        async {
            tokio::time::sleep(Duration::from_secs(15)).await;
            Ok(configuration(9))
        },
        |node_id| {
            let active = &active;
            let calls = &calls;
            async move {
                calls.set(calls.get() + 1);
                active.set(active.get() + 1);
                let _guard = InFlight(active);
                if node_id == id(0) {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    Ok(node(node_id))
                } else {
                    pending().await
                }
            }
        },
    )
    .await
    .unwrap();
    assert_eq!(report.elapsed, DEADLINE);
    assert_eq!(active.get(), 0);
    assert_eq!(calls.get(), CONCURRENCY);
    assert!(report.overall_deadline_reached);
    assert!(!report.all_node_reads_succeeded);
    assert!(matches!(
        report.nodes[0].outcome,
        NodeReadOutcome::Read { .. }
    ));
    assert_eq!(
        report.nodes[0].finished_after,
        Some(Duration::from_secs(16))
    );
    for row in &report.nodes[1..4] {
        assert!(matches!(row.outcome, NodeReadOutcome::OverallDeadline));
        assert_eq!(row.started_after, Some(Duration::from_secs(15)));
        assert_eq!(row.finished_after, None);
    }
    for row in &report.nodes[4..] {
        assert!(matches!(row.outcome, NodeReadOutcome::NotStarted));
        assert_eq!(row.started_after, None);
        assert_eq!(row.finished_after, None);
    }
}

#[tokio::test(start_paused = true)]
async fn deadline_wins_over_newly_ready_work_at_the_boundary() {
    let calls = Cell::new(0);
    let report = collect(id(250), 1, ready(Ok(configuration(5))), |node_id| {
        calls.set(calls.get() + 1);
        async move {
            tokio::time::sleep(DEADLINE).await;
            Ok(node(node_id))
        }
    })
    .await
    .unwrap();
    assert_eq!(calls.get(), CONCURRENCY);
    assert!(report.overall_deadline_reached);
    assert!(matches!(
        report.nodes[4].outcome,
        NodeReadOutcome::NotStarted
    ));
}

#[tokio::test(start_paused = true)]
async fn caller_cancellation_leaves_no_detached_reads() {
    let active = Cell::new(0);
    let calls = Cell::new(0);
    let work = collect(id(250), 1, ready(Ok(configuration(10))), |_| async {
        calls.set(calls.get() + 1);
        active.set(active.get() + 1);
        let _guard = InFlight(&active);
        pending().await
    });
    assert!(
        tokio::time::timeout(Duration::from_secs(1), work)
            .await
            .is_err()
    );
    assert_eq!(calls.get(), CONCURRENCY);
    assert_eq!(active.get(), 0);
}
