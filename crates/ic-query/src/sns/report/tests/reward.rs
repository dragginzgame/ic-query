use crate::sns::report::tests::{fixtures::*, *};

#[test]
fn reward_checkpoint_collects_stable_brackets_and_exhausted_rows_in_order() {
    let source = FixtureSnsRewardSource::new(vec![fixture_reward_page(
        vec![fixture_reward_row(1), fixture_reward_row(2)],
        None,
    )]);
    let report =
        build_sns_reward_checkpoint_report_with_source(&reward_checkpoint_request("1"), &source)
            .expect("stable checkpoint");
    let text = sns_reward_checkpoint_report_text(&report);

    assert_eq!(
        source.calls(),
        [
            "version",
            "parameters",
            "event",
            "page",
            "event",
            "parameters",
            "version",
        ]
    );
    assert_eq!(report.page_count, 1);
    assert_eq!(report.row_count, 2);
    assert_eq!(report.unique_neuron_id_count, 2);
    assert_eq!(report.client_query_count, 9);
    assert_eq!(report.aggregate_maturity_e8s_equivalent, 300);
    assert_eq!(report.aggregate_staked_maturity_e8s_equivalent, 30);
    assert_eq!(report.aggregate_combined_maturity_e8s_equivalent, 330);
    assert_eq!(
        report.collection_status,
        SnsRewardCollectionStatus::ApiExhaustedObserved
    );
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(
        report.maturity_conversion_policy_observed_status,
        SnsPolicyObservationStatus::ObservedSatisfied
    );
    assert!(text.contains("collection_status: api_exhausted_observed"));
    assert!(text.contains("row_count: 2"));
    assert!(!text.contains(&report.rows[0].neuron_id));
}

#[test]
fn reward_checkpoint_rejects_invalid_request_before_source_access() {
    let source = FixtureSnsRewardSource::new(vec![fixture_reward_page(Vec::new(), None)]);
    let mut request = reward_checkpoint_request("1");
    request.network = "local".to_string();
    assert!(matches!(
        build_sns_reward_checkpoint_report_with_source(&request, &source),
        Err(SnsHostError::UnsupportedNetwork { network }) if network == "local"
    ));
    assert!(source.calls().is_empty());

    let source = FixtureSnsRewardSource::new(vec![fixture_reward_page(Vec::new(), None)]);
    let request = reward_checkpoint_request("1").with_max_pages(Some(0));
    assert!(matches!(
        build_sns_reward_checkpoint_report_with_source(&request, &source),
        Err(SnsHostError::InvalidRewardCheckpointPageCap { max_pages: 0 })
    ));
    assert!(source.calls().is_empty());
}

#[test]
fn live_reward_source_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new(
        "local",
        "not a valid endpoint",
        "2026-08-03T00:00:00Z",
        "test",
    );
    let sns = fixture_sns_a();

    let errors = [
        LiveSnsSource
            .fetch_sns_reward_running_version(&request, &sns)
            .expect_err("running-version source must reject non-mainnet"),
        LiveSnsSource
            .fetch_sns_reward_parameters(&request, &sns)
            .expect_err("parameter source must reject non-mainnet"),
        LiveSnsSource
            .fetch_sns_reward_event(&request, &sns)
            .expect_err("reward-event source must reject non-mainnet"),
        LiveSnsSource
            .fetch_sns_reward_neuron_page(&request, &sns, SNS_REWARD_CHECKPOINT_PAGE_SIZE, None)
            .expect_err("neuron-page source must reject non-mainnet"),
    ];

    assert!(errors.into_iter().all(|error| matches!(
        error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    )));
}

#[test]
fn reward_checkpoint_rejects_any_changed_complete_bracket() {
    for (component, expected) in [
        ("parameters", "nervous-system parameters"),
        ("event", "reward event"),
        ("version", "running SNS version"),
    ] {
        let source = FixtureSnsRewardSource::unstable(
            vec![fixture_reward_page(vec![fixture_reward_row(1)], None)],
            component,
        );
        assert!(matches!(
            build_sns_reward_checkpoint_report_with_source(
                &reward_checkpoint_request("1"),
                &source,
            ),
            Err(SnsHostError::UnstableRewardCheckpoint { component }) if component == expected
        ));
    }
}

