#[cfg(feature = "host")]
use ic_query::HostCacheError;
use ic_query::icrc::IcrcMetadataValueKind;
use ic_query::report::ReportDataSource;
use ic_query::sns::{
    DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, MAX_SNS_METRICS_TIME_WINDOW_SECONDS,
    SnsCanisterCallType, SnsCanisterCycleBalanceStatus, SnsCanisterGapKind,
    SnsCanisterHealthQueryGap, SnsCanisterMethod, SnsCanisterReport, SnsCanisterRole,
    SnsCanisterRow, SnsCanisterStatus, SnsCustomProposalCriticality, SnsGovernanceParameters,
    SnsInfoReport, SnsListReport, SnsListRequest, SnsListSort, SnsLookupRequest,
    SnsMaturityDisbursementRow, SnsMetricsReport, SnsMetricsRequest, SnsNeuronAccount,
    SnsNeuronDetail, SnsNeuronDetailReport, SnsNeuronDissolveState, SnsNeuronFolloweeRow,
    SnsNeuronFolloweesRow, SnsNeuronPermissionList, SnsNeuronPermissionRow,
    SnsNeuronPermissionValue, SnsNeuronRow, SnsNeuronTopicFolloweesRow, SnsParamsReport,
    SnsPendingUpgrade, SnsPolicyObservationStatus, SnsProposalAction, SnsProposalBallotRow,
    SnsProposalDecisionState, SnsProposalEligibilityFilter, SnsProposalFailureReason,
    SnsProposalReport, SnsProposalRequest, SnsProposalRow, SnsProposalSortDirection,
    SnsProposalStatusFilter, SnsProposalTally, SnsProposalTopicFilter, SnsProposalVote,
    SnsProposalsReport, SnsProposalsRequest, SnsProposalsSort, SnsRewardAllocationStatus,
    SnsRewardCheckpointReport, SnsRewardCheckpointRow, SnsRewardCollectionStatus,
    SnsRewardDiffInvalidReasonKind, SnsRewardDiffReport, SnsRewardEvent, SnsRewardProposalId,
    SnsRunningVersionResponse, SnsSwapComponent, SnsSwapDerivedState, SnsSwapLifecycle,
    SnsSwapNeuronBasketConstructionParameters, SnsSwapQueryGap, SnsSwapReport,
    SnsSwapSaleParameters, SnsTokenMetadataRow, SnsTokenReport, SnsTokenStandardRow,
    SnsTreasuryKind, SnsTreasuryMetricRow, SnsUpgradeQueryGap, SnsUpgradeReport, SnsVersion,
    SnsVotingPowerMetrics, SnsVotingRewardsParameters, build_sns_reward_diff_report,
    sns_canister_report_text, sns_info_report_text, sns_list_report_text, sns_metrics_report_text,
    sns_neuron_detail_report_text, sns_neuron_permission_name, sns_params_report_text,
    sns_proposal_report_text, sns_proposals_report_text, sns_reward_checkpoint_report_text,
    sns_reward_diff_report_text, sns_swap_report_text, sns_token_report_text,
    sns_upgrade_report_text, validate_sns_reward_checkpoint_report,
};
#[cfg(feature = "host")]
use ic_query::sns::{
    DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS,
    DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, DEFAULT_SNS_SOURCE_ENDPOINT, LiveSnsSource,
    MainnetSns, MainnetSnsCanisterInventory, MainnetSnsCanisters, MainnetSnsInventory,
    MainnetSnsLifecycle, MainnetSnsMetadata, MainnetSnsMetrics, MainnetSnsNeuron,
    MainnetSnsNeuronPage, MainnetSnsNeurons, MainnetSnsProposal, MainnetSnsProposalPage,
    MainnetSnsProposals, MainnetSnsRewardNeuronPage, MainnetSnsSwap, MainnetSnsToken,
    MainnetSnsUpgrade, SnsCacheListRequest, SnsCacheStatusRequest, SnsCanisterSource,
    SnsCatalogCacheRequest, SnsCatalogRefreshReport, SnsCatalogRefreshRequest, SnsCatalogSource,
    SnsDiscoverySource, SnsHostError, SnsMetricsSource, SnsNeuronId, SnsNeuronRequest,
    SnsNeuronSource, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest, SnsNeuronsReport,
    SnsNeuronsRequest, SnsNeuronsSort, SnsNeuronsSource, SnsParamsSource, SnsProposalSource,
    SnsProposalsRefreshReport, SnsProposalsRefreshRequest, SnsProposalsSource,
    SnsRewardCheckpointRequest, SnsRewardSource, SnsSourceRequest, SnsSwapSource, SnsTokenSource,
    SnsUpgradeSource, build_sns_canister_report, build_sns_canister_report_with_source,
    build_sns_info_report, build_sns_info_report_with_source, build_sns_list_report,
    build_sns_list_report_from_cache, build_sns_list_report_from_cache_or_refresh,
    build_sns_list_report_with_source, build_sns_metrics_report,
    build_sns_metrics_report_with_source, build_sns_neuron_detail_report,
    build_sns_neuron_detail_report_with_source, build_sns_neurons_cache_list_report,
    build_sns_neurons_cache_status_report, build_sns_neurons_report,
    build_sns_neurons_report_with_source, build_sns_params_report,
    build_sns_params_report_with_source, build_sns_proposal_report,
    build_sns_proposal_report_with_source, build_sns_proposals_cache_list_report,
    build_sns_proposals_cache_status_report, build_sns_proposals_report,
    build_sns_proposals_report_with_source, build_sns_reward_checkpoint_report,
    build_sns_reward_checkpoint_report_with_source, build_sns_reward_diff_report_from_paths,
    build_sns_swap_report, build_sns_swap_report_with_source, build_sns_token_report,
    build_sns_token_report_with_source, build_sns_upgrade_report,
    build_sns_upgrade_report_with_source, load_sns_reward_checkpoint, refresh_sns_catalog,
    refresh_sns_neurons_cache, refresh_sns_neurons_cache_with_source, refresh_sns_proposals_cache,
    refresh_sns_proposals_cache_with_source, sns_catalog_cache_path, sns_catalog_refresh_lock_path,
    sns_catalog_refresh_report_text, sns_neurons_cache_list_report_text, sns_neurons_cache_path,
    sns_neurons_cache_status_report_text, sns_neurons_refresh_attempt_path,
    sns_neurons_refresh_lock_path, sns_neurons_refresh_report_text, sns_neurons_report_text,
    sns_proposals_cache_list_report_text, sns_proposals_cache_path,
    sns_proposals_cache_status_report_text, sns_proposals_refresh_attempt_path,
    sns_proposals_refresh_lock_path, sns_proposals_refresh_report_text,
};
use serde_json::json;
#[cfg(feature = "host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

const SAMPLE_SNS_ROOT_CANISTER_ID: &str = "be2us-64aaa-aaaaa-qaabq-cai";
const SAMPLE_SNS_GOVERNANCE_CANISTER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";
#[cfg(feature = "host")]
const SAMPLE_SNS_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
#[cfg(feature = "host")]
const SAMPLE_SNS_SWAP_CANISTER_ID: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
#[cfg(feature = "host")]
const SAMPLE_SNS_INDEX_CANISTER_ID: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const SAMPLE_SNS_FETCHED_AT: &str = "2023-11-14T22:13:20Z";
const SAMPLE_SNS_NEURON_ID: &str =
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[cfg(feature = "host")]
type SnsListBuilder = fn(&SnsListRequest) -> Result<SnsListReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsInfoBuilder = fn(&SnsLookupRequest) -> Result<SnsInfoReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsMetricsBuilder = fn(&SnsMetricsRequest) -> Result<SnsMetricsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsCanisterBuilder = fn(&SnsLookupRequest) -> Result<SnsCanisterReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsTokenBuilder = fn(&SnsLookupRequest) -> Result<SnsTokenReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsParamsBuilder = fn(&SnsLookupRequest) -> Result<SnsParamsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsSwapBuilder = fn(&SnsLookupRequest) -> Result<SnsSwapReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsUpgradeBuilder = fn(&SnsLookupRequest) -> Result<SnsUpgradeReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsProposalsBuilder = fn(&SnsProposalsRequest) -> Result<SnsProposalsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsProposalBuilder = fn(&SnsProposalRequest) -> Result<SnsProposalReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsNeuronBuilder = fn(&SnsNeuronRequest) -> Result<SnsNeuronDetailReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsRewardCheckpointBuilder =
    fn(&SnsRewardCheckpointRequest) -> Result<SnsRewardCheckpointReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsNeuronsBuilder = fn(&SnsNeuronsRequest) -> Result<SnsNeuronsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsNeuronsRefreshBuilder =
    fn(&SnsNeuronsRefreshRequest) -> Result<SnsNeuronsRefreshReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsProposalsRefreshBuilder =
    fn(&SnsProposalsRefreshRequest) -> Result<SnsProposalsRefreshReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsCatalogRefreshBuilder =
    fn(&SnsCatalogRefreshRequest) -> Result<SnsCatalogRefreshReport, SnsHostError>;

