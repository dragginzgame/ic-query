#[cfg(feature = "host")]
use ic_query::sns::{
    DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS,
    DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, DEFAULT_SNS_SOURCE_ENDPOINT, SnsHostError,
    SnsNeuronRow, SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusRequest,
    SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest, SnsNeuronsReport, SnsNeuronsRequest,
    SnsNeuronsSort, SnsProposalsCacheListRequest, SnsProposalsCacheStatusRequest,
    SnsProposalsRefreshReport, SnsProposalsRefreshRequest, build_sns_info_report,
    build_sns_list_report, build_sns_neurons_cache_list_report,
    build_sns_neurons_cache_status_report, build_sns_neurons_report, build_sns_params_report,
    build_sns_proposal_report, build_sns_proposals_cache_list_report,
    build_sns_proposals_cache_status_report, build_sns_proposals_report, build_sns_token_report,
    refresh_sns_neurons_cache, refresh_sns_proposals_cache, sns_neurons_cache_list_report_text,
    sns_neurons_cache_path, sns_neurons_cache_status_report_text, sns_neurons_refresh_attempt_path,
    sns_neurons_refresh_lock_path, sns_neurons_refresh_report_text, sns_neurons_report_text,
    sns_proposals_cache_list_report_text, sns_proposals_cache_path,
    sns_proposals_cache_status_report_text, sns_proposals_refresh_attempt_path,
    sns_proposals_refresh_lock_path, sns_proposals_refresh_report_text,
};
use ic_query::sns::{
    SnsCustomProposalCriticality, SnsGovernanceParameters, SnsInfoReport, SnsInfoRequest,
    SnsListReport, SnsListRequest, SnsListSort, SnsNeuronPermissionList, SnsParamsReport,
    SnsParamsRequest, SnsProposalBallotRow, SnsProposalEligibilityFilter, SnsProposalFailureReason,
    SnsProposalReport, SnsProposalRequest, SnsProposalRow, SnsProposalSortDirection,
    SnsProposalStatusFilter, SnsProposalTally, SnsProposalTopicFilter, SnsProposalsReport,
    SnsProposalsRequest, SnsProposalsSort, SnsTokenMetadataRow, SnsTokenReport, SnsTokenRequest,
    SnsTokenStandardRow, SnsVotingRewardsParameters, sns_info_report_text, sns_list_report_text,
    sns_params_report_text, sns_proposal_report_text, sns_proposals_report_text,
    sns_token_report_text,
};
use serde_json::json;
#[cfg(feature = "host")]
use std::path::{Path, PathBuf};

#[cfg(feature = "host")]
const SAMPLE_SNS_ROOT_CANISTER_ID: &str = "be2us-64aaa-aaaaa-qaabq-cai";
#[cfg(feature = "host")]
const SAMPLE_SNS_GOVERNANCE_CANISTER_ID: &str = "csyra-haaaa-aaaaa-qaaeq-cai";
#[cfg(feature = "host")]
const SAMPLE_SNS_FETCHED_AT: &str = "2023-11-14T22:13:20Z";

#[cfg(feature = "host")]
type SnsListBuilder = fn(&SnsListRequest) -> Result<SnsListReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsInfoBuilder = fn(&SnsInfoRequest) -> Result<SnsInfoReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsTokenBuilder = fn(&SnsTokenRequest) -> Result<SnsTokenReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsParamsBuilder = fn(&SnsParamsRequest) -> Result<SnsParamsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsProposalsBuilder = fn(&SnsProposalsRequest) -> Result<SnsProposalsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsProposalBuilder = fn(&SnsProposalRequest) -> Result<SnsProposalReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsNeuronsBuilder = fn(&SnsNeuronsRequest) -> Result<SnsNeuronsReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsNeuronsRefreshBuilder =
    fn(&SnsNeuronsRefreshRequest) -> Result<SnsNeuronsRefreshReport, SnsHostError>;
