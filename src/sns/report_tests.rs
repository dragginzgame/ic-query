use super::*;
use crate::test_support::temp_dir;
use std::fs;

const ROOT_A: &str = "be2us-64aaa-aaaaa-qaabq-cai";
const GOVERNANCE_A: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";
const LEDGER_A: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
const SWAP_A: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
const INDEX_A: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const ROOT_B: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
const GOVERNANCE_B: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
const LEDGER_B: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const SWAP_B: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const INDEX_B: &str = "r7inp-6aaaa-aaaaa-aaabq-cai";

#[test]
fn sns_list_report_uses_names_and_compact_ids_by_default() {
    let request = list_request(false);

    let report = build_sns_list_report_with_source(&request, &FixtureSnsListSource)
        .expect("sns list report");
    let text = sns_list_report_text(&report);

    assert_eq!(report.schema_version, SNS_LIST_REPORT_SCHEMA_VERSION);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.sns_wasm_canister_id, MAINNET_SNS_WASM_CANISTER_ID);
    assert_eq!(report.sns_count, 1);
    assert!(!report.verbose);
    assert_eq!(report.sort, "id");
    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "Fixture SNS");
    assert_eq!(report.sns_instances[0].root_canister_id, ROOT_A);
    assert_eq!(report.metadata_error_count, 0);
    assert_eq!(report.sns_instances[0].metadata_error, None);
    assert!(text.contains("ID   NAME"));
    assert!(text.contains("NAME"));
    assert!(text.contains("sort: id"));
    assert!(text.contains("metadata_errors: 0"));
    assert!(text.contains("Fixture SNS"));
    assert!(text.contains(&ROOT_A[..COMPACT_PRINCIPAL_CHARS]));
    assert!(!text.contains(ROOT_A));
}

#[test]
fn sns_list_report_verbose_text_keeps_full_ids() {
    let request = list_request(true);

    let report = build_sns_list_report_with_source(&request, &FixtureSnsListSource)
        .expect("sns list report");
    let text = sns_list_report_text(&report);

    assert!(report.verbose);
    assert!(text.contains(ROOT_A));
    assert!(text.contains(GOVERNANCE_A));
}

#[test]
fn sns_info_resolves_list_id() {
    let request = info_request("1");

    let report = build_sns_info_report_with_source(&request, &FixtureSnsListSource)
        .expect("sns info report");
    let text = sns_info_report_text(&report);

    assert_eq!(report.schema_version, SNS_INFO_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.description.as_deref(), Some("Fixture description"));
    assert_eq!(report.url.as_deref(), Some("https://example.com"));
    assert_eq!(report.metadata_error, None);
    assert!(text.contains("root_canister_id: be2us-64aaa-aaaaa-qaabq-cai"));
}

#[test]
fn sns_info_resolves_root_principal() {
    let request = info_request(ROOT_A);

    let report = build_sns_info_report_with_source(&request, &FixtureSnsListSource)
        .expect("sns info report");

    assert_eq!(report.id, 1);
    assert_eq!(report.root_canister_id, ROOT_A);
}

#[test]
fn sns_token_resolves_list_id_and_renders_ledger_metadata() {
    let request = token_request("1");

    let report = build_sns_token_report_with_source(&request, &FixtureSnsTokenSource)
        .expect("sns token report");
    let text = sns_token_report_text(&report);

    assert_eq!(report.schema_version, SNS_TOKEN_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.ledger_canister_id, LEDGER_A);
    assert_eq!(report.sns_index_canister_id, INDEX_A);
    assert_eq!(report.ledger_index_canister_id.as_deref(), Some(INDEX_A));
    assert_eq!(report.token_name, "Fixture Token");
    assert_eq!(report.token_symbol, "FIX");
    assert_eq!(report.decimals, 8);
    assert_eq!(report.transfer_fee, "10_000");
    assert_eq!(report.total_supply, "1_000_000_000");
    assert_eq!(report.minting_account_owner.as_deref(), Some(GOVERNANCE_A));
    assert_eq!(
        report.minting_account_subaccount_hex.as_deref(),
        Some("000102")
    );
    assert_eq!(report.supported_standards[0].name, "ICRC-1");
    assert_eq!(report.metadata[0].key, "icrc1:name");
    assert!(report.metadata.iter().any(|row| row.key == "icrc1:logo"
        && row.value_type == "bool"
        && row.value == serde_json::json!(true)));
    assert!(text.contains("token_symbol: FIX"));
    assert!(text.contains("transfer_fee: 0.00"));
    assert!(text.contains("total_supply: 10.00"));
    assert!(!text.contains("10_000"));
    assert!(!text.contains("1_000_000_000"));
    assert!(text.contains("ledger_index_canister_id: bw4dl-smaaa-aaaaa-qaacq-cai"));
    assert!(text.contains("ICRC-1"));
    assert!(text.contains("icrc1:name"));
    assert!(text.contains("icrc1:fee"));
    assert!(text.contains("icrc1:logo"));
    assert!(text.contains("true"));
    assert!(!text.contains("data:image"));
}