#[test]
fn public_sns_request_constructors_set_expected_fields() {
    let list = SnsListRequest::new("ic", "https://icp-api.io", 1_700_000_000)
        .with_verbose(true)
        .with_sort(SnsListSort::Name);
    assert_eq!(list.network, "ic");
    assert_eq!(list.sort, SnsListSort::Name);
    assert!(list.verbose);

    let lookup = SnsLookupRequest::new("ic", "https://icp-api.io", 1_700_000_000, "1");
    assert_eq!(lookup.input, "1");
    let token = SnsLookupRequest::new(
        "ic",
        "https://icp-api.io",
        1_700_000_000,
        "be2us-64aaa-aaaaa-qaabq-cai",
    );
    assert_eq!(token.source_endpoint, "https://icp-api.io");

    let metrics = SnsMetricsRequest::new("ic", "https://icp-api.io", 1_700_000_000, "1")
        .with_time_window_seconds(86_400);
    assert_eq!(metrics.time_window_seconds, 86_400);
    assert_eq!(DEFAULT_SNS_METRICS_TIME_WINDOW_SECONDS, 30 * 86_400);
    assert_eq!(MAX_SNS_METRICS_TIME_WINDOW_SECONDS, 365 * 86_400);

    let proposal = SnsProposalRequest::new("ic", "https://icp-api.io", 1_700_000_000, "1", 42)
        .with_verbose(true)
        .with_show_ballots(true);
    assert_eq!(proposal.proposal_id, 42);
    assert!(proposal.verbose);
    assert!(proposal.show_ballots);
    assert!(proposal.cache_root.is_none());

    let proposals = SnsProposalsRequest::new("ic", "https://icp-api.io", 1_700_000_000, "1", 25)
        .with_before_proposal_id(100)
        .with_status(SnsProposalStatusFilter::Open)
        .with_topic(SnsProposalTopicFilter::Governance)
        .with_eligibility(SnsProposalEligibilityFilter::Yes)
        .with_proposer_neuron_id("010203")
        .with_query("upgrade")
        .with_sort(SnsProposalsSort::Created)
        .with_sort_direction(SnsProposalSortDirection::Asc)
        .with_verbose(true);
    assert_eq!(proposals.limit, 25);
    assert_eq!(proposals.before_proposal_id, Some(100));
    assert_eq!(proposals.status, SnsProposalStatusFilter::Open);
    assert_eq!(proposals.topic, SnsProposalTopicFilter::Governance);
    assert_eq!(proposals.eligibility, SnsProposalEligibilityFilter::Yes);
    assert_eq!(proposals.proposer_neuron_id.as_deref(), Some("010203"));
    assert_eq!(proposals.query.as_deref(), Some("upgrade"));
    assert_eq!(proposals.sort, SnsProposalsSort::Created);
    assert_eq!(proposals.sort_direction, SnsProposalSortDirection::Asc);
    assert!(proposals.verbose);
}

#[test]
fn public_sns_neuron_detail_models_are_constructible_and_renderable() {
    let report = SnsNeuronDetailReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
        data_source: ReportDataSource::Live,
        detail: sample_sns_neuron_detail(),
    };

    assert_eq!(sns_neuron_permission_name(7), "merge_maturity");
    assert_eq!(sns_neuron_permission_name(99), "unknown");
    assert_eq!(
        report.detail.maturity_mint_conversion_observed_disabled,
        SnsPolicyObservationStatus::Violated
    );
    assert!(sns_neuron_detail_report_text(&report).contains("7:merge_maturity"));
    assert_eq!(
        serde_json::to_value(&report).expect("detail JSON")["neuron_id"],
        SAMPLE_SNS_NEURON_ID
    );
}

#[test]
fn public_sns_reward_checkpoint_models_round_trip_and_render_without_host() {
    let report = sample_sns_reward_checkpoint_report();
    let value = serde_json::to_value(&report).expect("checkpoint JSON");
    let decoded: SnsRewardCheckpointReport =
        serde_json::from_value(value).expect("strict checkpoint decode");

    validate_sns_reward_checkpoint_report(&decoded).expect("pure checkpoint validation");
    let mut cached = decoded.clone();
    cached.data_source = ReportDataSource::Cache;
    assert!(validate_sns_reward_checkpoint_report(&cached).is_err());
    assert_eq!(
        decoded.rows[0].checked_combined_maturity(),
        Some(15_000_000)
    );
    assert_eq!(
        decoded.rows[0].derived_policy_observations(),
        (
            SnsPolicyObservationStatus::ObservedSatisfied,
            SnsPolicyObservationStatus::ObservedSatisfied,
        )
    );
    assert!(
        sns_reward_checkpoint_report_text(&decoded)
            .contains("collection_status: api_exhausted_observed")
    );
}

#[test]
fn public_sns_reward_diff_reconciles_one_immediate_native_distribution() {
    let before = sample_sns_reward_checkpoint_report();
    let after = advance_reward_checkpoint(&before, 5_000, 5_000);

    let report = build_sns_reward_diff_report(&before, &after);
    assert_eq!(report.allocation_status, SnsRewardAllocationStatus::Valid);
    assert!(report.invalid_reasons.is_empty());
    assert_eq!(report.aggregate_maturity_delta_e8s_equivalent, Some(5_000));
    assert_eq!(
        report.summed_neuron_maturity_delta_e8s_equivalent,
        Some(5_000)
    );
    assert_eq!(
        report.rows[0].allocation_numerator_e8s_equivalent,
        Some(5_000)
    );
    assert_eq!(
        report.rows[0].allocation_denominator_e8s_equivalent,
        Some(5_000)
    );
    assert!(!report.checkpoint_content_authenticated);

    let value = serde_json::to_value(&report).expect("reward diff JSON");
    let decoded: SnsRewardDiffReport =
        serde_json::from_value(value).expect("strict reward diff decode");
    assert!(sns_reward_diff_report_text(&decoded).contains("allocation_status: valid"));
}

#[test]
fn public_sns_reward_diff_rejects_hidden_conversion_and_skipped_event() {
    let before = sample_sns_reward_checkpoint_report();
    let hidden_conversion = advance_reward_checkpoint(&before, 10, 50);
    let report = build_sns_reward_diff_report(&before, &hidden_conversion);
    assert_eq!(report.allocation_status, SnsRewardAllocationStatus::Invalid);
    assert!(
        report.invalid_reasons.iter().any(|reason| {
            reason.kind == SnsRewardDiffInvalidReasonKind::AggregateReconciliation
        })
    );

    let mut skipped_event = advance_reward_checkpoint(&before, 50, 50);
    skipped_event.reward_event_before.round += 1;
    skipped_event.reward_event_after.round += 1;
    let report = build_sns_reward_diff_report(&before, &skipped_event);
    assert!(
        report
            .invalid_reasons
            .iter()
            .any(|reason| { reason.kind == SnsRewardDiffInvalidReasonKind::RewardEventContinuity })
    );

    let mut uncovered_event = advance_reward_checkpoint(&before, 50, 50);
    uncovered_event.reward_event_before.actual_timestamp_seconds =
        before.collection_completed_at_unix_secs;
    uncovered_event.reward_event_after.actual_timestamp_seconds =
        before.collection_completed_at_unix_secs;
    let report = build_sns_reward_diff_report(&before, &uncovered_event);
    assert!(
        report
            .invalid_reasons
            .iter()
            .any(|reason| { reason.kind == SnsRewardDiffInvalidReasonKind::RewardEventCoverage })
    );
}

#[test]
fn public_sns_reward_diff_returns_no_allocation_for_exact_zero_distribution() {
    let before = sample_sns_reward_checkpoint_report();
    let after = advance_reward_checkpoint(&before, 0, 0);

    let report = build_sns_reward_diff_report(&before, &after);
    assert_eq!(
        report.allocation_status,
        SnsRewardAllocationStatus::NoAllocation
    );
    assert!(report.invalid_reasons.is_empty());
    assert_eq!(report.rows[0].allocation_denominator_e8s_equivalent, None);
}

#[test]
fn public_sns_reward_diff_fails_closed_for_unknown_permissions_and_tampering() {
    let before = sample_sns_reward_checkpoint_report();
    let mut unknown = advance_reward_checkpoint(&before, 5_000, 5_000);
    unknown.rows[0].permissions[0].permission_types = vec![SnsNeuronPermissionValue::from_code(11)];
    unknown.rows[0].maturity_mint_conversion_observed_disabled =
        SnsPolicyObservationStatus::Unassessable;
    unknown.rows[0].manual_maturity_staking_observed_disabled =
        SnsPolicyObservationStatus::Unassessable;
    unknown.unassessable_permission_code_count = 1;
    unknown.maturity_mint_conversion_observed_disabled = SnsPolicyObservationStatus::Unassessable;
    unknown.manual_maturity_staking_observed_disabled = SnsPolicyObservationStatus::Unassessable;
    unknown.maturity_conversion_policy_observed_status = SnsPolicyObservationStatus::Unassessable;
    let report = build_sns_reward_diff_report(&before, &unknown);
    assert!(report.invalid_reasons.iter().any(|reason| {
        reason.kind == SnsRewardDiffInvalidReasonKind::PolicyNotObservedSatisfied
    }));

    let mut tampered = advance_reward_checkpoint(&before, 5_000, 5_000);
    tampered.aggregate_combined_maturity_e8s_equivalent += 1;
    let report = build_sns_reward_diff_report(&before, &tampered);
    assert!(
        report.invalid_reasons.iter().any(|reason| {
            reason.kind == SnsRewardDiffInvalidReasonKind::AfterCheckpointInvalid
        })
    );
    assert_eq!(report.allocation_status, SnsRewardAllocationStatus::Invalid);
}

#[test]
fn public_sns_reward_diff_preserves_negative_missing_and_new_neuron_evidence() {
    let before = sample_sns_reward_checkpoint_report();
    let mut decreased = advance_reward_checkpoint(&before, 0, 50);
    decreased.rows[0].maturity_e8s_equivalent -= 40;
    decreased.rows[0].combined_maturity_e8s_equivalent -= 40;
    decreased.aggregate_maturity_e8s_equivalent -= 40;
    decreased.aggregate_combined_maturity_e8s_equivalent -= 40;
    let report = build_sns_reward_diff_report(&before, &decreased);
    assert_eq!(report.rows[0].maturity_delta_e8s_equivalent, -40);
    assert!(
        report
            .invalid_reasons
            .iter()
            .any(|reason| { reason.kind == SnsRewardDiffInvalidReasonKind::NegativeMaturityDelta })
    );

    let mut missing = advance_reward_checkpoint(&before, 0, 0);
    clear_reward_checkpoint_rows(&mut missing);
    let report = build_sns_reward_diff_report(&before, &missing);
    assert!(report.rows[0].missing_after);
    assert!(
        report
            .invalid_reasons
            .iter()
            .any(|reason| { reason.kind == SnsRewardDiffInvalidReasonKind::NeuronMissingAfter })
    );

    let mut empty_before = before.clone();
    clear_reward_checkpoint_rows(&mut empty_before);
    let mut new_after = advance_reward_checkpoint(&before, 0, 15_000_000);
    new_after.rows[0].created_timestamp_seconds = before.collection_completed_at_unix_secs + 1;
    let report = build_sns_reward_diff_report(&empty_before, &new_after);
    assert_eq!(report.allocation_status, SnsRewardAllocationStatus::Valid);
    assert!(report.rows[0].new_neuron);

    new_after.rows[0].created_timestamp_seconds = before.collection_completed_at_unix_secs;
    let report = build_sns_reward_diff_report(&empty_before, &new_after);
    assert!(report.invalid_reasons.iter().any(|reason| {
        reason.kind == SnsRewardDiffInvalidReasonKind::NewNeuronCreationUnexplained
    }));
}

