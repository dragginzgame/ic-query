use super::*;
use crate::nns::{
    MAINNET_GOVERNANCE_CANISTER_ID,
    governance::NnsGovernanceRequest,
    proposals::{NnsProposalBallotRow, NnsProposalTally, nns_proposal_activity_report_text},
};

const SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[test]
fn complete_activity_is_canonical_and_input_order_independent() {
    let collection = complete_collection(3);
    let proposals = vec![
        proposal_row(3, 172_799, 4, 4, 9),
        proposal_row(1, 86_399, -7, 1, 3),
        proposal_row(2, 86_400, 0, 4, 3),
    ];

    let report = build_nns_proposal_activity_report(
        &NnsProposalActivityRequest::default(),
        &collection,
        &proposals,
    )
    .expect("complete activity report");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, "ic");
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.collection_page_count, 1);
    assert_eq!(report.collected_proposal_count, 3);
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(report.included_proposal_count, 3);
    assert_eq!(report.excluded_before_from_count, 0);
    assert_eq!(report.excluded_at_or_after_until_count, 0);
    assert_eq!(
        report.earliest_included_proposal_timestamp_seconds,
        Some(86_399)
    );
    assert_eq!(
        report.latest_included_proposal_timestamp_seconds,
        Some(172_799)
    );
    assert_eq!(
        report
            .topic_counts
            .iter()
            .map(|row| (row.topic, row.topic_text, row.proposal_count))
            .collect::<Vec<_>>(),
        vec![
            (-7, NnsProposalTopic::Unspecified, 1),
            (0, NnsProposalTopic::Unspecified, 1),
            (4, NnsProposalTopic::Governance, 1),
        ]
    );
    assert_eq!(
        report
            .status_counts
            .iter()
            .map(|row| (row.status, row.proposal_count))
            .collect::<Vec<_>>(),
        vec![(1, 1), (4, 2)]
    );
    assert_eq!(
        report
            .reward_status_counts
            .iter()
            .map(|row| (row.reward_status, row.proposal_count))
            .collect::<Vec<_>>(),
        vec![(3, 2), (9, 1)]
    );
    assert_eq!(
        report
            .day_counts
            .iter()
            .map(|row| (row.day_start_timestamp_seconds, row.proposal_count))
            .collect::<Vec<_>>(),
        vec![(0, 1), (86_400, 2)]
    );

    let mut permuted = proposals;
    permuted.reverse();
    let permuted_report = build_nns_proposal_activity_report(
        &NnsProposalActivityRequest::default(),
        &collection,
        &permuted,
    )
    .expect("permuted activity report");
    assert_eq!(report, permuted_report);
    assert_eq!(
        serde_json::to_vec(&report).expect("serialize activity report"),
        serde_json::to_vec(&permuted_report).expect("serialize permuted activity report")
    );
    validate_nns_proposal_activity_report(&report).expect("validate built activity report");
}

#[test]
fn serialized_activity_report_round_trips_through_pure_validation() {
    let report = fixture_activity_report();
    let encoded = serde_json::to_vec(&report).expect("serialize activity report");
    let restored: NnsProposalActivityReport =
        serde_json::from_slice(&encoded).expect("deserialize activity report");

    validate_nns_proposal_activity_report(&restored).expect("validate restored activity report");
    assert_eq!(restored, report);
}

#[test]
fn retained_activity_validation_rejects_header_and_selection_corruption() {
    let mut report = fixture_activity_report();
    report.schema_version = 2;
    assert_invalid_report(&report, "schema version");

    let mut report = fixture_activity_report();
    report.network = "local".to_string();
    assert_invalid_report(&report, "network");

    let mut report = fixture_activity_report();
    report.governance_canister_id = "aaaaa-aa".to_string();
    assert_invalid_report(&report, "governance_canister_id");

    let mut report = fixture_activity_report();
    report.collection_page_count = 0;
    assert_invalid_report(&report, "at least one collection page");

    let mut report = fixture_activity_report();
    report.point_in_time_guaranteed = true;
    assert_invalid_report(&report, "point-in-time");

    let mut report = fixture_activity_report();
    report.source = NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: "not-an-endpoint".to_string(),
        fetched_by: "fixture".to_string(),
    };
    assert_invalid_report(&report, "invalid collection source");

    let mut report = fixture_activity_report();
    report.source = NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
        collector_canister_id: "not-a-principal".to_string(),
    };
    assert_invalid_report(&report, "invalid collection provenance");

    let mut report = fixture_activity_report();
    report.from_proposal_timestamp_seconds = Some(200_000);
    report.until_proposal_timestamp_seconds = Some(100_000);
    assert_invalid_report(&report, "must be below");

    let mut report = fixture_activity_report();
    report.excluded_before_from_count = 1;
    assert_invalid_report(&report, "without a lower bound");

    let mut report = fixture_activity_report();
    report.collected_proposal_count += 1;
    assert_invalid_report(&report, "selection accounts");

    let mut report = fixture_activity_report();
    report.earliest_included_proposal_timestamp_seconds = None;
    assert_invalid_report(&report, "range presence");

    let mut report = fixture_activity_report();
    report.from_proposal_timestamp_seconds = Some(86_400);
    assert_invalid_report(&report, "precedes the lower bound");
}

