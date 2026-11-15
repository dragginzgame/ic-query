use std::time::Duration;

use ic_topology::node::{NodeError, NodeEvidence, NodeReport};
use ic_topology::subnet::configuration::{SubnetConfigurationEvidence, SubnetConfigurationReport};
use ic_topology::subnet::nodes::{NodeRead, NodeReadOutcome, SubnetNodesReport};

use super::*;

#[test]
fn distribution_output_preserves_partial_json_and_escapes_source_text() {
    use ic_topology::subnet::distribution::*;
    let inventory = inventory(NodeReadOutcome::NotStarted);
    let node_id = inventory.nodes[0].node_id;
    let mut report = SubnetDistributionReport {
        schema: 1,
        endpoint: "https://ic0.app",
        source_canister: "rwlgt-iiaaa-aaaaa-aaaaa-cai",
        version_method: "get_latest_version",
        requested_registry_version: u64::MAX,
        version_observed_at_unix_seconds: 100,
        started_at_unix_seconds: 99,
        elapsed: Duration::from_secs(60),
        query_signature_verified: true,
        result_certified: false,
        overall_deadline_reached: true,
        complete: false,
        configuration: inventory.configuration,
        nodes: vec![RecordRead {
            id: node_id,
            source_key: format!("node_record_{node_id}"),
            started_after: None,
            finished_after: None,
            outcome: RecordOutcome::NotStarted,
        }],
        operators: vec![],
        data_centers: vec![],
        relationships: vec![NodeRelationships {
            node_id,
            provider: Relationship::Unavailable {
                reason: UnavailableReason::NodeRecord,
            },
            data_center_reference: Relationship::Unavailable {
                reason: UnavailableReason::NodeRecord,
            },
            location: Relationship::Unavailable {
                reason: UnavailableReason::NodeRecord,
            },
        }],
        summary: DistributionSummary {
            selected_nodes: 1,
            provider_resolved: 0,
            provider_unknown: 1,
            data_center_references_resolved: 0,
            location_resolved: 0,
            location_unknown: 1,
            providers: vec![],
            data_centers: vec![],
            regions: vec![],
            region_unknown: 1,
            owners: vec![],
            owner_unknown: 1,
        },
    };
    let output = distribution_output(&report, true).unwrap();
    assert_eq!(output.exit_code, ExitCode::FAILURE);
    let json: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(json["requested_registry_version"], u64::MAX);
    assert_eq!(json["nodes"][0]["outcome"]["status"], "not_started");
    assert_eq!(
        json["relationships"][0]["provider"]["reason"],
        "node_record"
    );
    let text = distribution_output(&report, false).unwrap().text;
    assert!(text.contains("Providers: 0 / 1 resolved (1 unknown)"));
    assert!(text.contains("Locations: 0 / 1 resolved (1 unknown"));
    assert!(text.contains("Overall deadline reached"));
    assert!(text.contains("Consensus-certified: false"));

    // Presentation accepts declared groups; source text must never reach a terminal raw.
    let raw = "DC\n\u{1b}[31m";
    report.data_centers.push(RecordRead {
        id: raw.into(),
        source_key: "fixture".into(),
        started_after: None,
        finished_after: None,
        outcome: RecordOutcome::Read {
            report: DataCenterReport {
                id: raw.into(),
                region: raw.into(),
                owner: raw.into(),
                evidence: RecordEvidence {
                    source_method: "get_value",
                    record_version: 1,
                    record_timestamp_nanoseconds: 2,
                    observed_at_unix_seconds: 3,
                },
            },
        },
    });
    report.summary.data_centers.push(GroupCount {
        value: raw.into(),
        nodes: 1,
    });
    let text = distribution_output(&report, false).unwrap().text;
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("DC\\n\\u{1b}[31m"));
    let json: serde_json::Value =
        serde_json::from_str(&distribution_output(&report, true).unwrap().text).unwrap();
    assert_eq!(json["data_centers"][0]["outcome"]["report"]["region"], raw);

    report.complete = true;
    assert_eq!(
        distribution_output(&report, true).unwrap().exit_code,
        ExitCode::SUCCESS
    );
    assert_eq!(
        distribution_output(&report, false).unwrap().exit_code,
        ExitCode::SUCCESS
    );
}

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