#[test]
fn public_sns_reward_diff_matches_stable_canister_identity_not_display_metadata() {
    let before = sample_sns_reward_checkpoint_report();
    let mut after = advance_reward_checkpoint(&before, 5_000, 5_000);
    after.id += 10;
    after.name = "Renamed SNS".to_string();
    assert_eq!(
        build_sns_reward_diff_report(&before, &after).allocation_status,
        SnsRewardAllocationStatus::Valid
    );

    after.ledger_canister_id = "rkp4c-7iaaa-aaaaa-aaaca-cai".to_string();
    let report = build_sns_reward_diff_report(&before, &after);
    assert!(
        report
            .invalid_reasons
            .iter()
            .any(|reason| { reason.kind == SnsRewardDiffInvalidReasonKind::TargetMismatch })
    );
}

#[cfg(feature = "host")]
#[test]
fn public_sns_reward_diff_file_adapter_loads_without_live_source_calls() {
    let before = sample_sns_reward_checkpoint_report();
    let after = advance_reward_checkpoint(&before, 5_000, 5_000);
    let root = PathBuf::from(format!(
        "target/ic-query-sns-public-api-reward-diff-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).expect("reward diff test directory");
    let before_path = root.join("before.json");
    let after_path = root.join("after.json");
    std::fs::write(
        &before_path,
        serde_json::to_vec(&before).expect("before checkpoint JSON"),
    )
    .expect("write before checkpoint");
    std::fs::write(
        &after_path,
        serde_json::to_vec(&after).expect("after checkpoint JSON"),
    )
    .expect("write after checkpoint");

    assert_eq!(
        load_sns_reward_checkpoint(&before_path).expect("load checkpoint"),
        before
    );
    let report = build_sns_reward_diff_report_from_paths(&before_path, &after_path)
        .expect("local reward diff");
    assert_eq!(report.allocation_status, SnsRewardAllocationStatus::Valid);
    std::fs::write(&after_path, b"{\"schema_version\":1}")
        .expect("write malformed checkpoint shape");
    assert!(matches!(
        load_sns_reward_checkpoint(&after_path),
        Err(SnsHostError::ParseRewardCheckpoint { path, .. }) if path == after_path
    ));
    std::fs::remove_dir_all(root).expect("remove reward diff test directory");
}

#[test]
fn public_sns_list_api_is_constructible_and_renderable() {
    let request = SnsListRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        all_lifecycles: false,
        verbose: false,
        sort: SnsListSort::Id,
    };

    assert_eq!(request.sort.as_str(), "id");

    let report = SnsListReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: ReportDataSource::Live,
        cache_path: None,
        cache_complete: None,
        all_lifecycles: request.all_lifecycles,
        verbose: request.verbose,
        sort: request.sort.as_str().to_string(),
        catalog_sns_count: 0,
        excluded_sns_count: 0,
        sns_count: 0,
        metadata_error_count: 0,
        lifecycle_error_count: 0,
        sns_instances: Vec::new(),
    };

    let text = sns_list_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("sns_count: 0"));
}

#[test]
fn public_sns_info_api_is_constructible_and_renderable() {
    let request = SnsLookupRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
    };

    let report = SnsInfoReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        description: Some("Example description".to_string()),
        url: None,
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        swap_canister_id: "br5f7-7uaaa-aaaaa-qaaca-cai".to_string(),
        index_canister_id: "bw4dl-smaaa-aaaaa-qaacq-cai".to_string(),
        metadata_error: None,
    };

    assert_eq!(request.input, "1");

    let text = sns_info_report_text(&report);

    assert!(text.contains("sns_id: 1"));
    assert!(text.contains("description: Example description"));
    assert!(text.contains("url: -"));
}

#[test]
fn public_sns_metrics_api_is_constructible_and_renderable() {
    let report = SnsMetricsReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        method: SnsCanisterMethod::GetMetrics,
        call_type: SnsCanisterCallType::CompositeQuery,
        time_window_seconds: 86_400,
        point_in_time_guaranteed: false,
        treasury_metrics_cached: true,
        num_recently_submitted_proposals: Some(3),
        num_recently_executed_proposals: Some(2),
        last_ledger_block_timestamp: Some(1_700_000_010),
        genesis_timestamp_seconds: Some(1_600_000_000),
        treasury_metric_count: 1,
        treasury_metrics: vec![SnsTreasuryMetricRow {
            treasury: 1,
            treasury_kind: SnsTreasuryKind::Icp,
            name: Some("ICP treasury".to_string()),
            ledger_canister_id: None,
            account_owner: None,
            account_subaccount_hex: None,
            amount_e8s: Some(100_000_000),
            original_amount_e8s: Some(200_000_000),
            timestamp_seconds: Some(1_700_000_000),
        }],
        voting_power_metrics: Some(SnsVotingPowerMetrics {
            governance_total_potential_voting_power: Some(500_000_000),
            timestamp_seconds: Some(1_700_000_001),
        }),
    };

    let json = serde_json::to_value(&report).expect("serialize public SNS metrics report");
    let text = sns_metrics_report_text(&report);

    assert_eq!(json["treasury_metrics"][0]["treasury"], 1);
    assert_eq!(json["treasury_metrics"][0]["treasury_kind"], "icp");
    assert_eq!(json["method"], "get_metrics");
    assert_eq!(json["call_type"], "composite_query");
    assert!(text.contains("treasury_metrics_cached: yes"));
    assert!(text.contains("ICP treasury"));
}

#[test]
fn public_sns_canister_api_is_constructible_and_renderable() {
    let health_query_gap = SnsCanisterHealthQueryGap {
        method: SnsCanisterMethod::GetSnsCanistersSummary,
        reason: "health unavailable".to_string(),
    };
    let report = SnsCanisterReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        inventory_method: SnsCanisterMethod::ListSnsCanisters,
        health_method: SnsCanisterMethod::GetSnsCanistersSummary,
        health_call_type: SnsCanisterCallType::IngressUpdate,
        health_update_canister_list: false,
        point_in_time_guaranteed: false,
        canister_count: 1,
        health_status_count: 1,
        reported_zero_cycles_count: 0,
        cycles_unavailable_count: 0,
        gap_count: 0,
        health_query_gap: None,
        canisters: vec![SnsCanisterRow {
            role: SnsCanisterRole::Root,
            canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
            status: Some(SnsCanisterStatus::Running),
            module_hash_hex: Some("01020304".to_string()),
            cycles: Some("1000000".to_string()),
            cycle_balance_status: SnsCanisterCycleBalanceStatus::ReportedNonzero,
            memory_size: Some("2000000".to_string()),
            idle_cycles_burned_per_day: Some("3000".to_string()),
            controllers: vec![SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string()],
        }],
        gaps: Vec::new(),
    };

    let json = serde_json::to_value(&report).expect("serialize public SNS canister report");
    let text = sns_canister_report_text(&report);

    assert_eq!(SnsCanisterRole::Governance.as_str(), "governance");
    assert_eq!(SnsCanisterStatus::Stopped.as_str(), "stopped");
    assert_eq!(
        SnsCanisterCycleBalanceStatus::ReportedZero.as_str(),
        "reported_zero"
    );
    assert_eq!(
        SnsCanisterCallType::IngressUpdate.as_str(),
        "ingress_update"
    );
    assert_eq!(
        SnsCanisterGapKind::SummaryMissing.as_str(),
        "summary_missing"
    );
    assert_eq!(json["inventory_method"], "list_sns_canisters");
    assert_eq!(json["health_method"], "get_sns_canisters_summary");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["reported_zero_cycles_count"], 0);
    assert_eq!(json["cycles_unavailable_count"], 0);
    assert!(json["health_query_gap"].is_null());
    assert_eq!(
        json["canisters"][0]["cycle_balance_status"],
        "reported_nonzero"
    );
    assert_eq!(json["canisters"][0]["cycles"], "1000000");
    assert_eq!(json["canisters"][0]["memory_size"], "2000000");
    assert_eq!(json["canisters"][0]["idle_cycles_burned_per_day"], "3000");
    assert!(text.contains("health_call_type: ingress_update"));
    assert!(text.contains("running"));
    assert!(text.contains("1 M"));
    assert!(text.contains("1.91 MiB"));
    assert_eq!(health_query_gap.reason, "health unavailable");
}

#[test]
fn public_sns_token_api_is_constructible_and_renderable() {
    let request = SnsLookupRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
    };

    let report = SnsTokenReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: request.input,
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        sns_index_canister_id: "bw4dl-smaaa-aaaaa-qaacq-cai".to_string(),
        token_name: "Example Token".to_string(),
        token_symbol: "EXT".to_string(),
        decimals: 8,
        transfer_fee: "100_000_000".to_string(),
        total_supply: "1_000_000_000".to_string(),
        minting_account_owner: Some("aaaaa-aa".to_string()),
        minting_account_subaccount_hex: None,
        ledger_index_canister_id: None,
        ledger_index_error: Some("not configured".to_string()),
        supported_standards: vec![SnsTokenStandardRow {
            name: "ICRC-1".to_string(),
            url: "https://github.com/dfinity/ICRC-1".to_string(),
        }],
        metadata: vec![SnsTokenMetadataRow {
            key: "icrc1:symbol".to_string(),
            value_type: IcrcMetadataValueKind::Text,
            value: json!("EXT"),
        }],
    };

    let text = sns_token_report_text(&report);

    assert!(text.contains("token_symbol: EXT"));
    assert!(text.contains("transfer_fee: 1.00"));
    assert!(text.contains("ledger_index_error: not configured"));
    assert!(text.contains("ICRC-1"));
}