#[cfg(feature = "host")]
type SnsProposalsRefreshBuilder =
    fn(&SnsProposalsRefreshRequest) -> Result<SnsProposalsRefreshReport, SnsHostError>;

#[test]
fn public_sns_list_api_is_constructible_and_renderable() {
    let request = SnsListRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        verbose: false,
        sort: SnsListSort::Id,
    };

    assert_eq!(request.sort.as_str(), "id");

    let report = SnsListReport {
        schema_version: 3,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        verbose: request.verbose,
        sort: request.sort.as_str().to_string(),
        sns_count: 0,
        metadata_error_count: 0,
        sns_instances: Vec::new(),
    };

    let text = sns_list_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("sns_count: 0"));
}

#[test]
fn public_sns_info_api_is_constructible_and_renderable() {
    let request = SnsInfoRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
    };

    let report = SnsInfoReport {
        schema_version: 2,
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
        governance_canister_id: "csyra-haaaa-aaaaa-qaaeq-cai".to_string(),
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        swap_canister_id: "ca6gz-lqaaa-aaaaa-qaacu-cai".to_string(),
        index_canister_id: "qhbym-qaaaa-aaaaa-aaafq-cai".to_string(),
        metadata_error: None,
    };

    assert_eq!(request.input, "1");

    let text = sns_info_report_text(&report);

    assert!(text.contains("sns_id: 1"));
    assert!(text.contains("description: Example description"));
    assert!(text.contains("url: -"));
}

#[test]
fn public_sns_token_api_is_constructible_and_renderable() {
    let request = SnsTokenRequest {
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
        sns_index_canister_id: "qhbym-qaaaa-aaaaa-aaafq-cai".to_string(),
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
            value_type: "Text".to_string(),
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
    let request = SnsParamsRequest {
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
        governance_canister_id: "csyra-haaaa-aaaaa-qaaeq-cai".to_string(),
        parameters: SnsGovernanceParameters {
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
        },
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
        icp_root: None,
        verbose: true,
    };

    assert_eq!(request.sort.as_str(), "created");
    assert_eq!(request.topic.as_str(), "governance");

    let report = SnsProposalsReport {
        schema_version: 10,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "csyra-haaaa-aaaaa-qaaeq-cai".to_string(),
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
        data_source: "api".to_string(),
        cache_path: None,
        cache_complete: None,
        proposal_count: 1,
        proposals: vec![sample_sns_proposal_row()],
    };

    let text = sns_proposals_report_text(&report);

    assert!(text.contains("proposal_count: 1"));
    assert!(text.contains("topic_filter: governance"));
    assert!(text.contains("proposal_details:"));
    assert!(text.contains("title: Upgrade SNS"));
}

#[test]
fn public_sns_proposal_api_is_constructible_and_renderable() {
    let request = SnsProposalRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        input: "1".to_string(),
        proposal_id: 42,
        icp_root: None,
        verbose: false,
        show_ballots: true,
    };

    let report = SnsProposalReport {
        schema_version: 5,
        network: request.network,
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
        governance_canister_id: "csyra-haaaa-aaaaa-qaaeq-cai".to_string(),
        proposal_id: request.proposal_id,
        verbose: request.verbose,
        show_ballots: request.show_ballots,
        data_source: "api".to_string(),
        cache_path: None,
        cache_complete: None,
        proposal: sample_sns_proposal_row(),
    };

    let text = sns_proposal_report_text(&report);

    assert!(text.contains("proposal_id: 42"));
    assert!(text.contains("show_ballots: yes"));
    assert!(text.contains("ballots:"));
    assert!(text.contains("Upgrade SNS"));
}