#[test]
fn sns_token_logo_metadata_is_presence_boolean() {
    let row = metadata_row(
        SNS_TOKEN_LOGO_METADATA_KEY.to_string(),
        IcrcMetadataValue::Text("data:image/png;base64,large-logo".to_string()),
    );

    assert_eq!(row.key, SNS_TOKEN_LOGO_METADATA_KEY);
    assert_eq!(row.value_type, "bool");
    assert_eq!(row.value, serde_json::json!(true));
}

#[test]
fn sns_token_empty_logo_metadata_is_false() {
    let row = metadata_row(
        SNS_TOKEN_LOGO_METADATA_KEY.to_string(),
        IcrcMetadataValue::Text(" ".to_string()),
    );

    assert_eq!(row.value_type, "bool");
    assert_eq!(row.value, serde_json::json!(false));
}

#[test]
fn sns_params_resolves_list_id_and_renders_governance_parameters() {
    let request = params_request("1");

    let report = build_sns_params_report_with_source(&request, &FixtureSnsParamsSource)
        .expect("sns params report");
    let text = sns_params_report_text(&report);

    assert_eq!(report.schema_version, SNS_PARAMS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(
        report.parameters.neuron_minimum_stake_e8s,
        Some(100_000_000)
    );
    assert_eq!(report.parameters.transaction_fee_e8s, Some(10_000));
    assert_eq!(
        report
            .parameters
            .voting_rewards_parameters
            .as_ref()
            .and_then(|rewards| rewards.initial_reward_rate_basis_points),
        Some(1000)
    );
    assert!(text.contains("governance_canister_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("neuron_minimum_stake"));
    assert!(text.contains("transaction_fee"));
    assert!(text.contains("max_dissolve_delay"));
    assert!(text.contains("voting_reward_initial_rate"));
    assert!(text.contains("automatically_advance_target_version"));
    assert!(text.contains("1.00"));
    assert!(text.contains("0.00"));
    assert!(text.contains("2922d"));
    assert!(text.contains("10.00%"));
    assert!(text.contains("yes"));
    assert!(text.contains("1,2,3"));
}

#[test]
fn sns_neurons_resolves_list_id_and_renders_governance_neurons() {
    let mut request = neurons_request("1");
    request.owner_principal_id = Some(GOVERNANCE_A.to_string());

    let report = build_sns_neurons_report_with_source(&request, &FixtureSnsNeuronsSource)
        .expect("sns neurons report");
    let text = sns_neurons_report_text(&report);

    assert_eq!(report.schema_version, SNS_NEURONS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.requested_limit, 10);
    assert_eq!(report.owner_principal_id.as_deref(), Some(GOVERNANCE_A));
    assert_eq!(report.neuron_count, 1);
    assert_eq!(report.neurons[0].neuron_id, "0001020304");
    assert_eq!(report.neurons[0].cached_neuron_stake_e8s, 123);
    assert_eq!(report.neurons[0].maturity_e8s_equivalent, 456);
    assert_eq!(report.neurons[0].staked_maturity_e8s_equivalent, Some(789));
    assert_eq!(report.neurons[0].created_at, "2026-06-01T00:00:00Z");
    assert!(text.contains("governance_canister_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("requested_limit: 10"));
    assert!(text.contains("owner_principal_id: bkyz2-fmaaa-aaaaa-qaaaq-cai"));
    assert!(text.contains("00010203"));
    assert!(!text.contains("0001020304"));
    assert!(text.contains("STAKE"));
    assert!(text.contains("MATURITY"));
    assert!(text.contains("STAKED_MATURITY"));
    assert!(!text.contains("STAKE_E8S"));
    assert!(!text.contains("MATURITY_E8S"));
    assert!(text.contains("0.00"));
    assert!(text.contains("2026-06-01T00:00:00Z"));
}

#[test]
fn sns_neurons_text_formats_optional_e8s_as_token_decimals() {
    assert_eq!(text::optional_e8s_decimal_text(None), "-");
    assert_eq!(text::optional_e8s_decimal_text(Some(50_000_000)), "0.50");
}

#[test]
fn sns_neurons_verbose_text_keeps_full_neuron_ids() {
    let mut request = neurons_request("1");
    request.owner_principal_id = Some(GOVERNANCE_A.to_string());
    request.verbose = true;

    let report = build_sns_neurons_report_with_source(&request, &FixtureSnsNeuronsSource)
        .expect("sns neurons report");
    let text = sns_neurons_report_text(&report);

    assert!(report.verbose);
    assert!(text.contains("verbose: yes"));
    assert!(text.contains("0001020304"));
}

#[test]
fn sns_neurons_refresh_writes_complete_cache_and_cached_sort_uses_it() {
    let root = temp_dir("ic-query-sns-neurons-refresh");
    let request = SnsNeuronsRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: "1".to_string(),
        icp_root: root.clone(),
        page_size: 2,
        max_pages: None,
    };

    let refresh = refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect("refresh neurons");
    let cache_path = sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A);
    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    let lock_path = sns_neurons_refresh_lock_path(&root, MAINNET_NETWORK, ROOT_A);

    assert!(cache_path.is_file());
    assert!(attempt_path.is_file());
    assert!(!lock_path.exists());
    assert!(refresh.complete);
    assert_eq!(refresh.page_count, 3);
    assert_eq!(refresh.neuron_count, 3);

    let mut cached_request = neurons_request("1");
    cached_request.icp_root = Some(root.clone());
    cached_request.sort = SnsNeuronsSort::Stake;
    cached_request.limit = 2;
    let report =
        build_sns_neurons_report_with_source(&cached_request, &PagedFixtureSnsNeuronsSource)
            .expect("cached neurons report");

    assert_eq!(report.data_source, "cache");
    assert_eq!(report.sort, "stake");
    assert_eq!(report.total_neuron_count, 3);
    assert_eq!(report.neuron_count, 2);
    assert_eq!(report.neurons[0].neuron_id, "03");
    assert_eq!(report.neurons[0].cached_neuron_stake_e8s, 50);
    assert_eq!(report.neurons[1].neuron_id, "02");
    assert_eq!(report.neurons[1].cached_neuron_stake_e8s, 30);

    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read attempt"))
            .expect("parse attempt");
    assert_eq!(attempt["status"], "complete");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_neurons_refresh_max_pages_does_not_publish_incomplete_cache() {
    let root = temp_dir("ic-query-sns-neurons-incomplete-refresh");
    let request = SnsNeuronsRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: "1".to_string(),
        icp_root: root.clone(),
        page_size: 2,
        max_pages: Some(1),
    };

    let err = refresh_sns_neurons_cache_with_source(&request, &PagedFixtureSnsNeuronsSource)
        .expect_err("incomplete refresh");

    assert!(matches!(
        err,
        SnsHostError::IncompleteRefresh {
            pages_fetched: 1,
            rows_fetched: 2,
            ..
        }
    ));
    assert!(!sns_neurons_cache_path(&root, MAINNET_NETWORK, ROOT_A).exists());
    let attempt_path = sns_neurons_refresh_attempt_path(&root, MAINNET_NETWORK, ROOT_A);
    assert!(attempt_path.is_file());

    let attempt: serde_json::Value =
        serde_json::from_slice(&fs::read(attempt_path).expect("read attempt"))
            .expect("parse attempt");
    assert_eq!(attempt["status"], "failed");
    assert_eq!(attempt["pages_fetched"], 1);
    assert_eq!(attempt["rows_fetched"], 2);
    assert_eq!(attempt["last_cursor"], "02");
    assert!(
        attempt["last_error"]
            .as_str()
            .expect("last error")
            .contains("max pages reached before API exhaustion")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_list_ids_follow_source_order() {
    let request = list_request(false);

    let report = build_sns_list_report_with_source(&request, &UnsortedFixtureSnsListSource)
        .expect("sns list report");
    let info = build_sns_info_report_with_source(&info_request("1"), &UnsortedFixtureSnsListSource)
        .expect("sns info report");

    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "A Name");
    assert_eq!(report.sns_instances[0].root_canister_id, ROOT_A);
    assert_eq!(report.sns_instances[1].id, 2);
    assert_eq!(report.sns_instances[1].name, "Z Name");
    assert_eq!(report.sns_instances[1].root_canister_id, ROOT_B);
    assert_eq!(info.id, 1);
    assert_eq!(info.root_canister_id, ROOT_A);
}

#[test]
fn sns_list_name_sort_keeps_stable_ids() {
    let mut request = list_request(false);
    request.sort = SnsListSort::Name;

    let report = build_sns_list_report_with_source(&request, &UnsortedFixtureSnsListSource)
        .expect("sns list report");
    let text = sns_list_report_text(&report);
    let info = build_sns_info_report_with_source(&info_request("1"), &UnsortedFixtureSnsListSource)
        .expect("sns info report");

    assert_eq!(report.sort, "name");
    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "A Name");
    assert_eq!(report.sns_instances[1].id, 2);
    assert_eq!(report.sns_instances[1].name, "Z Name");
    assert!(text.contains("sort: name"));
    assert_eq!(info.id, 1);
    assert_eq!(info.root_canister_id, ROOT_A);
}

#[test]
fn sns_list_surfaces_metadata_fallbacks() {
    let request = list_request(true);

    let report = build_sns_list_report_with_source(&request, &MetadataErrorFixtureSnsListSource)
        .expect("sns list report");
    let text = sns_list_report_text(&report);
    let info =
        build_sns_info_report_with_source(&info_request("1"), &MetadataErrorFixtureSnsListSource)
            .expect("sns info report");
    let info_text = sns_info_report_text(&info);

    assert_eq!(report.metadata_error_count, 1);
    assert_eq!(
        report.sns_instances[0].metadata_error.as_deref(),
        Some("get_metadata: Canister has no Wasm module")
    );
    assert!(text.contains("metadata_errors: 1"));
    assert!(text.contains("metadata_error_details:"));
    assert!(text.contains("get_metadata: Canister has no Wasm module"));
    assert_eq!(
        info.metadata_error.as_deref(),
        Some("get_metadata: Canister has no Wasm module")
    );
    assert!(info_text.contains("metadata_error: get_metadata: Canister has no Wasm module"));
}

#[test]
fn sns_info_rejects_unknown_id() {
    let request = info_request("2");

    let err =
        build_sns_info_report_with_source(&request, &FixtureSnsListSource).expect_err("unknown id");

    assert!(matches!(
        err,
        SnsHostError::UnknownSnsId {
            id: 2,
            sns_count: 1
        }
    ));
}

#[test]
fn sns_list_rejects_local_network() {
    let request = SnsListRequest {
        network: "local".to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        verbose: false,
        sort: SnsListSort::Id,
    };

    let err = build_sns_list_report_with_source(&request, &FixtureSnsListSource)
        .expect_err("local rejected");

    assert!(matches!(err, SnsHostError::UnsupportedNetwork { .. }));
}

fn list_request(verbose: bool) -> SnsListRequest {
    SnsListRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        verbose,
        sort: SnsListSort::Id,
    }
}

fn info_request(input: &str) -> SnsInfoRequest {
    SnsInfoRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
    }
}

fn token_request(input: &str) -> SnsTokenRequest {
    SnsTokenRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
    }
}

