use ic_platform_types::Principal;
use ic_topology::node::{NodeEvidence, NodeReport};
use ic_topology::placement::{PlacementEvidence, PlacementReport};
use ic_topology::subnet::activity::{MessageChange, SubnetActivityReport};
use ic_topology::subnet::configuration::{SubnetConfigurationEvidence, SubnetConfigurationReport};
use ic_topology::subnet::list::{SubnetListEvidence, SubnetListReport};
use ic_topology::subnet::membership::SubnetMembershipReport;
use ic_topology::subnet::metrics::{SubnetMetricsEvidence, SubnetMetricsReport};
use ic_topology::subnet::{SubnetEvidence, SubnetReport};

use super::{
    node, placement, subnet, subnet_activity, subnet_configuration, subnet_list, subnet_membership,
    subnet_metrics,
};

#[test]
fn changes_text_is_compact_escapes_values_and_keeps_observation_limits() {
    let before = configuration_fixture();
    let mut after = configuration_fixture();
    after.evidence.record_version += 1;
    after.replica_version_id = "new\n\u{1b}[31m".into();
    after.registered_node_ids = vec![Principal::from_slice(&[3])];
    let report = ic_topology::subnet::changes::compare(before, after).unwrap();
    let text = super::subnet_changes(&report);
    assert!(text.contains("Selected configuration changed"));
    assert!(text.contains("300 -> 301"));
    assert!(text.contains("1 added, 0 removed"));
    assert!(text.contains("Added registered nodes"));
    assert!(text.contains("new\\n\\u{1b}[31m"));
    assert!(!text.contains('\u{1b}') && !text.contains('\u{202e}'));
    assert!(text.contains("not a complete change history"));
    let report =
        ic_topology::subnet::changes::compare(configuration_fixture(), configuration_fixture())
            .unwrap();
    let text = super::subnet_changes(&report);
    assert!(text.contains("No selected configuration changes observed"));
    assert!(!text.contains("Configured replica:"));
}

#[test]
fn node_text_preserves_presence_escapes_hostos_and_disclaims_attestation() {
    for (chip_id, hostos_version_id, expected_chip, expected_hostos) in [
        (None, None, "absent", "absent"),
        (Some(vec![]), Some(String::new()), "0x", "\"\""),
        (
            Some(vec![0, 255, 27]),
            Some("host\n\u{1b}[31m\u{202e}".into()),
            "0x00ff1b",
            "\"host\\n\\u{1b}[31m\\u{202e}\"",
        ),
    ] {
        let report = NodeReport {
            schema: 1,
            node_id: Principal::from_slice(&[1]),
            node_operator_id: Principal::from_slice(&[2]),
            chip_id,
            hostos_version_id,
            evidence: NodeEvidence {
                endpoint: "fixture",
                source_canister: "registry",
                source_method: "get_value",
                source_key: "node_record_fixture".into(),
                observed_at_unix_seconds: 123,
                record_version: 300,
                record_timestamp_nanoseconds: u64::MAX,
                query_signature_verified: false,
                result_certified: false,
                tee_attestation_verified: false,
            },
        };
        let text = node(&report);
        assert!(text.contains(&format!("present-empty): {expected_chip}\n")));
        assert!(text.contains(&format!("Configured HostOS version: {expected_hostos}\n")));
        assert!(!text.contains('\u{1b}') && !text.contains('\u{202e}'));
        assert!(text.contains("Query signature verified: false"));
        assert!(text.contains("Result consensus-certified: false"));
        assert!(text.contains("TEE attestation verified: false"));
        assert!(text.contains("Running HostOS version: not observed"));
        assert!(text.contains("Record mutation timestamp: 18446744073709551615 Unix nanoseconds"));
        assert!(text.contains("absence does not prove missing hardware capability"));
    }
}

fn configuration_fixture() -> SubnetConfigurationReport {
    SubnetConfigurationReport {
        schema: 1,
        subnet_id: Principal::from_slice(&[1]),
        registered_node_ids: vec![],
        replica_version_id: "future\n\u{1b}[31m\u{202e}".into(),
        registered_subnet_type: -1,
        is_halted: true,
        halt_at_cup_height: false,
        evidence: SubnetConfigurationEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "subnet_record_fixture".into(),
            observed_at_unix_seconds: 123,
            record_version: 300,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: false,
            result_certified: false,
            membership_complete: true,
        },
    }
}