#[test]
fn prices_output_keeps_daily_raw_values_partial_status_and_safe_text() {
    use ic_topology::cloud_engine::prices::*;
    let evidence = |method| PriceEvidence {
        endpoint: "https://ic0.app",
        source_canister: "q6cfj-fyaaa-aaaar-qb77q-cai",
        source_method: method,
        observed_at_unix_seconds: 123,
        query_signature_verified: true,
        result_certified: false,
    };
    let mut report = CloudEnginePricesReport {
        schema: 1,
        started_at_unix_seconds: 122,
        elapsed: Duration::from_secs(1),
        complete: false,
        atomic_snapshot: false,
        prices: PriceObservation {
            evidence: evidence("listMarketplacePrices"),
            entries: vec![MarketplacePrice {
                key: "type4,\x1b[31m\nDC".into(),
                node_type: serde_json::from_str("\"type4\"").unwrap(),
                provider: None,
                data_center: Some("\x1b[31m\nDC".into()),
                net_cycles_per_node_per_day: "123456789012345678901234567890".into(),
                gross_cycles_per_node_per_day: "223456789012345678901234567890".into(),
                updated_at_unix_nanoseconds: i64::MIN,
            }],
        },
        network_fee: FeeOutcome::Failed {
            error: AdmissionError::InvalidFee.into(),
        },
    };
    let output = prices_output(&report, true).unwrap();
    assert_eq!(output.exit_code, ExitCode::FAILURE);
    let json: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(
        json["prices"]["entries"][0]["net_cycles_per_node_per_day"],
        "123456789012345678901234567890"
    );
    assert_eq!(
        json["network_fee"]["error"]["admission"]["category"],
        "invalid_fee"
    );
    assert_eq!(json["prices"]["entries"][0]["data_center"], "\x1b[31m\nDC");
    let text = prices_output(&report, false).unwrap().text;
    assert!(text.contains("cycles per node per day") && text.contains("partial"));
    assert!(!text.contains('\x1b') && text.contains("\\nDC"));
    assert!(text.contains("prices retained"));
    report.network_fee = FeeOutcome::Read {
        markup: 0.25,
        evidence: evidence("getNetworkFee"),
    };
    report.complete = true;
    let output = prices_output(&report, false).unwrap();
    assert_eq!(output.exit_code, ExitCode::SUCCESS);
    assert!(output.text.contains("markup on net: 0.25"));
    report.prices.entries.clear();
    assert!(
        prices_output(&report, false)
            .unwrap()
            .text
            .contains("no declarations returned")
    );
    report.complete = false;
    report.network_fee = FeeOutcome::OverallDeadline;
    assert!(
        prices_output(&report, false)
            .unwrap()
            .text
            .contains("overall deadline reached")
    );
}

fn availability_fixture() -> ic_topology::cloud_engine::available::AvailableNodesReport {
    use ic_topology::cloud_engine::available::*;
    let principal = Principal::from_slice(&[1]);
    AvailableNodesReport {
        schema: 1,
        page_limit: 10,
        node_limit: 1000,
        started_at_unix_seconds: 123,
        elapsed: Duration::from_secs(3),
        endpoint: "https://ic0.app",
        source_canister: "q6cfj-fyaaa-aaaar-qb77q-cai",
        source_method: "listAvailableNodesPage",
        rejoining: None,
        query_signature_verified: true,
        result_certified: false,
        atomic_snapshot: false,
        point_in_time_inventory_complete: false,
        pagination_exhausted: false,
        registry_copy: RegistryCopyObservation {
            source_method: "registryCopyStatus",
            observed_at_unix_seconds: 123,
            version: 5,
            latest: 6,
            caught_up: false,
        },
        pages: vec![AvailablePage {
            start: None,
            next: Some(principal),
            registry_version: 6,
            observed_at_unix_seconds: 124,
            nodes: vec![AvailableNode {
                node_id: principal,
                operator_id: principal,
                provider_id: principal,
                node_type: serde_json::from_str("\"type4_5\"").unwrap(),
                data_center: Some("\x1b[31m\nDC".into()),
                sev_enabled: true,
            }],
        }],
        stop: TraversalStop::PageLimit,
    }
}