fn params_request(input: &str) -> SnsParamsRequest {
    SnsParamsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
    }
}

fn neurons_request(input: &str) -> SnsNeuronsRequest {
    SnsNeuronsRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        input: input.to_string(),
        limit: 10,
        owner_principal_id: None,
        sort: SnsNeuronsSort::Api,
        icp_root: None,
        verbose: false,
    }
}

struct FixtureSnsParamsSource;

impl SnsListSource for FixtureSnsParamsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsParamsSource for FixtureSnsParamsSource {
    fn fetch_sns_params(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        Ok(SnsGovernanceParameters {
            max_dissolve_delay_seconds: Some(252_460_800),
            max_dissolve_delay_bonus_percentage: Some(100),
            max_followees_per_function: Some(15),
            neuron_claimer_permissions: Some(SnsNeuronPermissionList {
                permissions: vec![1, 2, 3],
            }),
            neuron_minimum_stake_e8s: Some(100_000_000),
            max_neuron_age_for_age_bonus: Some(126_144_000),
            initial_voting_period_seconds: Some(345_600),
            neuron_minimum_dissolve_delay_to_vote_seconds: Some(2_592_000),
            reject_cost_e8s: Some(100_000_000),
            max_proposals_to_keep_per_action: Some(100),
            wait_for_quiet_deadline_increase_seconds: Some(86_400),
            max_number_of_neurons: Some(200_000),
            transaction_fee_e8s: Some(10_000),
            max_number_of_proposals_with_ballots: Some(700),
            max_age_bonus_percentage: Some(25),
            neuron_grantable_permissions: Some(SnsNeuronPermissionList {
                permissions: vec![4, 5],
            }),
            voting_rewards_parameters: Some(SnsVotingRewardsParameters {
                final_reward_rate_basis_points: Some(500),
                initial_reward_rate_basis_points: Some(1000),
                reward_rate_transition_duration_seconds: Some(189_216_000),
                round_duration_seconds: Some(86_400),
            }),
            maturity_modulation_disabled: Some(false),
            max_number_of_principals_per_neuron: Some(10),
            automatically_advance_target_version: Some(true),
            custom_proposal_criticality: Some(SnsCustomProposalCriticality {
                additional_critical_native_action_ids: vec![1, 2],
            }),
        })
    }
}