#[test]
fn configuration_text_escapes_source_version_and_distinguishes_configured_state() {
    let mut report = configuration_fixture();
    let text = subnet_configuration(&report);
    assert!(text.contains("Configured replica version: \"future\\n\\u{1b}[31m\\u{202e}\""));
    assert!(!text.contains('\u{1b}') && !text.contains('\u{202e}'));
    assert!(text.contains("Registered subnet type (raw): -1"));
    assert!(text.contains("Configured is_halted: true"));
    assert!(text.contains("Configured halt_at_cup_height: false"));
    assert!(text.contains("Query signature verified: false"));
    assert!(text.contains("Result consensus-certified: false"));
    assert!(text.contains("Record mutation timestamp: 18446744073709551615 Unix nanoseconds"));
    assert!(text.contains("Halt flags do not establish observed downtime"));
    assert!(text.ends_with("Registered node identities (0):\n"));
    report.replica_version_id.clear();
    report.registered_subnet_type = 5;
    let text = subnet_configuration(&report);
    assert!(text.contains("Configured replica version: \"\"\n"));
    assert!(text.contains("Effective/running version: not resolved"));
}

#[test]
fn membership_text_labels_differences_without_upgrading_source_assurance() {
    let registered = Principal::from_slice(&[2]);
    let observed = Principal::from_slice(&[3]);
    let mut configuration = configuration_fixture();
    configuration.registered_node_ids = vec![registered];
    let mut report = SubnetMembershipReport {
        schema: 1,
        atomic_snapshot: false,
        configuration,
        observation: SubnetReport {
            schema: 1,
            subnet_id: Principal::from_slice(&[1]),
            observed_node_ids: vec![observed],
            observed_subnet_type: None,
            evidence: SubnetEvidence {
                endpoint: "fixture",
                source_path: "/read_state".into(),
                observed_at_unix_seconds: 122,
                state_certificate_verified: false,
                membership_complete: false,
                registry_version: None,
            },
        },
        registered_without_observed_key: vec![registered],
        observed_key_without_registration: vec![observed],
    };
    let text = subnet_membership(&report);
    assert!(text.starts_with("Atomic snapshot: false"));
    assert!(text.contains(&format!(
        "Registered IDs without an observed key (1):\n{registered}"
    )));
    assert!(text.contains(&format!(
        "Observed key IDs absent from the returned Registry record (1):\n{observed}"
    )));
    assert!(text.contains("Query signature verified: false"));
    assert!(text.contains("State certificate verified: false"));
    assert!(text.contains("Complete Registry membership established: false"));
    assert!(text.contains("matching sets do not establish live nodes"));
    assert!(!text.contains('\u{1b}'));
    report.observation.observed_node_ids = vec![registered];
    report.registered_without_observed_key.clear();
    report.observed_key_without_registration.clear();
    let text = subnet_membership(&report);
    assert!(text.contains("Registered IDs without an observed key (0):"));
    assert!(text.contains("matching sets do not establish live nodes"));
}

#[test]
fn activity_text_preserves_both_samples_and_avoids_health_verdicts() {
    let observation = |total, verified| SubnetMetricsReport {
        schema: 1,
        subnet_id: Principal::from_slice(&[1]),
        num_canisters: 0,
        canister_state_bytes: 0,
        consumed_cycles_total: u128::MAX,
        update_transactions_total: total,
        evidence: SubnetMetricsEvidence {
            endpoint: "fixture",
            source_path: "/read_state".into(),
            state_tree_path: "/subnet/fixture/metrics".into(),
            observed_at_unix_seconds: 123,
            state_certificate_verified: verified,
        },
    };
    let mut report = SubnetActivityReport {
        schema: 1,
        interval_seconds: 10,
        before: observation(0, false),
        after: observation(u64::MAX, true),
        message_change: MessageChange::Increased { by: u64::MAX },
    };
    let text = subnet_activity(&report);
    assert!(text.contains("Requested wait between reads: 10 seconds"));
    assert!(text.contains("State certificate verified: false"));
    assert!(text.contains("State certificate verified: true"));
    assert!(text.contains("Replicated-message counter: increased by 18446744073709551615"));
    assert_eq!(text.matches(&u128::MAX.to_string()).count(), 2);
    assert!(text.ends_with(
        "Two samples do not establish uptime, throughput or a certified time interval."
    ));
    report.after.update_transactions_total = 0;
    report.message_change = MessageChange::Unchanged;
    assert!(subnet_activity(&report).contains("unchanged (does not establish downtime)"));
    report.before.update_transactions_total = u64::MAX;
    report.message_change = MessageChange::Decreased;
    assert!(subnet_activity(&report).contains("decreased (cause and state ordering unknown)"));
}