#[test]
fn available_output_distinguishes_exhaustion_from_inventory_completeness() {
    use ic_topology::cloud_engine::available::*;
    let mut report = availability_fixture();
    let output = available_output(&report, true).unwrap();
    assert_eq!(output.exit_code, ExitCode::FAILURE);
    let json: serde_json::Value = serde_json::from_str(&output.text).unwrap();
    assert_eq!(json["pagination_exhausted"], false);
    assert_eq!(json["point_in_time_inventory_complete"], false);
    assert_eq!(json["pages"][0]["nodes"][0]["data_center"], "\x1b[31m\nDC");
    assert_eq!(json["stop"]["reason"], "page_limit");
    let output = available_output(&report, false).unwrap();
    assert!(output.text.contains("partial: page limit") && output.text.contains("\\nDC"));
    assert!(!output.text.contains('\x1b'));
    assert!(output.text.contains("\"type4_5\"") && output.text.contains("not verified TEE"));
    report.pages.clear();
    report.stop = TraversalStop::Failed {
        start: None,
        observed_at_unix_seconds: Some(123),
        observed_registry_version: None,
        error: AvailableError::CopyNotReady,
    };
    assert!(
        available_output(&report, false)
            .unwrap()
            .text
            .contains("availability unknown")
    );
    report.stop = TraversalStop::Exhausted;
    report.pagination_exhausted = true;
    let output = available_output(&report, false).unwrap();
    assert_eq!(output.exit_code, ExitCode::SUCCESS);
    assert!(
        output.text.contains("no nodes returned") && output.text.contains("no atomic inventory")
    );
}

#[test]
fn candidates_output_retains_partial_evidence_and_distinguishes_unknown_prices() {
    use ic_topology::cloud_engine::{candidates::*, prices::*};
    let mut report = CandidatesReport {
        schema: 1,
        elapsed: Duration::from_secs(4),
        collection_complete: false,
        atomic_snapshot: false,
        availability: availability_fixture(),
        pricing: CandidatePricing::Failed {
            error: PriceError::OverallDeadline,
        },
    };
    let output = candidates_output(&report, false).unwrap();
    assert_eq!(output.exit_code, ExitCode::FAILURE);
    assert!(output.text.contains("matching coverage unknown"));
    let json: serde_json::Value =
        serde_json::from_str(&candidates_output(&report, true).unwrap().text).unwrap();
    assert_eq!(json["availability"]["stop"]["reason"], "page_limit");
    assert_eq!(json["pricing"]["error"]["category"], "overall_deadline");
    report.pricing = CandidatePricing::Read {
        report: CloudEnginePricesReport {
            schema: 1,
            elapsed: Duration::ZERO,
            started_at_unix_seconds: 130,
            complete: false,
            atomic_snapshot: false,
            prices: PriceObservation {
                evidence: PriceEvidence {
                    endpoint: "https://ic0.app",
                    source_canister: "q6cfj-fyaaa-aaaar-qb77q-cai",
                    source_method: "listMarketplacePrices",
                    observed_at_unix_seconds: 131,
                    query_signature_verified: true,
                    result_certified: false,
                },
                entries: vec![MarketplacePrice {
                    key: "type4_5,\x1b[31m\nDC".into(),
                    node_type: serde_json::from_str("\"type4_5\"").unwrap(),
                    provider: None,
                    data_center: Some("\x1b[31m\nDC".into()),
                    net_cycles_per_node_per_day: "123456789012345678901234567890".into(),
                    gross_cycles_per_node_per_day: "123456789012345678901234567891".into(),
                    updated_at_unix_nanoseconds: -1,
                }],
            },
            network_fee: FeeOutcome::OverallDeadline,
        },
        matches: vec![CandidateMatches {
            node_id: Principal::from_slice(&[1]),
            price_indices: vec![0],
        }],
    };
    let text = candidates_output(&report, false).unwrap().text;
    assert!(text.contains("0 none | 1 single | 0 multiple"));
    assert!(text.contains("| DECLARATION") && text.contains("+---"));
    assert!(!text.contains('\x1b') && text.contains("\\nDC"));
    assert!(text.contains("123456789012345678901234567890"));
    assert!(text.contains("No tariff selected") && text.contains("network fee unavailable"));
    let json: serde_json::Value =
        serde_json::from_str(&candidates_output(&report, true).unwrap().text).unwrap();
    assert_eq!(
        json["pricing"]["report"]["prices"]["entries"][0]["key"],
        "type4_5,\x1b[31m\nDC"
    );
    let CandidatePricing::Read { matches, .. } = &mut report.pricing else {
        panic!()
    };
    matches[0].price_indices.clear();
    assert!(
        candidates_output(&report, false)
            .unwrap()
            .text
            .contains("1 none | 0 single | 0 multiple")
    );
    report.collection_complete = true;
    assert_eq!(
        candidates_output(&report, true).unwrap().exit_code,
        ExitCode::SUCCESS
    );
}