struct FixtureSnsListSource;

impl SnsListSource for FixtureSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Ok(MainnetSnsList {
            network: MAINNET_NETWORK.to_string(),
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            sns_instances: vec![fixture_sns(
                "Fixture SNS",
                Some("Fixture description"),
                Some("https://example.com"),
                ROOT_A,
                GOVERNANCE_A,
                LEDGER_A,
                SWAP_A,
                INDEX_A,
                None,
            )],
        })
    }
}

struct UnsortedFixtureSnsListSource;

impl SnsListSource for UnsortedFixtureSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Ok(MainnetSnsList {
            network: MAINNET_NETWORK.to_string(),
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            sns_instances: vec![
                fixture_sns(
                    "A Name",
                    None,
                    None,
                    ROOT_A,
                    GOVERNANCE_A,
                    LEDGER_A,
                    SWAP_A,
                    INDEX_A,
                    None,
                ),
                fixture_sns(
                    "Z Name",
                    None,
                    None,
                    ROOT_B,
                    GOVERNANCE_B,
                    LEDGER_B,
                    SWAP_B,
                    INDEX_B,
                    None,
                ),
            ],
        })
    }
}

struct MetadataErrorFixtureSnsListSource;

impl SnsListSource for MetadataErrorFixtureSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Ok(MainnetSnsList {
            network: MAINNET_NETWORK.to_string(),
            sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            sns_instances: vec![fixture_sns(
                "unnamed-be2us",
                None,
                None,
                ROOT_A,
                GOVERNANCE_A,
                LEDGER_A,
                SWAP_A,
                INDEX_A,
                Some("get_metadata: Canister has no Wasm module"),
            )],
        })
    }
}

