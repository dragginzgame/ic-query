use std::time::Duration;

use ic_topology::node::{NodeError, NodeEvidence, NodeReport};
use ic_topology::subnet::configuration::{SubnetConfigurationEvidence, SubnetConfigurationReport};
use ic_topology::subnet::nodes::{NodeRead, NodeReadOutcome, SubnetNodesReport};

use super::*;

fn inventory(outcome: NodeReadOutcome) -> SubnetNodesReport {
    let node_id = Principal::from_slice(&[1]);
    let succeeded = matches!(outcome, NodeReadOutcome::Read { .. });
    let not_started = matches!(outcome, NodeReadOutcome::NotStarted);
    let cancelled = matches!(outcome, NodeReadOutcome::OverallDeadline);
    SubnetNodesReport {
        schema: 1,
        atomic_snapshot: false,
        started_at_unix_seconds: 100,
        elapsed: Duration::from_secs(60),
        all_node_reads_succeeded: succeeded,
        overall_deadline_reached: not_started || cancelled,
        configuration: SubnetConfigurationReport {
            schema: 1,
            subnet_id: Principal::from_slice(&[2]),
            registered_node_ids: vec![node_id],
            replica_version_id: "replica\u{1b}[31m".into(),
            registered_subnet_type: 2,
            is_halted: false,
            halt_at_cup_height: false,
            evidence: SubnetConfigurationEvidence {
                endpoint: "fixture",
                source_canister: "registry",
                source_method: "get_value",
                source_key: "subnet_record_fixture".into(),
                observed_at_unix_seconds: 101,
                record_version: 3,
                record_timestamp_nanoseconds: 4,
                query_signature_verified: true,
                result_certified: false,
                membership_complete: true,
            },
        },
        nodes: vec![NodeRead {
            node_id,
            started_after: (!not_started).then_some(Duration::from_secs(1)),
            finished_after: (!not_started && !cancelled).then_some(Duration::from_secs(2)),
            outcome,
        }],
    }
}

#[test]
fn partial_inventory_retains_json_and_exits_unsuccessfully_without_terminal_injection() {
    for (outcome, status, message) in [
        (
            NodeReadOutcome::Failed {
                error: NodeError::RegistryRejected {
                    code: 3,
                    reason: "reason\u{1b}[31m".into(),
                    key: vec![0, 255],
                },
            },
            "failed",
            "failed",
        ),
        (
            NodeReadOutcome::OverallDeadline,
            "overall_deadline",
            "cancelled",
        ),
        (NodeReadOutcome::NotStarted, "not_started", "not started"),
        (
            NodeReadOutcome::IdentityMismatch {
                observed_node_id: Principal::from_slice(&[9]),
            },
            "identity_mismatch",
            "ID mismatch",
        ),
    ] {
        let report = inventory(outcome);
        let output = nodes_output(&report, true).unwrap();
        assert_eq!(output.exit_code, ExitCode::FAILURE);
        let json: serde_json::Value = serde_json::from_str(&output.text).unwrap();
        assert_eq!(json["all_node_reads_succeeded"], false);
        assert_eq!(json["nodes"][0]["outcome"]["status"], status);
        if status == "failed" {
            assert_eq!(
                json["nodes"][0]["outcome"]["error"]["reason"],
                "reason\u{1b}[31m"
            );
        }
        let output = nodes_output(&report, false).unwrap();
        assert_eq!(output.exit_code, ExitCode::FAILURE);
        assert!(output.text.contains("Nodes: 0 / 1 read | 60.00s | partial"));
        assert!(output.text.contains("Atomic snapshot: false"));
        assert!(output.text.contains(message));
        assert!(!output.text.contains('\u{1b}'));
    }
}

#[test]
fn complete_inventory_prints_node_evidence_and_exits_successfully() {
    let report = inventory(NodeReadOutcome::Read {
        report: Box::new(NodeReport {
            schema: 1,
            node_id: Principal::from_slice(&[1]),
            node_operator_id: Principal::from_slice(&[3]),
            chip_id: Some(vec![0, 255]),
            hostos_version_id: Some("host\n\u{1b}[31m".into()),
            evidence: NodeEvidence {
                endpoint: "fixture",
                source_canister: "registry",
                source_method: "get_value",
                source_key: "node_record_fixture".into(),
                observed_at_unix_seconds: 102,
                record_version: 5,
                record_timestamp_nanoseconds: u64::MAX,
                query_signature_verified: true,
                result_certified: false,
                tee_attestation_verified: false,
            },
        }),
    });
    let output = nodes_output(&report, true).unwrap();
    assert_eq!(output.exit_code, ExitCode::SUCCESS);
    let json: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(json["all_node_reads_succeeded"], true);
    assert_eq!(
        json["nodes"][0]["outcome"]["report"]["chip_id"],
        serde_json::json!([0, 255])
    );
    let output = nodes_output(&report, false).unwrap();
    assert_eq!(output.exit_code, ExitCode::SUCCESS);
    assert!(
        output
            .text
            .contains("Nodes: 1 / 1 read | 60.00s | complete")
    );
    assert!(output.text.contains("TEE attestation are not established"));
    assert!(output.text.contains("present"));
    assert!(output.text.contains("host\\n\\u{1b}[31m"));
    assert!(!output.text.contains('\u{1b}'));
    assert!(!output.text.contains("Some("));
    assert!(!output.text.contains("0x00ff"));
    assert_eq!(output.text.matches("Source:").count(), 1);
    assert!(output.text.contains(&report.nodes[0].node_id.to_string()));
    assert!(output.text.contains("Registered operators\n  O1"));
    assert!(output.text.contains("Configured HostOS versions\n  H1"));
}