#[test]
fn retained_activity_validation_rejects_noncanonical_dimensions() {
    let mut report = fixture_activity_report();
    report.topic_counts.swap(0, 1);
    assert_invalid_report(&report, "topic count rows");

    let mut report = fixture_activity_report();
    report.topic_counts[0].topic_text = NnsProposalTopic::NodeAdmin;
    assert_invalid_report(&report, "topic classification");

    let mut report = fixture_activity_report();
    report.topic_counts[0].proposal_count = 0;
    assert_invalid_report(&report, "topic count row must be nonzero");

    let mut report = fixture_activity_report();
    report.topic_counts[0].proposal_count += 1;
    assert_invalid_report(&report, "topic counts sum");

    let mut report = fixture_activity_report();
    report.status_counts[0].status_text = NnsProposalStatus::Failed;
    assert_invalid_report(&report, "status classification");

    let mut report = fixture_activity_report();
    report.reward_status_counts.swap(0, 1);
    assert_invalid_report(&report, "reward-status count rows");

    let mut report = fixture_activity_report();
    report.day_counts[0].day_start_timestamp_seconds = 1;
    assert_invalid_report(&report, "00:00:00 UTC");

    let mut report = fixture_activity_report();
    report.day_counts.swap(0, 1);
    assert_invalid_report(&report, "day count rows");

    let mut report = fixture_activity_report();
    report.earliest_included_proposal_timestamp_seconds = Some(86_400);
    assert_invalid_report(&report, "day count endpoints");
}

#[test]
fn half_open_window_counts_both_exclusion_sides_and_empty_windows() {
    let collection = complete_collection(3);
    let proposals = vec![
        proposal_row(1, 86_399, 4, 1, 3),
        proposal_row(2, 86_400, 4, 4, 3),
        proposal_row(3, 172_799, 4, 4, 3),
    ];
    let request = NnsProposalActivityRequest {
        from_proposal_timestamp_seconds: Some(86_400),
        until_proposal_timestamp_seconds: Some(172_799),
    };

    let report = build_nns_proposal_activity_report(&request, &collection, &proposals)
        .expect("windowed activity report");

    assert_eq!(report.included_proposal_count, 1);
    assert_eq!(report.excluded_before_from_count, 1);
    assert_eq!(report.excluded_at_or_after_until_count, 1);
    assert_eq!(
        report.earliest_included_proposal_timestamp_seconds,
        Some(86_400)
    );
    assert_eq!(
        report.latest_included_proposal_timestamp_seconds,
        Some(86_400)
    );
    assert_eq!(report.day_counts[0].day_start_timestamp_seconds, 86_400);

    let empty = build_nns_proposal_activity_report(
        &NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: Some(200_000),
            until_proposal_timestamp_seconds: Some(300_000),
        },
        &collection,
        &proposals,
    )
    .expect("empty valid window");
    assert_eq!(empty.included_proposal_count, 0);
    assert_eq!(empty.excluded_before_from_count, 3);
    assert_eq!(empty.excluded_at_or_after_until_count, 0);
    assert!(empty.earliest_included_proposal_timestamp_seconds.is_none());
    assert!(empty.latest_included_proposal_timestamp_seconds.is_none());
    assert!(empty.topic_counts.is_empty());
    assert!(empty.status_counts.is_empty());
    assert!(empty.reward_status_counts.is_empty());
    assert!(empty.day_counts.is_empty());
}