struct FixtureSnsTokenSource;

impl SnsListSource for FixtureSnsTokenSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsTokenSource for FixtureSnsTokenSource {
    fn fetch_sns_token(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        assert_eq!(sns.ledger_canister_id, LEDGER_A);
        Ok(MainnetSnsToken {
            token_name: "Fixture Token".to_string(),
            token_symbol: "FIX".to_string(),
            decimals: 8,
            transfer_fee: "10_000".to_string(),
            total_supply: "1_000_000_000".to_string(),
            minting_account_owner: Some(GOVERNANCE_A.to_string()),
            minting_account_subaccount_hex: Some("000102".to_string()),
            ledger_index_canister_id: Some(INDEX_A.to_string()),
            ledger_index_error: None,
            supported_standards: vec![
                SnsTokenStandardRow {
                    name: "ICRC-1".to_string(),
                    url: "https://github.com/dfinity/ICRC-1".to_string(),
                },
                SnsTokenStandardRow {
                    name: "ICRC-2".to_string(),
                    url: "https://github.com/dfinity/ICRC-2".to_string(),
                },
            ],
            metadata: vec![
                SnsTokenMetadataRow {
                    key: "icrc1:name".to_string(),
                    value_type: "text".to_string(),
                    value: serde_json::json!("Fixture Token"),
                },
                SnsTokenMetadataRow {
                    key: "icrc1:decimals".to_string(),
                    value_type: "nat".to_string(),
                    value: serde_json::json!("8"),
                },
                SnsTokenMetadataRow {
                    key: "icrc1:fee".to_string(),
                    value_type: "nat".to_string(),
                    value: serde_json::json!("10_000"),
                },
                SnsTokenMetadataRow {
                    key: "icrc1:logo".to_string(),
                    value_type: "bool".to_string(),
                    value: serde_json::json!(true),
                },
            ],
        })
    }
}