#[test]
fn public_sns_params_api_is_constructible_and_renderable() {
    let request = SnsLookupRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
    };

    let report = SnsParamsReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
        parameters: sample_sns_governance_parameters(),
    };

    let text = sns_params_report_text(&report);

    assert!(text.contains("sns_id: 1"));
    assert!(text.contains("neuron_minimum_stake"));
    assert!(text.contains("1.00"));
    assert!(text.contains("max_dissolve_delay"));
    assert!(text.contains("2h"));
    assert!(text.contains("maturity_modulation_disabled"));
    assert!(text.contains("no"));
    assert!(text.contains("additional_critical_native_actions"));
    assert!(text.contains("7,8"));
}

#[test]
fn public_sns_swap_api_is_constructible_serializable_and_renderable() {
    let report = SnsSwapReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        swap_canister_id: "br5f7-7uaaa-aaaaa-qaaca-cai".to_string(),
        lifecycle_method: SnsCanisterMethod::GetLifecycle,
        sale_parameters_method: SnsCanisterMethod::GetSaleParameters,
        derived_state_method: SnsCanisterMethod::GetDerivedState,
        point_in_time_guaranteed: false,
        component_query_count: 3,
        successful_component_query_count: 3,
        component_gap_count: 0,
        lifecycle: Some(SnsSwapLifecycle {
            lifecycle: Some(2),
            lifecycle_name: Some("open".to_string()),
            decentralization_sale_open_timestamp_seconds: Some(1_700_000_000),
            decentralization_swap_termination_timestamp_seconds: None,
        }),
        sale_parameters: Some(SnsSwapSaleParameters {
            min_icp_e8s: 100_000_000,
            max_icp_e8s: 100_000_000_000,
            min_direct_participation_icp_e8s: Some(1_000_000_000),
            max_direct_participation_icp_e8s: Some(90_000_000_000),
            sns_token_e8s: 250_000_000_000,
            min_participants: 25,
            min_participant_icp_e8s: 100_000_000,
            max_participant_icp_e8s: 10_000_000_000,
            swap_due_timestamp_seconds: 1_700_086_400,
            sale_delay_seconds: Some(3_600),
            neuron_basket_construction_parameters: Some(
                SnsSwapNeuronBasketConstructionParameters {
                    count: 5,
                    dissolve_delay_interval_seconds: 2_592_000,
                },
            ),
        }),
        derived_state: Some(SnsSwapDerivedState {
            sns_tokens_per_icp: Some(2.5),
            buyer_total_icp_e8s: Some(1_000_000_000),
            direct_participation_icp_e8s: Some(900_000_000),
            neurons_fund_participation_icp_e8s: Some(100_000_000),
            direct_participant_count: Some(10),
            cf_participant_count: None,
            cf_neuron_count: None,
        }),
        gaps: Vec::new(),
    };
    let gap = SnsSwapQueryGap {
        component: SnsSwapComponent::DerivedState,
        method: SnsCanisterMethod::GetDerivedState,
        reason: "fixture rejection".to_string(),
    };

    let text = sns_swap_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize swap report");

    assert!(text.contains("lifecycle_name"));
    assert_eq!(json["lifecycle_method"], "get_lifecycle");
    assert_eq!(json["sale_parameters_method"], "get_sale_parameters");
    assert_eq!(json["derived_state_method"], "get_derived_state");
    assert_eq!(json["lifecycle"]["lifecycle"], 2);
    assert_eq!(
        serde_json::to_value(gap).expect("serialize swap gap")["component"],
        "derived_state"
    );
}

#[test]
fn public_sns_upgrade_api_is_constructible_serializable_and_renderable() {
    let deployed_version = sample_sns_version("01");
    let next_version = sample_sns_version("02");
    let report = SnsUpgradeReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        running_version_method: SnsCanisterMethod::GetRunningSnsVersion,
        next_version_method: SnsCanisterMethod::GetNextSnsVersion,
        point_in_time_guaranteed: false,
        component_query_count: 2,
        successful_component_query_count: 2,
        component_gap_count: 0,
        deployed_version,
        pending_upgrade: Some(SnsPendingUpgrade {
            mark_failed_at_seconds: 1_700_086_400,
            checking_upgrade_lock: 7,
            proposal_id: 42,
            target_version: Some(next_version.clone()),
        }),
        next_version: Some(next_version),
        next_version_gap: None,
    };
    let gap = SnsUpgradeQueryGap {
        method: SnsCanisterMethod::GetNextSnsVersion,
        reason: "fixture rejection".to_string(),
    };

    let text = sns_upgrade_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize upgrade report");

    assert!(text.contains("next_version: available"));
    assert_eq!(json["running_version_method"], "get_running_sns_version");
    assert_eq!(json["next_version_method"], "get_next_sns_version");
    assert_eq!(json["deployed_version"]["root_wasm_hash_hex"], "01");
    assert_eq!(
        serde_json::to_value(gap).expect("serialize upgrade gap")["method"],
        "get_next_sns_version"
    );
}

#[test]
fn public_sns_proposals_api_is_constructible_and_renderable() {
    let request = SnsProposalsRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
        limit: 10,
        before_proposal_id: Some(100),
        status: SnsProposalStatusFilter::Any,
        topic: SnsProposalTopicFilter::Governance,
        eligibility: SnsProposalEligibilityFilter::Yes,
        proposer_neuron_id: Some("010203".to_string()),
        query: Some("upgrade".to_string()),
        sort: SnsProposalsSort::Created,
        sort_direction: SnsProposalSortDirection::Desc,
        cache_root: None,
        verbose: true,
    };

    assert_eq!(request.sort.as_str(), "created");
    assert_eq!(request.topic.as_str(), "governance");

    let report = SnsProposalsReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status.as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        eligibility_filter: request.eligibility.as_str().to_string(),
        proposer_filter: request.proposer_neuron_id,
        query_filter: request.query,
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),
        verbose: request.verbose,
        data_source: ReportDataSource::Live,
        cache_path: None,
        cache_complete: None,
        proposal_count: 1,
        proposals: vec![sample_sns_proposal_row()],
    };

    let text = sns_proposals_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize SNS proposals report");

    assert!(text.contains("proposal_count: 1"));
    assert!(text.contains("topic_filter: governance"));
    assert!(text.contains("proposal_details:"));
    assert!(text.contains("title: Upgrade SNS"));
    assert_eq!(
        json["proposals"][0]["action"],
        "upgrade_sns_to_next_version"
    );
    assert_eq!(json["proposals"][0]["ballots"][0]["vote_text"], "yes");
}