#[test]
fn builder_rejects_incomplete_invalid_and_mismatched_collection_inputs() {
    let governance = NnsGovernanceRequest::replica_query(
        "ic",
        SOURCE_ENDPOINT,
        "2026-08-15T00:00:00Z",
        "fixture",
    );
    let ready = NnsProposalCollectionState::new(&governance, 1, 1).expect("ready state");
    assert!(matches!(
        build_nns_proposal_activity_report(&NnsProposalActivityRequest::default(), &ready, &[]),
        Err(NnsProposalActivityError::CollectionNotComplete {
            status: NnsProposalCollectionStatus::Ready
        })
    ));

    let page_limited = page_limited_collection();
    assert!(matches!(
        build_nns_proposal_activity_report(
            &NnsProposalActivityRequest::default(),
            &page_limited,
            &[proposal_row(1, 1, 4, 1, 3)]
        ),
        Err(NnsProposalActivityError::CollectionNotComplete {
            status: NnsProposalCollectionStatus::PageLimitReached
        })
    ));

    assert!(matches!(
        build_nns_proposal_activity_report(
            &NnsProposalActivityRequest {
                from_proposal_timestamp_seconds: Some(10),
                until_proposal_timestamp_seconds: Some(10),
            },
            &complete_collection(1),
            &[proposal_row(1, 1, 4, 1, 3)]
        ),
        Err(NnsProposalActivityError::InvalidTimeWindow {
            from_proposal_timestamp_seconds: 10,
            until_proposal_timestamp_seconds: 10
        })
    ));

    assert!(matches!(
        build_nns_proposal_activity_report(
            &NnsProposalActivityRequest::default(),
            &complete_collection(2),
            &[proposal_row(1, 1, 4, 1, 3)]
        ),
        Err(NnsProposalActivityError::ProposalCountMismatch {
            expected: 2,
            actual: 1
        })
    ));

    let mut invalid = serde_json::to_value(complete_collection(1)).expect("serialize state");
    invalid["schema_version"] = serde_json::json!(9);
    let invalid: NnsProposalCollectionState =
        serde_json::from_value(invalid).expect("deserialize invalid state");
    assert!(matches!(
        build_nns_proposal_activity_report(
            &NnsProposalActivityRequest::default(),
            &invalid,
            &[proposal_row(1, 1, 4, 1, 3)]
        ),
        Err(NnsProposalActivityError::InvalidCollectionState { .. })
    ));
}

#[test]
fn builder_rejects_malformed_rows_before_applying_the_view() {
    let collection = complete_collection(1);
    let request = NnsProposalActivityRequest {
        from_proposal_timestamp_seconds: Some(1_000),
        until_proposal_timestamp_seconds: None,
    };

    let mut row = proposal_row(1, 1, 4, 1, 3);
    row.proposal_id = None;
    assert!(matches!(
        build_nns_proposal_activity_report(&request, &collection, &[row]),
        Err(NnsProposalActivityError::MissingProposalId)
    ));

    assert!(matches!(
        build_nns_proposal_activity_report(&request, &collection, &[proposal_row(0, 1, 4, 1, 3)]),
        Err(NnsProposalActivityError::ZeroProposalId)
    ));
    assert!(matches!(
        build_nns_proposal_activity_report(&request, &collection, &[proposal_row(1, 0, 4, 1, 3)]),
        Err(NnsProposalActivityError::ZeroProposalTimestamp { proposal_id: 1 })
    ));

    let mut row = proposal_row(1, 1, 4, 1, 3);
    row.topic_text = NnsProposalTopic::NodeAdmin;
    assert!(matches!(
        build_nns_proposal_activity_report(&request, &collection, &[row]),
        Err(NnsProposalActivityError::TopicClassificationMismatch { .. })
    ));

    let mut row = proposal_row(1, 1, 4, 1, 3);
    row.status_text = NnsProposalStatus::Failed;
    assert!(matches!(
        build_nns_proposal_activity_report(&request, &collection, &[row]),
        Err(NnsProposalActivityError::StatusClassificationMismatch { .. })
    ));

    let mut row = proposal_row(1, 1, 4, 1, 3);
    row.reward_status_text = NnsProposalRewardStatus::Ineligible;
    assert!(matches!(
        build_nns_proposal_activity_report(&request, &collection, &[row]),
        Err(NnsProposalActivityError::RewardStatusClassificationMismatch { .. })
    ));

    assert!(matches!(
        build_nns_proposal_activity_report(
            &NnsProposalActivityRequest::default(),
            &complete_collection(2),
            &[proposal_row(1, 1, 4, 1, 3), proposal_row(1, 2, 4, 1, 3)]
        ),
        Err(NnsProposalActivityError::DuplicateProposalId { proposal_id: 1 })
    ));
}

#[test]
fn empty_report_json_retains_raw_fields_and_text_retains_every_section() {
    let report = build_nns_proposal_activity_report(
        &NnsProposalActivityRequest {
            from_proposal_timestamp_seconds: Some(2),
            until_proposal_timestamp_seconds: Some(3),
        },
        &complete_collection(1),
        &[proposal_row(1, 1, 4, 1, 3)],
    )
    .expect("empty activity projection");
    let json = serde_json::to_value(&report).expect("serialize activity report");

    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["source"]["source_transport"], "replica_query");
    assert_eq!(json["from_proposal_timestamp_seconds"], 2);
    assert_eq!(json["until_proposal_timestamp_seconds"], 3);
    assert_eq!(json["included_proposal_count"], 0);
    assert_eq!(json["point_in_time_guaranteed"], false);
    assert_eq!(json["topic_counts"], serde_json::json!([]));

    let text = nns_proposal_activity_report_text(&report);
    assert!(text.contains("source_transport: replica_query"));
    assert!(text.contains("point_in_time_guaranteed: no"));
    assert!(
        text.contains(
            "\n\ntopics:\n-\n\nstatuses:\n-\n\nreward_statuses:\n-\n\ndaily_activity:\n-"
        )
    );
}

