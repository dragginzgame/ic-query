use crate::sns::report::source::validate_mainnet_sns_neuron;
use crate::sns::report::tests::{fixtures::*, *};

#[test]
fn neuron_evidence_is_rejected_at_live_detail_page_and_restored_boundaries() {
    let mut row = fixture_reward_row(1);
    row.disburse_maturity_in_progress = fixture_sns_neuron().detail.disburse_maturity_in_progress;
    let (mint, staking) = row.derived_policy_observations();
    row.maturity_mint_conversion_observed_disabled = mint;
    row.manual_maturity_staking_observed_disabled = staking;
    let valid = build_sns_reward_checkpoint_report_with_source(
        &reward_checkpoint_request("1"),
        &FixtureSnsRewardSource::new(vec![fixture_reward_page(vec![row], None)]),
    )
    .expect("valid variable evidence");
    let mutations: [fn(&mut SnsRewardCheckpointRow); 8] = [
        |row| row.permissions[0].principal = Some("invalid".to_string()),
        |row| row.permissions[0].principal = Some(ROOT_A.to_uppercase()),
        |row| row.permissions.push(row.permissions[0].clone()),
        |row| {
            let permission = row.permissions[0].permission_types[0].clone();
            row.permissions[0].permission_types.push(permission);
        },
        |row| row.permissions[0].permission_types[0].name = "incorrect".to_string(),
        |row| {
            row.disburse_maturity_in_progress[0]
                .account_to_disburse_to
                .as_mut()
                .unwrap()
                .owner = Some("invalid".to_string());
        },
        |row| {
            row.disburse_maturity_in_progress[0]
                .account_to_disburse_to
                .as_mut()
                .unwrap()
                .subaccount_hex = Some("AB".repeat(32));
        },
        |row| {
            row.disburse_maturity_in_progress[0]
                .account_to_disburse_to
                .as_mut()
                .unwrap()
                .subaccount_hex = Some("ab".to_string());
        },
    ];
    for mutate in mutations {
        let mut restored = valid.clone();
        mutate(&mut restored.rows[0]);
        assert!(validate_sns_reward_checkpoint_report(&restored).is_err());
        assert!(matches!(
            validate_mainnet_sns_reward_neuron_page(&fixture_reward_page(
                restored.rows.clone(),
                None,
            )),
            Err(SnsHostError::InvalidSourceData {
                capability: "SNS reward checkpoint",
                ..
            })
        ));
        let row = &restored.rows[0];
        let mut neuron = fixture_sns_neuron();
        neuron.detail.permissions = row.permissions.clone();
        neuron.detail.disburse_maturity_in_progress = row.disburse_maturity_in_progress.clone();
        let (mint, staking) = neuron.detail.derived_policy_observations();
        neuron.detail.maturity_mint_conversion_observed_disabled = mint;
        neuron.detail.manual_maturity_staking_observed_disabled = staking;
        assert!(matches!(
            validate_mainnet_sns_neuron(&neuron, &neuron.detail.neuron.neuron_id),
            Err(SnsHostError::InvalidSourceData {
                capability: "SNS neuron detail",
                ..
            })
        ));
    }
}

#[test]
fn checkpoint_retains_unassessable_permission_evidence_while_detail_requires_principals() {
    let mut row = fixture_reward_row(1);
    row.permissions[0].permission_types = [0, 11, -1]
        .map(SnsNeuronPermissionValue::from_code)
        .to_vec();
    row.permissions[0].principal = None;
    let (mint, staking) = row.derived_policy_observations();
    row.maturity_mint_conversion_observed_disabled = mint;
    row.manual_maturity_staking_observed_disabled = staking;
    let report = build_sns_reward_checkpoint_report_with_source(
        &reward_checkpoint_request("1"),
        &FixtureSnsRewardSource::new(vec![fixture_reward_page(vec![row], None)]),
    )
    .expect("checkpoint preserves incomplete and unknown evidence");
    validate_sns_reward_checkpoint_report(&report).expect("restored evidence remains valid");
    assert_eq!(
        report.maturity_conversion_policy_observed_status,
        SnsPolicyObservationStatus::Unassessable,
    );
    let mut neuron = fixture_sns_neuron();
    neuron.detail.permissions = report.rows[0].permissions.clone();
    assert!(matches!(
        validate_mainnet_sns_neuron(&neuron, &neuron.detail.neuron.neuron_id),
        Err(SnsHostError::InvalidSourceData { reason, .. })
            if reason.contains("principal is missing")
    ));
    neuron.detail.permissions[0].principal = Some(ROOT_A.to_string());
    let (mint, staking) = neuron.detail.derived_policy_observations();
    neuron.detail.maturity_mint_conversion_observed_disabled = mint;
    neuron.detail.manual_maturity_staking_observed_disabled = staking;
    validate_mainnet_sns_neuron(&neuron, &neuron.detail.neuron.neuron_id)
        .expect("detail preserves unknown permission codes with canonical labels");
}

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
    assert_eq!(source.calls(), Vec::<&str>::new());

    let source = FixtureSnsRewardSource::new(vec![fixture_reward_page(Vec::new(), None)]);
    let request = reward_checkpoint_request("1").with_max_pages(Some(0));
    assert!(matches!(
        build_sns_reward_checkpoint_report_with_source(&request, &source),
        Err(SnsHostError::InvalidRewardCheckpointPageCap { max_pages: 0 })
    ));
    assert_eq!(source.calls(), Vec::<&str>::new());
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
    assert!(!state.exhausted());
    state
        .ingest_page(fixture_reward_page(full_rows.clone(), Some(final_cursor)))
        .expect("first full page");
    assert!(!state.exhausted());
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
    assert_eq!(state.page_count(), 1);
    assert_eq!(state.row_count(), 100);
    assert_eq!(
        state.next_cursor().expect("full-page cursor").id,
        vec![100; 32]
    );
    assert!(!state.exhausted());
    state
        .ingest_page(fixture_reward_page(Vec::new(), None))
        .expect("empty terminal page after a rejected page");
    assert_eq!(state.page_count(), 2);
    assert_eq!(state.row_count(), 100);
    assert!(state.next_cursor().is_none());
    assert!(state.exhausted());

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
    for rows in [Vec::new(), vec![fixture_reward_row(1)]] {
        let row_count = rows.len();
        let mut state = SnsRewardCollectionState::new();
        assert!(!state.exhausted());
        state
            .ingest_page(fixture_reward_page(rows, None))
            .expect("short or empty first page exhausts the API");
        assert!(state.exhausted());
        assert!(matches!(
            state.ingest_page(fixture_reward_page(Vec::new(), None)),
            Err(SnsHostError::InvalidSourceData { reason, .. })
                if reason.contains("after reported API exhaustion")
        ));
        assert_eq!(state.page_count(), 1);
        assert_eq!(state.row_count(), row_count);
        assert!(state.exhausted());
    }
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