#[test]
fn public_sns_proposal_api_is_constructible_and_renderable() {
    let request = SnsProposalRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
        proposal_id: 42,
        cache_root: None,
        verbose: false,
        show_ballots: true,
    };

    let report = SnsProposalReport {
        schema_version: 1,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
        proposal_id: request.proposal_id,
        verbose: request.verbose,
        show_ballots: request.show_ballots,
        data_source: ReportDataSource::Live,
        cache_path: None,
        cache_complete: None,
        proposal: sample_sns_proposal_row(),
    };

    let text = sns_proposal_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize SNS proposal report");

    assert!(text.contains("proposal_id: 42"));
    assert!(text.contains("show_ballots: yes"));
    assert!(text.contains("ballots:"));
    assert!(text.contains("Upgrade SNS"));
    assert_eq!(json["proposal"]["action"], "upgrade_sns_to_next_version");
    assert_eq!(json["proposal"]["ballots"][0]["vote_text"], "yes");
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_exposes_live_builder_entry_points() {
    accepts_public_function::<SnsListBuilder>(build_sns_list_report);
    accepts_public_function::<SnsInfoBuilder>(build_sns_info_report);
    accepts_public_function::<SnsMetricsBuilder>(build_sns_metrics_report);
    accepts_public_function::<SnsCanisterBuilder>(build_sns_canister_report);
    accepts_public_function::<SnsTokenBuilder>(build_sns_token_report);
    accepts_public_function::<SnsParamsBuilder>(build_sns_params_report);
    accepts_public_function::<SnsSwapBuilder>(build_sns_swap_report);
    accepts_public_function::<SnsUpgradeBuilder>(build_sns_upgrade_report);
    accepts_public_function::<SnsProposalsBuilder>(build_sns_proposals_report);
    accepts_public_function::<SnsProposalBuilder>(build_sns_proposal_report);
    accepts_public_function::<SnsNeuronBuilder>(build_sns_neuron_detail_report);
    accepts_public_function::<SnsRewardCheckpointBuilder>(build_sns_reward_checkpoint_report);
    accepts_public_function::<SnsNeuronsBuilder>(build_sns_neurons_report);
    accepts_public_function::<SnsNeuronsRefreshBuilder>(refresh_sns_neurons_cache);
    accepts_public_function::<SnsProposalsRefreshBuilder>(refresh_sns_proposals_cache);
    accepts_public_function::<SnsCatalogRefreshBuilder>(refresh_sns_catalog);
    let live_source = LiveSnsSource;
    accepts_public_function(live_source);
    assert_eq!(DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS, 30 * 60);
    assert_eq!(DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, 30 * 60);
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_accepts_custom_source_adapters() -> Result<(), SnsHostError> {
    let source = FixtureSnsSource;
    let source_request = SnsSourceRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        SAMPLE_SNS_FETCHED_AT,
        "fixture",
    );
    assert_eq!(source_request.endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
    assert_eq!(source_request.network, "ic");

    let list_request = SnsListRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000);
    let list = build_sns_list_report_with_source(&list_request, &source)?;
    assert_eq!(list.sns_count, 1);
    assert_eq!(list.sns_instances[0].id, 1);

    let info_request = SnsLookupRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1");
    let info = build_sns_info_report_with_source(&info_request, &source)?;
    assert_eq!(info.root_canister_id, SAMPLE_SNS_ROOT_CANISTER_ID);

    let metrics_request =
        SnsMetricsRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1");
    let metrics = build_sns_metrics_report_with_source(&metrics_request, &source)?;
    assert_eq!(metrics.method, SnsCanisterMethod::GetMetrics);
    assert_eq!(metrics.treasury_metric_count, 1);

    let canisters = build_sns_canister_report_with_source(&info_request, &source)?;
    assert_eq!(canisters.canister_count, 1);
    assert_eq!(canisters.canisters[0].role, SnsCanisterRole::Root);

    let token_request = SnsLookupRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        SAMPLE_SNS_ROOT_CANISTER_ID,
    );
    let token = build_sns_token_report_with_source(&token_request, &source)?;
    assert_eq!(token.token_symbol, "EXT");

    let params_request =
        SnsLookupRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1");
    let params = build_sns_params_report_with_source(&params_request, &source)?;
    assert_eq!(
        params.parameters.neuron_minimum_stake_e8s,
        Some(100_000_000)
    );

    let swap = build_sns_swap_report_with_source(&params_request, &source)?;
    assert_eq!(swap.swap_canister_id, SAMPLE_SNS_SWAP_CANISTER_ID);
    assert_eq!(swap.successful_component_query_count, 3);

    let upgrade = build_sns_upgrade_report_with_source(&params_request, &source)?;
    assert_eq!(
        upgrade.governance_canister_id,
        SAMPLE_SNS_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(upgrade.successful_component_query_count, 2);

    Ok(())
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_accepts_custom_proposal_source_adapters() -> Result<(), SnsHostError> {
    let source = FixtureSnsSource;
    let detail_request =
        SnsProposalRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1", 42)
            .with_show_ballots(true);
    let detail = build_sns_proposal_report_with_source(&detail_request, &source)?;
    assert_eq!(detail.proposal.proposal_id, 42);
    assert_eq!(detail.data_source.as_str(), "live");

    let list_request =
        SnsProposalsRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1", 10)
            .with_before_proposal_id(99)
            .with_status(SnsProposalStatusFilter::Open)
            .with_topic(SnsProposalTopicFilter::Governance);
    let list = build_sns_proposals_report_with_source(&list_request, &source)?;
    assert_eq!(list.proposal_count, 1);
    assert_eq!(list.proposals[0].title, "Upgrade SNS");

    let cache_root = proposal_source_cache_root();
    let _ = fs::remove_dir_all(&cache_root);
    let refresh_request = SnsProposalsRefreshRequest::new(
        cache_root.clone(),
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        "1",
        100,
    )
    .with_max_pages(Some(1));
    let refresh = refresh_sns_proposals_cache_with_source(&refresh_request, &source)?;
    assert_eq!(refresh.proposal_count, 1);
    assert!(refresh.complete);
    let _ = fs::remove_dir_all(cache_root);

    Ok(())
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_accepts_custom_neuron_source_adapters() -> Result<(), SnsHostError> {
    let source = FixtureSnsSource;
    let detail = build_sns_neuron_detail_report_with_source(
        &SnsNeuronRequest::new(
            "ic",
            DEFAULT_SNS_SOURCE_ENDPOINT,
            1_700_000_000,
            "1",
            SAMPLE_SNS_NEURON_ID,
        ),
        &source,
    )?;
    assert_eq!(detail.neuron_id, SAMPLE_SNS_NEURON_ID);
    assert_eq!(detail.detail.permissions.len(), 1);

    let report = build_sns_neurons_report_with_source(
        &SnsNeuronsRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1", 50)
            .with_owner_principal_id(SAMPLE_SNS_GOVERNANCE_CANISTER_ID),
        &source,
    )?;
    assert_eq!(report.neuron_count, 1);
    assert_eq!(report.neurons[0].neuron_id, SAMPLE_SNS_NEURON_ID);

    let cache_root = neuron_source_cache_root();
    let _ = fs::remove_dir_all(&cache_root);
    let refresh_request = SnsNeuronsRefreshRequest::new(
        cache_root.clone(),
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        "1",
        100,
    )
    .with_max_pages(Some(1));
    let refresh = refresh_sns_neurons_cache_with_source(&refresh_request, &source)?;
    assert_eq!(refresh.neuron_count, 1);
    assert!(refresh.complete);
    let _ = fs::remove_dir_all(cache_root);

    Ok(())
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_accepts_custom_reward_source_adapter() -> Result<(), SnsHostError> {
    let report = build_sns_reward_checkpoint_report_with_source(
        &SnsRewardCheckpointRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1")
            .with_max_pages(Some(1)),
        &FixtureSnsSource,
    )?;

    assert_eq!(report.row_count, 1);
    assert_eq!(report.page_count, 1);
    assert_eq!(report.client_query_count, 9);
    assert_eq!(
        report.maturity_conversion_policy_observed_status,
        SnsPolicyObservationStatus::ObservedSatisfied
    );
    Ok(())
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_exposes_neuron_request_constructor() {
    let checkpoint = SnsRewardCheckpointRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        SAMPLE_SNS_ROOT_CANISTER_ID,
    )
    .with_max_pages(Some(10));
    assert_eq!(checkpoint.max_pages, Some(10));

    let detail = SnsNeuronRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        SAMPLE_SNS_ROOT_CANISTER_ID,
        SAMPLE_SNS_NEURON_ID,
    );
    assert_eq!(detail.neuron_id, SAMPLE_SNS_NEURON_ID);

    let cache_root = PathBuf::from("target/ic-query-sns-public-api-empty-root");
    let request = SnsNeuronsRequest::new(
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        SAMPLE_SNS_ROOT_CANISTER_ID,
        50,
    )
    .with_owner_principal_id("aaaaa-aa")
    .with_sort(SnsNeuronsSort::Stake)
    .with_cache_root(cache_root.clone())
    .with_verbose(true);

    assert_eq!(request.input, SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(request.limit, 50);
    assert_eq!(request.owner_principal_id.as_deref(), Some("aaaaa-aa"));
    assert_eq!(request.sort, SnsNeuronsSort::Stake);
    assert_eq!(request.cache_root.as_deref(), Some(cache_root.as_path()));
    assert!(request.verbose);
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_exposes_catalog_cache_contract() {
    let cache_root = PathBuf::from("target/ic-query-sns-public-api-empty-root");
    let catalog_request = SnsCatalogCacheRequest::new(&cache_root, "ic");
    let catalog_path = sns_catalog_cache_path(&cache_root, "ic");
    let catalog_lock_path = sns_catalog_refresh_lock_path(&cache_root, "ic");
    assert_eq!(catalog_request.cache_root, cache_root);
    assert_eq!(
        catalog_lock_path,
        catalog_path.with_file_name("full.refresh.lock")
    );
    let list_request = SnsListRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000);
    let _: fn(&SnsListRequest, &Path) -> Result<SnsListReport, SnsHostError> =
        build_sns_list_report_from_cache;
    let _: fn(
        &SnsListRequest,
        &Path,
        &mut dyn ic_query::QueryProgress,
    ) -> Result<SnsListReport, SnsHostError> = build_sns_list_report_from_cache_or_refresh;
    assert!(matches!(
        build_sns_list_report_from_cache(&list_request, &cache_root),
        Err(SnsHostError::MissingCatalogCache { .. })
    ));

    let refresh_request = SnsCatalogRefreshRequest::new(
        &cache_root,
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        30 * 60,
    );
    assert_eq!(refresh_request.cache, catalog_request);
    let refresh_report = SnsCatalogRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        cache_path: catalog_path.display().to_string(),
        refresh_lock_path: catalog_lock_path.display().to_string(),
        replaced_existing_cache: false,
        sns_count: 1,
        metadata_error_count: 0,
        lifecycle_error_count: 0,
    };
    assert!(sns_catalog_refresh_report_text(&refresh_report).contains("sns_count: 1"));

    let parse_error =
        serde_json::from_str::<serde_json::Value>("{").expect_err("malformed public cache fixture");
    let error = SnsHostError::from(HostCacheError::parse_cache(
        "SNS",
        "cache.json".into(),
        parse_error,
    ));
    assert!(matches!(
        error,
        SnsHostError::Cache(HostCacheError::ParseCache { .. })
    ));
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_exposes_cache_paths_and_local_reports() -> Result<(), SnsHostError> {
    let cache_root = PathBuf::from("target/ic-query-sns-public-api-empty-root");

    let neurons_cache_path = sns_neurons_cache_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let neurons_lock_path =
        sns_neurons_refresh_lock_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let neurons_attempt_path =
        sns_neurons_refresh_attempt_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(
        neurons_lock_path,
        neurons_cache_path.with_file_name("full.refresh.lock")
    );
    assert_eq!(
        neurons_attempt_path,
        neurons_cache_path.with_file_name("full.refresh-attempt.json")
    );

    let proposals_cache_path =
        sns_proposals_cache_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let proposals_lock_path =
        sns_proposals_refresh_lock_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let proposals_attempt_path =
        sns_proposals_refresh_attempt_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(
        proposals_lock_path,
        proposals_cache_path.with_file_name("full.refresh.lock")
    );
    assert_eq!(
        proposals_attempt_path,
        proposals_cache_path.with_file_name("full.refresh-attempt.json")
    );

    let neurons_list_request = SnsCacheListRequest::new(cache_root.clone(), "ic");
    assert_eq!(neurons_list_request.cache_root(), cache_root.as_path());
    let neurons_list_report = build_sns_neurons_cache_list_report(&neurons_list_request)?;
    assert_eq!(neurons_list_report.cache_count, 0);
    assert!(sns_neurons_cache_list_report_text(&neurons_list_report).contains("cache_count: 0"));

    let neurons_status_request =
        SnsCacheStatusRequest::new(cache_root.clone(), "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(neurons_status_request.cache_root(), cache_root.as_path());
    let neurons_status_report = build_sns_neurons_cache_status_report(&neurons_status_request)?;
    assert!(!neurons_status_report.found);
    let expected_neurons_cache_path = neurons_cache_path.display().to_string();
    assert_eq!(
        neurons_status_report.expected_cache_path.as_deref(),
        Some(expected_neurons_cache_path.as_str())
    );
    assert!(sns_neurons_cache_status_report_text(&neurons_status_report).contains("found: no"));

    let proposals_list_request = SnsCacheListRequest::new(cache_root.clone(), "ic");
    assert_eq!(proposals_list_request.cache_root(), cache_root.as_path());
    let proposals_list_report = build_sns_proposals_cache_list_report(&proposals_list_request)?;
    assert_eq!(proposals_list_report.cache_count, 0);
    assert!(
        sns_proposals_cache_list_report_text(&proposals_list_report).contains("cache_count: 0")
    );

    let proposals_status_request =
        SnsCacheStatusRequest::new(cache_root.clone(), "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(proposals_status_request.cache_root(), cache_root.as_path());
    let proposals_status_report =
        build_sns_proposals_cache_status_report(&proposals_status_request)?;
    assert!(!proposals_status_report.found);
    let expected_proposals_cache_path = proposals_cache_path.display().to_string();
    assert_eq!(
        proposals_status_report.expected_cache_path.as_deref(),
        Some(expected_proposals_cache_path.as_str())
    );
    assert!(sns_proposals_cache_status_report_text(&proposals_status_report).contains("found: no"));

    Ok(())
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_exposes_refresh_requests_and_renderers() {
    let cache_root = PathBuf::from("target/ic-query-sns-public-api-empty-root");
    let neurons_cache_path = sns_neurons_cache_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let neurons_lock_path =
        sns_neurons_refresh_lock_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let neurons_attempt_path =
        sns_neurons_refresh_attempt_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let proposals_cache_path =
        sns_proposals_cache_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let proposals_lock_path =
        sns_proposals_refresh_lock_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    let proposals_attempt_path =
        sns_proposals_refresh_attempt_path(&cache_root, "ic", SAMPLE_SNS_ROOT_CANISTER_ID);

    let neurons_refresh_request = SnsNeuronsRefreshRequest::new(
        cache_root.clone(),
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        SAMPLE_SNS_ROOT_CANISTER_ID,
        500,
    )
    .with_max_pages(Some(2));
    assert_eq!(neurons_refresh_request.max_pages, Some(2));

    let proposals_refresh_request = SnsProposalsRefreshRequest::new(
        cache_root,
        "ic",
        DEFAULT_SNS_SOURCE_ENDPOINT,
        1_700_000_000,
        SAMPLE_SNS_ROOT_CANISTER_ID,
        100,
    )
    .with_max_pages(Some(3));
    assert_eq!(proposals_refresh_request.max_pages, Some(3));

    let neurons_report = sample_sns_neurons_report();
    assert!(sns_neurons_report_text(&neurons_report).contains("neuron_count: 1"));
    let neurons_json = serde_json::to_value(&neurons_report).expect("serialize neurons report");
    assert_eq!(neurons_json["schema_version"], 1);
    assert_eq!(neurons_json["neurons"][0]["source_nns_neuron_id"], 42);
    assert_eq!(neurons_json["neurons"][0]["auto_stake_maturity"], true);
    assert_eq!(
        neurons_json["neurons"][0]["dissolve_state"],
        json!({"kind": "dissolve_delay_seconds", "value": 31_536_000})
    );
    assert_eq!(neurons_json["neurons"][0]["neuron_fees_e8s"], 10);
    assert!(
        sns_neurons_refresh_report_text(&sample_sns_neurons_refresh_report(
            &neurons_cache_path,
            &neurons_lock_path,
            &neurons_attempt_path,
        ))
        .contains("wrote_cache: yes")
    );
    assert!(
        sns_proposals_refresh_report_text(&sample_sns_proposals_refresh_report(
            &proposals_cache_path,
            &proposals_lock_path,
            &proposals_attempt_path,
        ))
        .contains("proposal_count: 1")
    );
}

#[cfg(feature = "host")]
fn accepts_public_function<T>(_function: T) {}

#[cfg(feature = "host")]
struct FixtureSnsSource;

#[cfg(feature = "host")]
impl SnsDiscoverySource for FixtureSnsSource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        assert_eq!(request.endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
        Ok(sample_mainnet_sns_inventory(request))
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        assert_eq!(request.endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
        Ok(targets
            .iter()
            .map(|target| MainnetSnsMetadata {
                root_canister_id: target.root_canister_id.clone(),
                name: Some("Example SNS".to_string()),
                description: Some("Example description".to_string()),
                url: Some("https://example.com/sns".to_string()),
                metadata_error: None,
            })
            .collect())
    }
}

#[cfg(feature = "host")]
impl SnsCatalogSource for FixtureSnsSource {
    fn fetch_sns_lifecycles(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        assert_eq!(request.endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
        Ok(targets
            .iter()
            .map(|target| MainnetSnsLifecycle {
                root_canister_id: target.root_canister_id.clone(),
                lifecycle: Some(3),
                lifecycle_name: Some("committed".to_string()),
                lifecycle_error: None,
            })
            .collect())
    }
}

#[cfg(feature = "host")]
impl SnsCanisterSource for FixtureSnsSource {
    fn fetch_sns_canisters(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsCanisterInventory, SnsHostError> {
        Ok(MainnetSnsCanisterInventory {
            inventory_method: SnsCanisterMethod::ListSnsCanisters,
            health_method: SnsCanisterMethod::GetSnsCanistersSummary,
            health_call_type: SnsCanisterCallType::IngressUpdate,
            health_update_canister_list: false,
            point_in_time_guaranteed: false,
            canisters: vec![SnsCanisterRow {
                role: SnsCanisterRole::Root,
                canister_id: sns.root_canister_id.clone(),
                status: Some(SnsCanisterStatus::Running),
                module_hash_hex: Some("01020304".to_string()),
                cycles: Some("1000000".to_string()),
                cycle_balance_status: SnsCanisterCycleBalanceStatus::ReportedNonzero,
                memory_size: Some("2000000".to_string()),
                idle_cycles_burned_per_day: Some("3000".to_string()),
                controllers: vec![sns.governance_canister_id.clone()],
            }],
            health_query_gap: None,
            gaps: Vec::new(),
        })
    }
}

#[cfg(feature = "host")]
impl SnsTokenSource for FixtureSnsSource {
    fn fetch_sns_token(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        assert_eq!(sns.root_canister_id, SAMPLE_SNS_ROOT_CANISTER_ID);
        Ok(sample_mainnet_sns_token())
    }
}

#[cfg(feature = "host")]
impl SnsParamsSource for FixtureSnsSource {
    fn fetch_sns_params(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        Ok(sample_sns_governance_parameters())
    }
}

#[cfg(feature = "host")]
impl SnsMetricsSource for FixtureSnsSource {
    fn fetch_sns_metrics(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        time_window_seconds: u64,
    ) -> Result<MainnetSnsMetrics, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        Ok(sample_mainnet_sns_metrics(time_window_seconds))
    }
}

#[cfg(feature = "host")]
impl SnsSwapSource for FixtureSnsSource {
    fn fetch_sns_swap(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsSwap, SnsHostError> {
        assert_eq!(sns.swap_canister_id, SAMPLE_SNS_SWAP_CANISTER_ID);
        Ok(sample_mainnet_sns_swap())
    }
}

#[cfg(feature = "host")]
impl SnsUpgradeSource for FixtureSnsSource {
    fn fetch_sns_upgrade(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsUpgrade, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        Ok(sample_mainnet_sns_upgrade())
    }
}

#[cfg(feature = "host")]
impl SnsProposalSource for FixtureSnsSource {
    fn fetch_sns_proposal(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        assert_eq!(proposal_id, 42);
        Ok(MainnetSnsProposal {
            proposal: sample_sns_proposal_row(),
        })
    }
}

#[cfg(feature = "host")]
impl SnsProposalsSource for FixtureSnsSource {
    fn fetch_sns_proposals(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        assert_eq!(limit, 10);
        assert_eq!(before_proposal_id, Some(99));
        assert_eq!(include_status, &[1]);
        assert_eq!(topic, SnsProposalTopicFilter::Governance);
        Ok(MainnetSnsProposals {
            proposals: vec![sample_sns_proposal_row()],
        })
    }

    fn fetch_sns_proposal_page(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> Result<MainnetSnsProposalPage, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        assert_eq!(limit, 100);
        assert_eq!(before_proposal_id, None);
        Ok(MainnetSnsProposalPage {
            proposals: vec![sample_sns_proposal_row()],
        })
    }
}

#[cfg(feature = "host")]
impl SnsNeuronSource for FixtureSnsSource {
    fn fetch_sns_neuron(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        neuron_id: &str,
    ) -> Result<MainnetSnsNeuron, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        assert_eq!(neuron_id, SAMPLE_SNS_NEURON_ID);
        Ok(MainnetSnsNeuron {
            detail: sample_sns_neuron_detail(),
        })
    }
}

#[cfg(feature = "host")]
impl SnsNeuronsSource for FixtureSnsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        assert_eq!(limit, 50);
        assert_eq!(owner_principal_id, Some(SAMPLE_SNS_GOVERNANCE_CANISTER_ID));
        Ok(MainnetSnsNeurons {
            neurons: vec![sample_sns_neuron_row()],
        })
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(
            sns.governance_canister_id,
            SAMPLE_SNS_GOVERNANCE_CANISTER_ID
        );
        assert_eq!(limit, 100);
        assert!(start_page_at.is_none());
        assert_eq!(owner_principal_id, None);
        Ok(MainnetSnsNeuronPage {
            neurons: vec![sample_sns_neuron_row()],
            last_cursor: Some(SnsNeuronId { id: vec![1; 32] }),
        })
    }
}

#[cfg(feature = "host")]
impl SnsRewardSource for FixtureSnsSource {
    fn fetch_sns_reward_running_version(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<SnsRunningVersionResponse, SnsHostError> {
        Ok(SnsRunningVersionResponse {
            deployed_version: Some(sample_sns_version(&"01".repeat(32))),
            pending_version: None,
        })
    }

    fn fetch_sns_reward_parameters(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        let mut parameters = sample_sns_governance_parameters();
        parameters.max_number_of_neurons = Some(100);
        parameters.neuron_grantable_permissions = Some(SnsNeuronPermissionList {
            permissions: vec![2, 4],
        });
        Ok(parameters)
    }

    fn fetch_sns_reward_event(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
    ) -> Result<SnsRewardEvent, SnsHostError> {
        Ok(sample_sns_reward_checkpoint_report().reward_event_after)
    }

    fn fetch_sns_reward_neuron_page(
        &self,
        _request: &SnsSourceRequest,
        _sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
    ) -> Result<MainnetSnsRewardNeuronPage, SnsHostError> {
        assert_eq!(limit, 100);
        assert!(start_page_at.is_none());
        Ok(MainnetSnsRewardNeuronPage {
            neurons: sample_sns_reward_checkpoint_report().rows,
            next_cursor: None,
        })
    }
}

#[cfg(feature = "host")]
fn proposal_source_cache_root() -> PathBuf {
    PathBuf::from(format!(
        "target/ic-query-sns-public-api-proposal-source-{}",
        std::process::id()
    ))
}

#[cfg(feature = "host")]
fn neuron_source_cache_root() -> PathBuf {
    PathBuf::from(format!(
        "target/ic-query-sns-public-api-neuron-source-{}",
        std::process::id()
    ))
}

#[cfg(feature = "host")]
fn sample_mainnet_sns_inventory(request: &SnsSourceRequest) -> MainnetSnsInventory {
    MainnetSnsInventory {
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances: vec![MainnetSnsCanisters {
            root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
            governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
            ledger_canister_id: SAMPLE_SNS_LEDGER_CANISTER_ID.to_string(),
            swap_canister_id: SAMPLE_SNS_SWAP_CANISTER_ID.to_string(),
            index_canister_id: SAMPLE_SNS_INDEX_CANISTER_ID.to_string(),
        }],
    }
}

#[cfg(feature = "host")]
fn sample_mainnet_sns_token() -> MainnetSnsToken {
    MainnetSnsToken {
        token_name: "Example Token".to_string(),
        token_symbol: "EXT".to_string(),
        decimals: 8,
        transfer_fee: "100000000".to_string(),
        total_supply: "1000000000".to_string(),
        minting_account_owner: Some("aaaaa-aa".to_string()),
        minting_account_subaccount_hex: None,
        ledger_index_canister_id: Some(SAMPLE_SNS_INDEX_CANISTER_ID.to_string()),
        ledger_index_error: None,
        supported_standards: vec![SnsTokenStandardRow {
            name: "ICRC-1".to_string(),
            url: "https://github.com/dfinity/ICRC-1".to_string(),
        }],
        metadata: vec![SnsTokenMetadataRow {
            key: "icrc1:symbol".to_string(),
            value_type: IcrcMetadataValueKind::Text,
            value: json!("EXT"),
        }],
    }
}

#[cfg(feature = "host")]
fn sample_mainnet_sns_swap() -> MainnetSnsSwap {
    MainnetSnsSwap {
        swap_canister_id: SAMPLE_SNS_SWAP_CANISTER_ID.to_string(),
        lifecycle_method: SnsCanisterMethod::GetLifecycle,
        sale_parameters_method: SnsCanisterMethod::GetSaleParameters,
        derived_state_method: SnsCanisterMethod::GetDerivedState,
        point_in_time_guaranteed: false,
        lifecycle: Some(SnsSwapLifecycle {
            lifecycle: Some(3),
            lifecycle_name: Some("committed".to_string()),
            decentralization_sale_open_timestamp_seconds: Some(1_700_000_000),
            decentralization_swap_termination_timestamp_seconds: Some(1_700_086_400),
        }),
        sale_parameters: None,
        derived_state: Some(SnsSwapDerivedState {
            sns_tokens_per_icp: Some(2.5),
            buyer_total_icp_e8s: Some(1_000_000_000),
            direct_participation_icp_e8s: Some(900_000_000),
            neurons_fund_participation_icp_e8s: Some(100_000_000),
            direct_participant_count: Some(10),
            cf_participant_count: None,
            cf_neuron_count: None,
        }),
        gaps: Vec::new(),
    }
}

#[cfg(feature = "host")]
fn sample_mainnet_sns_metrics(time_window_seconds: u64) -> MainnetSnsMetrics {
    MainnetSnsMetrics {
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        method: SnsCanisterMethod::GetMetrics,
        call_type: SnsCanisterCallType::CompositeQuery,
        time_window_seconds,
        point_in_time_guaranteed: false,
        treasury_metrics_cached: true,
        num_recently_submitted_proposals: Some(3),
        num_recently_executed_proposals: Some(2),
        last_ledger_block_timestamp: Some(1_700_000_010),
        genesis_timestamp_seconds: Some(1_600_000_000),
        treasury_metrics: vec![SnsTreasuryMetricRow {
            treasury: 1,
            treasury_kind: SnsTreasuryKind::Icp,
            name: Some("ICP treasury".to_string()),
            ledger_canister_id: None,
            account_owner: None,
            account_subaccount_hex: None,
            amount_e8s: Some(100_000_000),
            original_amount_e8s: Some(200_000_000),
            timestamp_seconds: Some(1_700_000_000),
        }],
        voting_power_metrics: Some(SnsVotingPowerMetrics {
            governance_total_potential_voting_power: Some(500_000_000),
            timestamp_seconds: Some(1_700_000_001),
        }),
    }
}

fn sample_sns_version(hash: &str) -> SnsVersion {
    SnsVersion {
        archive_wasm_hash_hex: hash.to_string(),
        root_wasm_hash_hex: hash.to_string(),
        swap_wasm_hash_hex: hash.to_string(),
        ledger_wasm_hash_hex: hash.to_string(),
        governance_wasm_hash_hex: hash.to_string(),
        index_wasm_hash_hex: hash.to_string(),
    }
}

#[cfg(feature = "host")]
fn sample_mainnet_sns_upgrade() -> MainnetSnsUpgrade {
    MainnetSnsUpgrade {
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        running_version_method: SnsCanisterMethod::GetRunningSnsVersion,
        next_version_method: SnsCanisterMethod::GetNextSnsVersion,
        point_in_time_guaranteed: false,
        deployed_version: sample_sns_version("01"),
        pending_upgrade: None,
        next_version: Some(sample_sns_version("02")),
        next_version_gap: None,
    }
}

#[cfg(feature = "host")]
fn sample_sns_neuron_row() -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
        cached_neuron_stake_e8s: 100_000_000,
        maturity_e8s_equivalent: 10_000_000,
        staked_maturity_e8s_equivalent: Some(5_000_000),
        created_timestamp_seconds: 1_700_000_000,
        created_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_nns_neuron_id: Some(42),
        auto_stake_maturity: Some(true),
        aging_since_timestamp_seconds: 1_700_000_100,
        dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000)),
        voting_power_percentage_multiplier: 100,
        vesting_period_seconds: Some(63_072_000),
        neuron_fees_e8s: 10,
    }
}

fn sample_sns_governance_parameters() -> SnsGovernanceParameters {
    SnsGovernanceParameters {
        default_followees: None,
        max_dissolve_delay_seconds: Some(7_200),
        max_dissolve_delay_bonus_percentage: Some(50),
        max_followees_per_function: Some(15),
        neuron_claimer_permissions: Some(SnsNeuronPermissionList {
            permissions: vec![1, 2],
        }),
        neuron_minimum_stake_e8s: Some(100_000_000),
        max_neuron_age_for_age_bonus: Some(86_400),
        initial_voting_period_seconds: Some(3_600),
        neuron_minimum_dissolve_delay_to_vote_seconds: Some(600),
        reject_cost_e8s: Some(10_000_000),
        max_proposals_to_keep_per_action: Some(100),
        wait_for_quiet_deadline_increase_seconds: Some(300),
        max_number_of_neurons: Some(10_000),
        transaction_fee_e8s: Some(10_000),
        max_number_of_proposals_with_ballots: Some(500),
        max_age_bonus_percentage: Some(25),
        neuron_grantable_permissions: None,
        voting_rewards_parameters: Some(SnsVotingRewardsParameters {
            final_reward_rate_basis_points: Some(125),
            initial_reward_rate_basis_points: Some(250),
            reward_rate_transition_duration_seconds: Some(31_536_000),
            round_duration_seconds: Some(86_400),
        }),
        maturity_modulation_disabled: Some(false),
        max_number_of_principals_per_neuron: Some(5),
        automatically_advance_target_version: Some(true),
        custom_proposal_criticality: Some(SnsCustomProposalCriticality {
            additional_critical_native_action_ids: vec![7, 8],
        }),
    }
}

#[cfg(feature = "host")]
fn sample_sns_neurons_report() -> SnsNeuronsReport {
    SnsNeuronsReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        requested_limit: 1,
        owner_principal_id: None,
        verbose: false,
        data_source: ReportDataSource::Live,
        sort: SnsNeuronsSort::Api.as_str().to_string(),
        cache_path: None,
        cache_complete: None,
        total_neuron_count: 1,
        neuron_count: 1,
        neurons: vec![SnsNeuronRow {
            neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
            cached_neuron_stake_e8s: 100_000_000,
            maturity_e8s_equivalent: 10_000_000,
            staked_maturity_e8s_equivalent: Some(5_000_000),
            created_timestamp_seconds: 1_700_000_000,
            created_at: SAMPLE_SNS_FETCHED_AT.to_string(),
            source_nns_neuron_id: Some(42),
            auto_stake_maturity: Some(true),
            aging_since_timestamp_seconds: 1_700_000_100,
            dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000)),
            voting_power_percentage_multiplier: 100,
            vesting_period_seconds: Some(63_072_000),
            neuron_fees_e8s: 10,
        }],
    }
}