#[test]
fn engine_output_preserves_omission_raw_evidence_and_membership_uncertainty() {
    use ic_topology::cloud_engine::engine::*;
    let mut report = EngineReport {
        schema: 1,
        requested_engine_id: " engine\n\x1b\u{202e} ".parse().unwrap(),
        started_at_unix_seconds: 12,
        elapsed: Duration::from_secs(1),
        atomic_snapshot: false,
        outcome: EngineOutcome::NotReturned,
        evidence: EngineEvidence {
            endpoint: "https://ic0.app",
            source_canister: "q6cfj-fyaaa-aaaar-qb77q-cai",
            source_method: "getEngineViews",
            observed_at_unix_seconds: 13,
            query_signature_verified: true,
            result_certified: false,
        },
    };
    let output = engine_output(&report, false).unwrap();
    assert_eq!(output.exit_code, ExitCode::SUCCESS);
    assert!(output.text.contains("unknown or hidden"));
    assert!(!output.text.contains('\x1b') && !output.text.contains('\u{202e}'));
    let json: serde_json::Value =
        serde_json::from_str(&engine_output(&report, true).unwrap().text).unwrap();
    assert_eq!(json["outcome"]["status"], "not_returned");
    assert_eq!(json["requested_engine_id"], " engine\n\x1b\u{202e} ");
    let node = Principal::from_slice(&[1]);
    report.outcome = EngineOutcome::Read {
        record: Box::new(EngineRecord {
            subnet_id: Some(node),
            operator_canister_id: None,
            record_node_ids: vec![node],
            deleted_at_unix_nanoseconds: Some(i64::MIN),
            registry_copy_subnet: None,
            membership: MembershipComparison::Unavailable,
        }),
    };
    let text = engine_output(&report, false).unwrap().text;
    assert!(
        text.contains("membership comparison unknown")
            && text.contains("| NODE")
            && text.contains("| unknown")
    );
    assert!(text.contains("Deletion timestamp recorded: -9223372036854775808"));
    let EngineOutcome::Read { record } = &mut report.outcome else {
        panic!()
    };
    record.registry_copy_subnet = Some(RegistryCopySubnet {
        node_ids: vec![],
        configured_replica_version: "v\n\x1b\u{202e}".into(),
        halted: true,
        maximum_state_size_bytes: Some(u64::MAX),
    });
    record.membership = MembershipComparison::Compared {
        record_only: vec![node],
        registry_copy_only: vec![],
    };
    let text = engine_output(&report, false).unwrap().text;
    assert!(
        text.contains("1 engine record / 0 Registry copy")
            && text.contains("1 record-only / 0 copy-only")
    );
    assert!(!text.contains('\x1b') && !text.contains('\u{202e}'));
    assert!(
        text.contains("18446744073709551615 bytes") && text.contains("Registered halt flag: true")
    );
    let json: serde_json::Value =
        serde_json::from_str(&engine_output(&report, true).unwrap().text).unwrap();
    assert_eq!(
        json["outcome"]["record"]["registry_copy_subnet"]["configured_replica_version"],
        "v\n\x1b\u{202e}"
    );
    assert_eq!(
        json["outcome"]["record"]["registry_copy_subnet"]["maximum_state_size_bytes"],
        u64::MAX
    );
    let EngineOutcome::Read { record } = &mut report.outcome else {
        panic!()
    };
    record
        .registry_copy_subnet
        .as_mut()
        .unwrap()
        .configured_replica_version
        .clear();
    assert!(
        engine_output(&report, false)
            .unwrap()
            .text
            .contains("standard engine version")
    );
}