#[test]
fn count_increment_reports_overflow() {
    let mut count = u64::MAX;
    assert!(matches!(
        increment_count(&mut count, "fixture_count"),
        Err(NnsProposalActivityError::AccountingOverflow {
            field: "fixture_count"
        })
    ));
}

fn fixture_activity_report() -> NnsProposalActivityReport {
    build_nns_proposal_activity_report(
        &NnsProposalActivityRequest::default(),
        &complete_collection(3),
        &[
            proposal_row(3, 172_799, 4, 4, 9),
            proposal_row(1, 86_399, -7, 1, 3),
            proposal_row(2, 86_400, 0, 4, 3),
        ],
    )
    .expect("fixture activity report")
}

fn assert_invalid_report(report: &NnsProposalActivityReport, expected_reason: &str) {
    let error = validate_nns_proposal_activity_report(report).expect_err("invalid report");
    assert!(
        error.reason.contains(expected_reason),
        "unexpected validation reason: {}",
        error.reason
    );
}

fn complete_collection(proposals_fetched: usize) -> NnsProposalCollectionState {
    let page_size = u32::try_from(proposals_fetched.max(1)).expect("small fixture collection");
    serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "network": "ic",
        "governance_canister_id": MAINNET_GOVERNANCE_CANISTER_ID,
        "requested_source": {
            "source_transport": "replica_query",
            "endpoint": SOURCE_ENDPOINT,
            "fetched_by": "fixture"
        },
        "source": {
            "source_transport": "replica_query",
            "endpoint": SOURCE_ENDPOINT,
            "fetched_by": "fixture"
        },
        "page_size": page_size,
        "max_pages": 1,
        "pages_fetched": 1,
        "proposals_fetched": proposals_fetched,
        "next_before_proposal_id": null,
        "started_at": "2026-08-15T00:00:00Z",
        "updated_at": "2026-08-15T00:00:01Z",
        "status": "complete"
    }))
    .expect("complete collection fixture")
}

fn page_limited_collection() -> NnsProposalCollectionState {
    serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "network": "ic",
        "governance_canister_id": MAINNET_GOVERNANCE_CANISTER_ID,
        "requested_source": {
            "source_transport": "replica_query",
            "endpoint": SOURCE_ENDPOINT,
            "fetched_by": "fixture"
        },
        "source": {
            "source_transport": "replica_query",
            "endpoint": SOURCE_ENDPOINT,
            "fetched_by": "fixture"
        },
        "page_size": 1,
        "max_pages": 1,
        "pages_fetched": 1,
        "proposals_fetched": 1,
        "next_before_proposal_id": 2,
        "started_at": "2026-08-15T00:00:00Z",
        "updated_at": "2026-08-15T00:00:01Z",
        "status": "page_limit_reached"
    }))
    .expect("page-limited collection fixture")
}

fn proposal_row(
    proposal_id: u64,
    proposal_timestamp_seconds: u64,
    topic: i32,
    status: i32,
    reward_status: i32,
) -> NnsProposalRow {
    NnsProposalRow {
        proposal_id: Some(proposal_id),
        proposer_neuron_id: Some(99),
        topic,
        topic_text: NnsProposalTopic::from_code(topic),
        status,
        status_text: NnsProposalStatus::from_code(status),
        reward_status,
        reward_status_text: NnsProposalRewardStatus::from_code(reward_status),
        title: Some(format!("Proposal {proposal_id}")),
        summary: "Fixture proposal".to_string(),
        url: String::new(),
        action_text: None,
        reject_cost_e8s: 100_000_000,
        proposal_timestamp_seconds,
        proposed_at: "fixture".to_string(),
        deadline_timestamp_seconds: None,
        deadline_at: None,
        decided_timestamp_seconds: 0,
        decided_at: None,
        executed_timestamp_seconds: 0,
        executed_at: None,
        failed_timestamp_seconds: 0,
        failed_at: None,
        reward_event_round: 0,
        total_potential_voting_power: None,
        latest_tally: None::<NnsProposalTally>,
        ballot_count: 0,
        ballots: Vec::<NnsProposalBallotRow>::new(),
    }
}