#[cfg(feature = "host")]
fn sample_sns_neurons_refresh_report(
    cache_path: &Path,
    refresh_lock_path: &Path,
    refresh_attempt_path: &Path,
) -> SnsNeuronsRefreshReport {
    SnsNeuronsRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_lock_path: refresh_lock_path.display().to_string(),
        refresh_attempt_path: refresh_attempt_path.display().to_string(),
        page_size: 500,
        page_count: 1,
        neuron_count: 1,
        complete: true,
        replaced_existing_cache: false,
        wrote_cache: true,
        attempt_finalization_error: None,
    }
}

#[cfg(feature = "host")]
fn sample_sns_proposals_refresh_report(
    cache_path: &Path,
    refresh_lock_path: &Path,
    refresh_attempt_path: &Path,
) -> SnsProposalsRefreshReport {
    SnsProposalsRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        cache_path: cache_path.display().to_string(),
        refresh_lock_path: refresh_lock_path.display().to_string(),
        refresh_attempt_path: refresh_attempt_path.display().to_string(),
        page_size: 100,
        page_count: 1,
        proposal_count: 1,
        complete: true,
        replaced_existing_cache: false,
        wrote_cache: true,
        attempt_finalization_error: None,
    }
}

fn sample_sns_proposal_row() -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: 42,
        action_id: 7,
        action: SnsProposalAction::UpgradeSnsToNextVersion,
        title: "Upgrade SNS".to_string(),
        summary: "Upgrade the SNS controlled canister.".to_string(),
        url: Some("https://example.com/proposal/42".to_string()),
        decision_state: SnsProposalDecisionState::Open,
        status: Some(1),
        topic: Some("governance".to_string()),
        reject_cost_e8s: 100_000_000,
        proposal_creation_timestamp_seconds: 1_700_000_000,
        created_at: "2023-11-14T22:13:20Z".to_string(),
        decided_timestamp_seconds: None,
        decided_at: None,
        executed_timestamp_seconds: None,
        executed_at: None,
        failed_timestamp_seconds: None,
        failed_at: None,
        failure_reason: Some(SnsProposalFailureReason {
            error_type: 0,
            error_message: "none".to_string(),
        }),
        reward_event_round: 12,
        reward_event_end_timestamp_seconds: Some(1_700_086_400),
        is_eligible_for_rewards: true,
        latest_tally: Some(SnsProposalTally {
            timestamp_seconds: 1_700_000_100,
            yes: 100_000_000,
            no: 10_000_000,
            total: 110_000_000,
        }),
        ballot_count: 1,
        ballots: vec![SnsProposalBallotRow {
            neuron_id: "0102030405060708".to_string(),
            vote: 1,
            vote_text: SnsProposalVote::Yes,
            cast_timestamp_seconds: 1_700_000_200,
            cast_at: Some("2023-11-14T22:16:40Z".to_string()),
            voting_power: 100_000_000,
        }],
        payload_text_rendering: Some("Upgrade payload".to_string()),
        proposer_neuron_id: Some("010203".to_string()),
    }
}