#[test]
fn metrics_text_preserves_precision_and_observation_limits() {
    let report = SubnetMetricsReport {
        schema: 1,
        subnet_id: Principal::from_slice(&[1]),
        num_canisters: 0,
        canister_state_bytes: 0,
        consumed_cycles_total: u128::MAX,
        update_transactions_total: u64::MAX,
        evidence: SubnetMetricsEvidence {
            endpoint: "fixture",
            source_path: "/read_state".into(),
            state_tree_path: "/subnet/fixture/metrics".into(),
            observed_at_unix_seconds: 123,
            state_certificate_verified: false,
        },
    };
    let text = subnet_metrics(&report);
    assert!(text.contains("State certificate verified: false\n\nSubnet:"));
    assert!(text.contains("Current canisters: 0"));
    assert!(text.contains("Canister state bytes (refreshed every 10 blocks): 0"));
    assert!(text.contains(&format!(
        "Nominal cycles consumed (may decrease): {}",
        u128::MAX
    )));
    assert!(text.contains(&format!("Replicated messages total: {}", u64::MAX)));
    assert!(text.ends_with("These values do not establish available capacity, rates or uptime."));
}

#[test]
fn subnet_list_text_labels_record_version_and_preserves_evidence() {
    let id = Principal::from_slice(&[1]);
    let report = SubnetListReport {
        schema: 1,
        subnet_ids: vec![id],
        evidence: SubnetListEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "get_value",
            source_key: "subnet_list",
            observed_at_unix_seconds: 123,
            record_version: 456,
            record_timestamp_nanoseconds: u64::MAX,
            query_signature_verified: false,
            result_certified: false,
            selected_record_complete: true,
        },
    };
    let text = subnet_list(&report);
    assert!(text.contains("Record mutation version: 456"));
    assert!(text.contains("Record mutation timestamp: 18446744073709551615 Unix nanoseconds"));
    assert!(text.contains("Query signature verified: false"));
    assert!(text.ends_with(&format!(
        "Selected record complete: true\n\nRegistered subnets (1):\n{id}"
    )));
}

#[test]
fn placement_text_uses_evidence_instead_of_hardcoded_verification_claims() {
    let report = PlacementReport {
        schema: 1,
        canister_id: Principal::from_slice(&[1]),
        subnet_id: Principal::from_slice(&[2]),
        evidence: PlacementEvidence {
            endpoint: "fixture",
            source_canister: "registry",
            source_method: "lookup",
            observed_at_unix_seconds: 123,
            query_signature_verified: false,
            result_certified: false,
            registry_version: Some(456),
            selected_lookup_complete: true,
        },
    };
    let text = placement(&report);
    assert!(text.contains("Query signature verified: false"));
    assert!(text.contains("Registry version: 456\n\nCanister:"));
    assert!(text.contains(&format!("Subnet: {}", report.subnet_id)));
}

#[test]
fn subnet_text_preserves_coverage_and_separates_metadata_from_inventory() {
    let node = Principal::from_slice(&[1]);
    let mut report = SubnetReport {
        schema: 1,
        subnet_id: Principal::from_slice(&[2]),
        observed_node_ids: vec![node],
        observed_subnet_type: None,
        evidence: SubnetEvidence {
            endpoint: "fixture",
            source_path: "/read_state".into(),
            observed_at_unix_seconds: 123,
            state_certificate_verified: true,
            membership_complete: false,
            registry_version: None,
        },
    };
    let text = subnet(&report);
    assert!(text.contains("Complete Registry membership established: false"));
    assert!(text.contains("Registry version: unavailable\n\nSubnet:"));
    assert!(text.ends_with(&format!("Observed node identities (1):\n{node}")));
    assert!(
        text.contains("Observed subnet type (subnet-certified, not Registry-confirmed): absent\n")
    );
    for (raw, rendered) in [
        ("cloud_engine", "\"cloud_engine\""),
        ("", "\"\""),
        (
            "future\n\u{1b}[31m\u{202e}",
            "\"future\\n\\u{1b}[31m\\u{202e}\"",
        ),
    ] {
        report.observed_subnet_type = Some(raw.into());
        let text = subnet(&report);
        assert!(text.contains(&format!("Registry-confirmed): {rendered}\n")));
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\u{202e}'));
    }
}