#[cfg(feature = "host")]
#[test]
fn public_sns_host_api_exposes_live_builder_entry_points() {
    accepts_public_function::<SnsListBuilder>(build_sns_list_report);
    accepts_public_function::<SnsInfoBuilder>(build_sns_info_report);
    accepts_public_function::<SnsTokenBuilder>(build_sns_token_report);
    accepts_public_function::<SnsParamsBuilder>(build_sns_params_report);
    accepts_public_function::<SnsProposalsBuilder>(build_sns_proposals_report);
    accepts_public_function::<SnsProposalBuilder>(build_sns_proposal_report);
    accepts_public_function::<SnsNeuronsBuilder>(build_sns_neurons_report);
    accepts_public_function::<SnsNeuronsRefreshBuilder>(refresh_sns_neurons_cache);
    accepts_public_function::<SnsProposalsRefreshBuilder>(refresh_sns_proposals_cache);
    assert_eq!(DEFAULT_SNS_NEURONS_REFRESH_LOCK_STALE_SECONDS, 30 * 60);
    assert_eq!(DEFAULT_SNS_PROPOSALS_REFRESH_LOCK_STALE_SECONDS, 30 * 60);
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

    let neurons_list_request = SnsNeuronsCacheListRequest::new(cache_root.clone(), "ic");
    assert_eq!(neurons_list_request.icp_root(), cache_root.as_path());
    let neurons_list_report = build_sns_neurons_cache_list_report(&neurons_list_request)?;
    assert_eq!(neurons_list_report.cache_count, 0);
    assert!(sns_neurons_cache_list_report_text(&neurons_list_report).contains("cache_count: 0"));

    let neurons_status_request =
        SnsNeuronsCacheStatusRequest::new(cache_root.clone(), "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(neurons_status_request.icp_root(), cache_root.as_path());
    let neurons_status_report = build_sns_neurons_cache_status_report(&neurons_status_request)?;
    assert!(!neurons_status_report.found);
    let expected_neurons_cache_path = neurons_cache_path.display().to_string();
    assert_eq!(
        neurons_status_report.expected_cache_path.as_deref(),
        Some(expected_neurons_cache_path.as_str())
    );
    assert!(sns_neurons_cache_status_report_text(&neurons_status_report).contains("found: no"));

    let proposals_list_request = SnsProposalsCacheListRequest::new(cache_root.clone(), "ic");
    assert_eq!(proposals_list_request.icp_root(), cache_root.as_path());
    let proposals_list_report = build_sns_proposals_cache_list_report(&proposals_list_request)?;
    assert_eq!(proposals_list_report.cache_count, 0);
    assert!(
        sns_proposals_cache_list_report_text(&proposals_list_report).contains("cache_count: 0")
    );

    let proposals_status_request =
        SnsProposalsCacheStatusRequest::new(cache_root.clone(), "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(proposals_status_request.icp_root(), cache_root.as_path());
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

    assert!(sns_neurons_report_text(&sample_sns_neurons_report()).contains("neuron_count: 1"));
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
        data_source: "api".to_string(),
        sort: SnsNeuronsSort::Api.as_str().to_string(),
        cache_path: None,
        cache_complete: None,
        total_neuron_count: 1,
        neuron_count: 1,
        neurons: vec![SnsNeuronRow {
            neuron_id: "0102030405060708".to_string(),
            cached_neuron_stake_e8s: 100_000_000,
            maturity_e8s_equivalent: 10_000_000,
            staked_maturity_e8s_equivalent: Some(5_000_000),
            created_timestamp_seconds: 1_700_000_000,
            created_at: SAMPLE_SNS_FETCHED_AT.to_string(),
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
    }
}

fn sample_sns_proposal_row() -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: Some(42),
        action_id: 7,
        action: "UpgradeSnsControlledCanister".to_string(),
        title: "Upgrade SNS".to_string(),
        summary: "Upgrade the SNS controlled canister.".to_string(),
        url: Some("https://example.com/proposal/42".to_string()),
        decision_state: "open".to_string(),
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
            vote_text: "yes".to_string(),
            cast_timestamp_seconds: 1_700_000_200,
            cast_at: Some("2023-11-14T22:16:40Z".to_string()),
            voting_power: 100_000_000,
        }],
        payload_text_rendering: Some("Upgrade payload".to_string()),
        proposer_neuron_id: Some("010203".to_string()),
    }
}