fn sample_sns_neuron_detail() -> SnsNeuronDetail {
    SnsNeuronDetail {
        neuron: SnsNeuronRow {
            neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
            cached_neuron_stake_e8s: 100_000_000,
            maturity_e8s_equivalent: 10_000_000,
            staked_maturity_e8s_equivalent: Some(5_000_000),
            created_timestamp_seconds: 1_700_000_000,
            created_at: SAMPLE_SNS_FETCHED_AT.to_string(),
            source_nns_neuron_id: Some(42),
            auto_stake_maturity: Some(true),
            aging_since_timestamp_seconds: 1_700_000_100,
            dissolve_state: Some(SnsNeuronDissolveState::DissolveDelaySeconds(31_536_000)),
            voting_power_percentage_multiplier: 100,
            vesting_period_seconds: Some(63_072_000),
            neuron_fees_e8s: 10_000,
        },
        permissions: vec![SnsNeuronPermissionRow {
            principal: Some(SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string()),
            permission_types: vec![SnsNeuronPermissionValue::from_code(7)],
        }],
        disburse_maturity_in_progress: vec![SnsMaturityDisbursementRow {
            timestamp_of_disbursement_seconds: 1_700_000_200,
            amount_e8s: 1_000,
            account_to_disburse_to: Some(SnsNeuronAccount {
                owner: Some(SAMPLE_SNS_ROOT_CANISTER_ID.to_string()),
                subaccount_hex: Some("ab".repeat(32)),
            }),
            finalize_disbursement_timestamp_seconds: Some(1_700_086_600),
        }],
        followees: vec![SnsNeuronFolloweesRow {
            function_id: 1,
            followee_neuron_ids: vec!["11".repeat(32)],
        }],
        topic_followees: Some(vec![SnsNeuronTopicFolloweesRow {
            topic_code: 5,
            topic: Some("governance".to_string()),
            followees: vec![SnsNeuronFolloweeRow {
                neuron_id: Some("22".repeat(32)),
                alias: Some("lead".to_string()),
            }],
        }]),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::Violated,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
    }
}

