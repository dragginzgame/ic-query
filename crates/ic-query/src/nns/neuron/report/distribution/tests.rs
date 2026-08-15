use super::*;
use crate::nns::{
    governance::NnsGovernanceRequest,
    neuron::{NnsKnownNeuronData, nns_neuron_distribution_report_text},
};

const SOURCE_ENDPOINT: &str = "https://icp-api.io";

#[test]
fn complete_distribution_preserves_raw_dimensions_and_optional_coverage() {
    let neurons = fixture_neurons();
    let report = build_nns_neuron_distribution_report(&complete_collection(4), &neurons)
        .expect("complete neuron distribution");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.collection_page_count, 1);
    assert_eq!(report.collected_neuron_count, 4);
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(report.earliest_retrieved_at_timestamp_seconds, Some(100));
    assert_eq!(report.latest_retrieved_at_timestamp_seconds, Some(120));
    assert_eq!(report.total_effective_stake_e8s, 1_000);
    assert_eq!(report.reported_staked_maturity_neuron_count, 2);
    assert_eq!(report.unreported_staked_maturity_neuron_count, 2);
    assert_eq!(report.total_reported_staked_maturity_e8s_equivalent, 40);
    assert_eq!(report.reported_deciding_voting_power_neuron_count, 2);
    assert_eq!(report.unreported_deciding_voting_power_neuron_count, 2);
    assert_eq!(report.total_reported_deciding_voting_power, 50);
    assert_eq!(report.reported_potential_voting_power_neuron_count, 2);
    assert_eq!(report.unreported_potential_voting_power_neuron_count, 2);
    assert_eq!(report.total_reported_potential_voting_power, 90);
    assert_eq!(report.known_neuron_metadata_count, 2);
    assert_eq!(report.neurons_fund_join_timestamp_present_count, 2);
    assert_eq!(
        report
            .state_distribution
            .iter()
            .map(|row| {
                (
                    row.state,
                    row.state_text,
                    row.neuron_count,
                    row.effective_stake_e8s,
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (-7, NnsNeuronState::Unknown(-7), 1, 400),
            (1, NnsNeuronState::NotDissolving, 2, 300),
            (3, NnsNeuronState::Dissolved, 1, 300),
        ]
    );
    assert_eq!(
        report
            .visibility_distribution
            .iter()
            .map(|row| {
                (
                    row.visibility,
                    row.visibility_text,
                    row.neuron_count,
                    row.effective_stake_e8s,
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (None, NnsNeuronVisibility::Unknown, 1, 100),
            (Some(-8), NnsNeuronVisibility::UnknownCode(-8), 1, 400,),
            (Some(2), NnsNeuronVisibility::Public, 2, 500),
        ]
    );
    assert_eq!(
        report
            .neuron_type_distribution
            .iter()
            .map(|row| {
                (
                    row.neuron_type,
                    row.neuron_type_text,
                    row.neuron_count,
                    row.effective_stake_e8s,
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (None, NnsNeuronType::Unknown, 1, 100),
            (Some(1), NnsNeuronType::Seed, 2, 500),
            (Some(9), NnsNeuronType::UnknownCode(9), 1, 400),
        ]
    );
    validate_nns_neuron_distribution_report(&report).expect("validate built distribution");
}

#[test]
fn empty_complete_collection_produces_canonical_zero_distribution() {
    let report = build_nns_neuron_distribution_report(&complete_collection(0), &[])
        .expect("empty complete distribution");

    assert_eq!(report.collected_neuron_count, 0);
    assert_eq!(report.total_effective_stake_e8s, 0);
    assert_eq!(report.earliest_retrieved_at_timestamp_seconds, None);
    assert_eq!(report.latest_retrieved_at_timestamp_seconds, None);
    assert!(report.state_distribution.is_empty());
    assert!(report.visibility_distribution.is_empty());
    assert!(report.neuron_type_distribution.is_empty());
    assert_eq!(report.reported_staked_maturity_neuron_count, 0);
    assert_eq!(report.unreported_staked_maturity_neuron_count, 0);

    let text = nns_neuron_distribution_report_text(&report);
    assert!(text.contains("\n\nstates:\n-\n\nvisibilities:\n-\n\nneuron_types:\n-"));
}

#[test]
fn builder_rejects_incomplete_invalid_mismatched_and_malformed_inputs() {
    let governance = NnsGovernanceRequest::replica_query(
        MAINNET_NETWORK,
        SOURCE_ENDPOINT,
        "2026-08-15T00:00:00Z",
        "fixture",
    );
    let ready = NnsNeuronCollectionState::new(&governance, 2, 1).expect("ready state");
    assert!(matches!(
        build_nns_neuron_distribution_report(&ready, &[]),
        Err(NnsNeuronDistributionError::CollectionNotComplete {
            status: NnsNeuronCollectionStatus::Ready
        })
    ));

    let page_limited = page_limited_collection();
    assert!(matches!(
        build_nns_neuron_distribution_report(
            &page_limited,
            &[sample_neuron(1, 100, 1, Some(2), None, 100)]
        ),
        Err(NnsNeuronDistributionError::CollectionNotComplete {
            status: NnsNeuronCollectionStatus::PageLimitReached
        })
    ));

    assert!(matches!(
        build_nns_neuron_distribution_report(
            &complete_collection(2),
            &[sample_neuron(1, 100, 1, Some(2), None, 100)]
        ),
        Err(NnsNeuronDistributionError::NeuronCountMismatch {
            expected: 2,
            actual: 1
        })
    ));

    let mut mismatched = sample_neuron(1, 100, 1, Some(2), None, 100);
    mismatched.state_text = NnsNeuronState::Dissolved;
    assert!(matches!(
        build_nns_neuron_distribution_report(&complete_collection(1), &[mismatched]),
        Err(NnsNeuronDistributionError::InvalidNeuronRows { .. })
    ));

    let unordered = [
        sample_neuron(2, 100, 1, Some(2), None, 100),
        sample_neuron(1, 100, 1, Some(2), None, 100),
    ];
    assert!(matches!(
        build_nns_neuron_distribution_report(&complete_collection(2), &unordered),
        Err(NnsNeuronDistributionError::InvalidNeuronRows { .. })
    ));

    let mut invalid = serde_json::to_value(complete_collection(1)).expect("serialize state");
    invalid["schema_version"] = serde_json::json!(9);
    let invalid: NnsNeuronCollectionState =
        serde_json::from_value(invalid).expect("deserialize invalid state");
    assert!(matches!(
        build_nns_neuron_distribution_report(
            &invalid,
            &[sample_neuron(1, 100, 1, Some(2), None, 100)]
        ),
        Err(NnsNeuronDistributionError::InvalidCollectionState { .. })
    ));
}

#[test]
fn checked_distribution_accounting_rejects_overflow() {
    let neurons = [
        sample_neuron(1, u64::MAX, 1, Some(2), None, 100),
        sample_neuron(2, 1, 1, Some(2), None, 100),
    ];
    assert!(matches!(
        build_nns_neuron_distribution_report(&complete_collection(2), &neurons),
        Err(NnsNeuronDistributionError::AccountingOverflow {
            field: "total_effective_stake_e8s"
        })
    ));

    let mut first = sample_neuron(1, 0, 1, Some(2), None, 100);
    first.staked_maturity_e8s_equivalent = Some(u64::MAX);
    let mut second = sample_neuron(2, 0, 1, Some(2), None, 100);
    second.staked_maturity_e8s_equivalent = Some(1);
    assert!(matches!(
        build_nns_neuron_distribution_report(&complete_collection(2), &[first, second]),
        Err(NnsNeuronDistributionError::AccountingOverflow {
            field: "total_reported_staked_maturity_e8s_equivalent"
        })
    ));
}

#[test]
fn serialized_distribution_round_trips_and_text_separates_sections() {
    let report = fixture_distribution_report();
    let encoded = serde_json::to_vec(&report).expect("serialize distribution");
    let restored: NnsNeuronDistributionReport =
        serde_json::from_slice(&encoded).expect("deserialize distribution");

    validate_nns_neuron_distribution_report(&restored).expect("validate restored distribution");
    assert_eq!(restored, report);
    let json = serde_json::to_value(&report).expect("distribution JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["source"]["source_transport"], "replica_query");
    assert_eq!(json["total_effective_stake_e8s"], 1_000);
    assert_eq!(
        json["visibility_distribution"][0]["visibility"],
        serde_json::Value::Null
    );

    let text = nns_neuron_distribution_report_text(&report);
    assert!(text.contains("source_transport: replica_query"));
    assert!(text.contains("point_in_time_guaranteed: no"));
    assert!(text.contains("total_reported_staked_maturity_icp: 0.00"));
    assert!(text.contains("\n\nstates:\n"));
    assert!(text.contains("\n\nvisibilities:\n"));
    assert!(text.contains("\n\nneuron_types:\n"));
}

#[test]
fn retained_distribution_validation_rejects_corruption() {
    let mut report = fixture_distribution_report();
    report.schema_version = 2;
    assert_invalid_report(&report, "schema version");

    let mut report = fixture_distribution_report();
    report.point_in_time_guaranteed = true;
    assert_invalid_report(&report, "point-in-time");

    let mut report = fixture_distribution_report();
    report.source = NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: "invalid".to_string(),
        fetched_by: "fixture".to_string(),
    };
    assert_invalid_report(&report, "invalid collection source");

    let mut report = fixture_distribution_report();
    report.source = NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
        collector_canister_id: "not-a-principal".to_string(),
    };
    assert_invalid_report(&report, "invalid collection provenance");

    let mut report = fixture_distribution_report();
    report.collection_page_count = 6;
    assert_invalid_report(&report, "requires at least 5 collected neurons");

    let mut report = fixture_distribution_report();
    report.earliest_retrieved_at_timestamp_seconds = None;
    assert_invalid_report(&report, "retrieval timestamp range");

    let mut report = fixture_distribution_report();
    report.reported_deciding_voting_power_neuron_count += 1;
    assert_invalid_report(&report, "coverage accounts");

    let mut report = fixture_distribution_report();
    report.known_neuron_metadata_count = 5;
    assert_invalid_report(&report, "known_neuron_metadata_count");

    let mut report = fixture_distribution_report();
    report.state_distribution.swap(0, 1);
    assert_invalid_report(&report, "state distribution");

    let mut report = fixture_distribution_report();
    report.visibility_distribution[0].visibility_text = NnsNeuronVisibility::Private;
    assert_invalid_report(&report, "visibility classification");

    let mut report = fixture_distribution_report();
    report.neuron_type_distribution[0].neuron_count = 0;
    assert_invalid_report(&report, "at least one neuron");

    let mut report = fixture_distribution_report();
    report.state_distribution[0].effective_stake_e8s += 1;
    assert_invalid_report(&report, "effective stake sums");
}

fn fixture_distribution_report() -> NnsNeuronDistributionReport {
    build_nns_neuron_distribution_report(&complete_collection(4), &fixture_neurons())
        .expect("fixture distribution")
}

fn fixture_neurons() -> Vec<NnsNeuronRow> {
    let mut first = sample_neuron(1, 100, 1, None, None, 100);
    first.staked_maturity_e8s_equivalent = Some(10);
    first.deciding_voting_power = Some(20);
    first.potential_voting_power = None;
    first.known_neuron_data = Some(known_neuron(1));

    let mut second = sample_neuron(2, 200, 1, Some(2), Some(1), 110);
    second.staked_maturity_e8s_equivalent = None;
    second.deciding_voting_power = Some(30);
    second.potential_voting_power = Some(40);
    second.joined_community_fund_timestamp_seconds = Some(50);

    let mut third = sample_neuron(3, 300, 3, Some(2), Some(1), 105);
    third.staked_maturity_e8s_equivalent = Some(30);
    third.deciding_voting_power = None;
    third.potential_voting_power = Some(50);
    third.known_neuron_data = Some(known_neuron(3));
    third.joined_community_fund_timestamp_seconds = Some(60);

    let mut fourth = sample_neuron(4, 400, -7, Some(-8), Some(9), 120);
    fourth.staked_maturity_e8s_equivalent = None;
    fourth.deciding_voting_power = None;
    fourth.potential_voting_power = None;

    vec![first, second, third, fourth]
}

fn sample_neuron(
    neuron_id: u64,
    stake_e8s: u64,
    state: i32,
    visibility: Option<i32>,
    neuron_type: Option<i32>,
    retrieved_at_timestamp_seconds: u64,
) -> NnsNeuronRow {
    NnsNeuronRow {
        neuron_id,
        state,
        state_text: NnsNeuronState::from_code(state),
        visibility,
        visibility_text: NnsNeuronVisibility::from_code(visibility),
        neuron_type,
        neuron_type_text: NnsNeuronType::from_code(neuron_type),
        stake_e8s,
        staked_maturity_e8s_equivalent: None,
        dissolve_delay_seconds: 0,
        age_seconds: 0,
        created_timestamp_seconds: 1,
        retrieved_at_timestamp_seconds,
        voting_power: 0,
        deciding_voting_power: None,
        potential_voting_power: None,
        voting_power_refreshed_timestamp_seconds: None,
        joined_community_fund_timestamp_seconds: None,
        eight_year_gang_bonus_base_e8s: None,
        known_neuron_data: None,
        recent_ballots: Vec::new(),
    }
}

fn known_neuron(neuron_id: u64) -> NnsKnownNeuronData {
    NnsKnownNeuronData {
        name: format!("Neuron {neuron_id}"),
        description: None,
        links: Vec::new(),
    }
}

fn complete_collection(neurons_fetched: usize) -> NnsNeuronCollectionState {
    let page_size = u32::try_from(neurons_fetched.saturating_add(1)).expect("small fixture");
    serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "network": MAINNET_NETWORK,
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
        "neurons_fetched": neurons_fetched,
        "next_start_neuron_id": null,
        "started_at": "2026-08-15T00:00:00Z",
        "updated_at": "2026-08-15T00:00:01Z",
        "status": "complete"
    }))
    .expect("complete collection fixture")
}

fn page_limited_collection() -> NnsNeuronCollectionState {
    serde_json::from_value(serde_json::json!({
        "schema_version": 1,
        "network": MAINNET_NETWORK,
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
        "neurons_fetched": 1,
        "next_start_neuron_id": 1,
        "started_at": "2026-08-15T00:00:00Z",
        "updated_at": "2026-08-15T00:00:01Z",
        "status": "page_limit_reached"
    }))
    .expect("page-limited collection fixture")
}

fn assert_invalid_report(report: &NnsNeuronDistributionReport, expected_reason: &str) {
    let error = validate_nns_neuron_distribution_report(report).expect_err("invalid report");
    assert!(
        error.reason.contains(expected_reason),
        "unexpected validation reason: {}",
        error.reason
    );
}