struct FixtureSnsNeuronsSource;

impl SnsListSource for FixtureSnsNeuronsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsNeuronsSource for FixtureSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert_eq!(owner_principal_id, Some(GOVERNANCE_A));
        Ok(MainnetSnsNeurons {
            neurons: vec![SnsNeuronRow {
                neuron_id: "0001020304".to_string(),
                cached_neuron_stake_e8s: 123,
                maturity_e8s_equivalent: 456,
                staked_maturity_e8s_equivalent: Some(789),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
            }],
        })
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert!(start_page_at.is_none());
        assert_eq!(owner_principal_id, None);
        Ok(MainnetSnsNeuronPage {
            neurons: vec![SnsNeuronRow {
                neuron_id: "0001020304".to_string(),
                cached_neuron_stake_e8s: 123,
                maturity_e8s_equivalent: 456,
                staked_maturity_e8s_equivalent: Some(789),
                created_timestamp_seconds: 1_780_272_000,
                created_at: "2026-06-01T00:00:00Z".to_string(),
            }],
            last_cursor: Some(SnsNeuronId {
                id: vec![0, 1, 2, 3],
            }),
        })
    }
}

struct PagedFixtureSnsNeuronsSource;

impl SnsListSource for PagedFixtureSnsNeuronsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsNeuronsSource for PagedFixtureSnsNeuronsSource {
    fn fetch_sns_neurons(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        unreachable!("paged fixture is only used by complete cache refresh tests")
    }

    fn fetch_sns_neuron_page(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 2);
        assert_eq!(owner_principal_id, None);
        let cursor = start_page_at.map(|cursor| cursor.id.as_slice());
        let (neurons, last_cursor) = match cursor {
            None => (
                vec![neuron_row("01", 10), neuron_row("02", 30)],
                Some(vec![2]),
            ),
            Some([2]) => (
                vec![neuron_row("02", 30), neuron_row("03", 50)],
                Some(vec![3]),
            ),
            Some([3]) => (vec![neuron_row("03", 50)], Some(vec![3])),
            Some(other) => panic!("unexpected cursor {other:?}"),
        };
        Ok(MainnetSnsNeuronPage {
            neurons,
            last_cursor: last_cursor.map(|id| SnsNeuronId { id }),
        })
    }
}

fn neuron_row(neuron_id: &str, stake: u64) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: neuron_id.to_string(),
        cached_neuron_stake_e8s: stake,
        maturity_e8s_equivalent: stake / 2,
        staked_maturity_e8s_equivalent: None,
        created_timestamp_seconds: 1_780_272_000 + stake,
        created_at: format_utc_timestamp_secs(1_780_272_000 + stake),
    }
}

#[allow(clippy::too_many_arguments)]
fn fixture_sns(
    name: &str,
    description: Option<&str>,
    url: Option<&str>,
    root_canister_id: &str,
    governance_canister_id: &str,
    ledger_canister_id: &str,
    swap_canister_id: &str,
    index_canister_id: &str,
    metadata_error: Option<&str>,
) -> MainnetSns {
    MainnetSns {
        id: 0,
        name: name.to_string(),
        description: description.map(str::to_string),
        url: url.map(str::to_string),
        root_canister_id: root_canister_id.to_string(),
        governance_canister_id: governance_canister_id.to_string(),
        ledger_canister_id: ledger_canister_id.to_string(),
        swap_canister_id: swap_canister_id.to_string(),
        index_canister_id: index_canister_id.to_string(),
        metadata_error: metadata_error.map(str::to_string),
    }
}