fn sample_sns_reward_checkpoint_report() -> SnsRewardCheckpointReport {
    let mut parameters = sample_sns_governance_parameters();
    parameters.neuron_grantable_permissions = Some(SnsNeuronPermissionList {
        permissions: vec![2, 4],
    });
    let reward_event = SnsRewardEvent {
        rounds_since_last_distribution: Some(1),
        actual_timestamp_seconds: 1_700_086_300,
        end_timestamp_seconds: Some(1_700_086_400),
        total_available_e8s_equivalent: Some(10_000),
        distributed_e8s_equivalent: 5_000,
        round: 42,
        settled_proposals: vec![SnsRewardProposalId { id: 7 }],
    };
    let running_version = SnsRunningVersionResponse {
        deployed_version: Some(sample_sns_version(&"01".repeat(32))),
        pending_version: None,
    };
    let row = SnsRewardCheckpointRow {
        neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
        created_timestamp_seconds: 1_700_000_000,
        maturity_e8s_equivalent: 10_000_000,
        staked_maturity_e8s_equivalent: Some(5_000_000),
        combined_maturity_e8s_equivalent: 15_000_000,
        auto_stake_maturity: Some(true),
        permissions: vec![SnsNeuronPermissionRow {
            principal: Some(SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string()),
            permission_types: vec![SnsNeuronPermissionValue::from_code(4)],
        }],
        disburse_maturity_in_progress: Vec::new(),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
    };
    SnsRewardCheckpointReport {
        schema_version: 1,
        network: "ic".to_string(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        swap_canister_id: "br5f7-7uaaa-aaaaa-qaaca-cai".to_string(),
        index_canister_id: "bw4dl-smaaa-aaaaa-qaacq-cai".to_string(),
        data_source: ReportDataSource::Live,
        collection_started_at_unix_secs: 1_700_086_400,
        collection_started_at: "2023-11-15T22:13:20Z".to_string(),
        collection_completed_at_unix_secs: 1_700_086_401,
        collection_completed_at: "2023-11-15T22:13:21Z".to_string(),
        page_size: 100,
        page_count: 1,
        row_count: 1,
        unique_neuron_id_count: 1,
        collection_row_ceiling: 10_000,
        client_query_count: 9,
        collection_status: SnsRewardCollectionStatus::ApiExhaustedObserved,
        point_in_time_guaranteed: false,
        parameters_before: parameters.clone(),
        parameters_after: parameters,
        reward_event_before: reward_event.clone(),
        reward_event_after: reward_event,
        running_version_before: running_version.clone(),
        running_version_after: running_version,
        aggregate_maturity_e8s_equivalent: 10_000_000,
        aggregate_staked_maturity_e8s_equivalent: 5_000_000,
        aggregate_combined_maturity_e8s_equivalent: 15_000_000,
        permission_entry_count: 1,
        unassessable_permission_code_count: 0,
        pending_maturity_disbursement_count: 0,
        auto_stake_maturity_enabled_count: 1,
        auto_stake_maturity_disabled_count: 0,
        auto_stake_maturity_unspecified_count: 0,
        manage_principals_grantable: Some(true),
        maturity_mint_conversion_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        manual_maturity_staking_observed_disabled: SnsPolicyObservationStatus::ObservedSatisfied,
        maturity_conversion_policy_observed_status: SnsPolicyObservationStatus::ObservedSatisfied,
        rows: vec![row],
    }
}

fn advance_reward_checkpoint(
    before: &SnsRewardCheckpointReport,
    maturity_delta: u64,
    distributed: u64,
) -> SnsRewardCheckpointReport {
    let mut after = before.clone();
    after.collection_started_at_unix_secs = 1_700_172_800;
    after.collection_started_at = "2023-11-16T22:13:20Z".to_string();
    after.collection_completed_at_unix_secs = 1_700_172_801;
    after.collection_completed_at = "2023-11-16T22:13:21Z".to_string();
    let mut event = before.reward_event_after.clone();
    event.actual_timestamp_seconds = 1_700_172_700;
    event.end_timestamp_seconds = Some(1_700_172_800);
    event.distributed_e8s_equivalent = distributed;
    event.round = before.reward_event_after.round + 1;
    event.rounds_since_last_distribution = Some(1);
    after.reward_event_before = event.clone();
    after.reward_event_after = event;
    after.rows[0].maturity_e8s_equivalent += maturity_delta;
    after.rows[0].combined_maturity_e8s_equivalent += maturity_delta;
    after.aggregate_maturity_e8s_equivalent += maturity_delta;
    after.aggregate_combined_maturity_e8s_equivalent += maturity_delta;
    after
}

fn clear_reward_checkpoint_rows(report: &mut SnsRewardCheckpointReport) {
    report.rows.clear();
    report.row_count = 0;
    report.unique_neuron_id_count = 0;
    report.aggregate_maturity_e8s_equivalent = 0;
    report.aggregate_staked_maturity_e8s_equivalent = 0;
    report.aggregate_combined_maturity_e8s_equivalent = 0;
    report.permission_entry_count = 0;
    report.unassessable_permission_code_count = 0;
    report.pending_maturity_disbursement_count = 0;
    report.auto_stake_maturity_enabled_count = 0;
    report.auto_stake_maturity_disabled_count = 0;
    report.auto_stake_maturity_unspecified_count = 0;
}