#[test]
fn reward_checkpoint_rejects_overlap_and_noncanonical_cursor_evidence() {
    let mut full_rows = (1..=100).map(fixture_reward_row).collect::<Vec<_>>();
    let final_cursor = SnsNeuronId { id: vec![100; 32] };
    let mut state = SnsRewardCollectionState::new();
    state
        .ingest_page(fixture_reward_page(full_rows.clone(), Some(final_cursor)))
        .expect("first full page");
    assert!(matches!(
        state.ingest_page(fixture_reward_page(
            vec![fixture_reward_row(100)],
            None,
        )),
        Err(SnsHostError::InvalidSourceData {
            capability: "SNS reward checkpoint",
            reason,
        }) if reason.contains("does not increase")
    ));

    let short_with_cursor = fixture_reward_page(
        vec![fixture_reward_row(1)],
        Some(SnsNeuronId { id: vec![1; 32] }),
    );
    assert!(matches!(
        validate_mainnet_sns_reward_neuron_page(&short_with_cursor),
        Err(SnsHostError::InvalidSourceData { reason, .. })
            if reason.contains("must not advertise a cursor")
    ));

    full_rows[99].neuron_id = "ff".repeat(32);
    let wrong_cursor = fixture_reward_page(full_rows, Some(SnsNeuronId { id: vec![100; 32] }));
    assert!(matches!(
        validate_mainnet_sns_reward_neuron_page(&wrong_cursor),
        Err(SnsHostError::InvalidSourceData { reason, .. })
            if reason.contains("does not equal final neuron id")
    ));
}

#[test]
fn reward_checkpoint_rejects_diagnostic_cap_before_exhaustion() {
    let rows = (1..=100).map(fixture_reward_row).collect::<Vec<_>>();
    let source = FixtureSnsRewardSource::new(vec![
        fixture_reward_page(rows, Some(SnsNeuronId { id: vec![100; 32] })),
        fixture_reward_page(Vec::new(), None),
    ]);
    let request = reward_checkpoint_request("1").with_max_pages(Some(1));

    assert!(matches!(
        build_sns_reward_checkpoint_report_with_source(&request, &source),
        Err(SnsHostError::IncompleteRewardCheckpoint {
            pages_fetched: 1,
            rows_fetched: 100,
            reason,
        }) if reason.contains("diagnostic max_pages 1")
    ));
}

#[test]
fn reward_checkpoint_enforces_parameter_derived_collection_ceiling() {
    for ceiling in [None, Some(0), Some(200_001)] {
        let source = FixtureSnsRewardSource::new(vec![fixture_reward_page(Vec::new(), None)])
            .with_max_number_of_neurons(ceiling);
        assert!(matches!(
            build_sns_reward_checkpoint_report_with_source(
                &reward_checkpoint_request("1"),
                &source,
            ),
            Err(SnsHostError::InvalidRewardCheckpointCeiling { value, maximum: 200_000 })
                if value == ceiling
        ));
    }

    let source = FixtureSnsRewardSource::new(vec![fixture_reward_page(
        vec![fixture_reward_row(1), fixture_reward_row(2)],
        None,
    )])
    .with_max_number_of_neurons(Some(1));
    assert!(matches!(
        build_sns_reward_checkpoint_report_with_source(&reward_checkpoint_request("1"), &source),
        Err(SnsHostError::InvalidSourceData { reason, .. })
            if reason.contains("above mandatory ceiling 1")
    ));
}

#[test]
fn reward_checkpoint_collection_rejects_rows_after_exhaustion() {
    let mut state = SnsRewardCollectionState::new();
    state
        .ingest_page(fixture_reward_page(vec![fixture_reward_row(1)], None))
        .expect("short page exhausts the API");

    assert!(matches!(
        state.ingest_page(fixture_reward_page(Vec::new(), None)),
        Err(SnsHostError::InvalidSourceData { reason, .. })
            if reason.contains("after reported API exhaustion")
    ));
}

#[test]
fn reward_row_policy_fails_closed_but_preserves_known_violations() {
    let mut row = fixture_reward_row(1);
    row.permissions[0].principal = None;
    row.permissions[0]
        .permission_types
        .push(SnsNeuronPermissionValue::from_code(11));
    let observations = row.derived_policy_observations();
    assert_eq!(
        observations,
        (
            SnsPolicyObservationStatus::Unassessable,
            SnsPolicyObservationStatus::Unassessable,
        )
    );

    row.permissions[0]
        .permission_types
        .push(SnsNeuronPermissionValue::from_code(7));
    row.maturity_mint_conversion_observed_disabled = SnsPolicyObservationStatus::Violated;
    row.manual_maturity_staking_observed_disabled = SnsPolicyObservationStatus::Unassessable;
    assert_eq!(
        row.derived_policy_observations().0,
        SnsPolicyObservationStatus::Violated
    );
}
