#[cfg(feature = "sns-host")]
use ic_query::HostCacheError;
use ic_query::icrc::IcrcMetadataValueKind;
use ic_query::icrc::IcrcTokenMetadataRow;
use ic_query::icrc::IcrcTokenStandardRow;
#[cfg(feature = "sns-host")]
use ic_query::report::CollectionSourceRequest;
use ic_query::report::ReportDataSource;
use ic_query::report::SortDirection;
#[cfg(feature = "sns-host")]
use ic_query::sns::DEFAULT_SNS_SOURCE_ENDPOINT;
#[cfg(feature = "sns-host")]
use ic_query::sns::MainnetSns;
use ic_query::sns::MainnetSnsCanisters;
#[cfg(feature = "sns-host")]
use ic_query::sns::MainnetSnsInventory;
#[cfg(feature = "sns-host")]
use ic_query::sns::MainnetSnsMetadata;
use ic_query::sns::MainnetSnsMetrics;
#[cfg(feature = "sns-host")]
use ic_query::sns::MainnetSnsNeuronPage;

use ic_query::sns::MainnetSnsSwap;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCacheListRequest;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCacheStatusRequest;
use ic_query::sns::SnsCanisterCallType;
use ic_query::sns::SnsCanisterCycleBalanceStatus;
use ic_query::sns::SnsCanisterGapKind;
use ic_query::sns::SnsCanisterHealthQueryGap;
use ic_query::sns::SnsCanisterMethod;
use ic_query::sns::SnsCanisterReport;
use ic_query::sns::SnsCanisterRole;
use ic_query::sns::SnsCanisterRow;
use ic_query::sns::SnsCanisterStatus;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCatalogCacheRequest;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCatalogRefreshReport;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsCatalogRefreshRequest;
use ic_query::sns::SnsCustomProposalCriticality;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsDiscoverySource;
use ic_query::sns::SnsGovernanceParameters;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsHostError;
use ic_query::sns::SnsInfoReport;
use ic_query::sns::SnsListReport;
use ic_query::sns::SnsListRequest;
use ic_query::sns::SnsListSort;
use ic_query::sns::SnsLookupRequest;
use ic_query::sns::SnsMaturityDisbursementRow;
use ic_query::sns::SnsMetricsReport;
use ic_query::sns::SnsNeuronAccount;
use ic_query::sns::SnsNeuronDetail;
use ic_query::sns::SnsNeuronDetailReport;
use ic_query::sns::SnsNeuronDissolveState;
use ic_query::sns::SnsNeuronFolloweeRow;
use ic_query::sns::SnsNeuronFolloweesRow;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsNeuronId;
use ic_query::sns::SnsNeuronPermissionList;
use ic_query::sns::SnsNeuronPermissionRow;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsNeuronRequest;
use ic_query::sns::SnsNeuronRow;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsNeuronSource;
use ic_query::sns::SnsNeuronTopicFolloweesRow;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsNeuronsReport;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsNeuronsRequest;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsNeuronsSource;
use ic_query::sns::SnsParamsReport;
use ic_query::sns::SnsPendingUpgrade;
use ic_query::sns::SnsPolicyObservationStatus;
use ic_query::sns::SnsProposalBallotRow;
use ic_query::sns::SnsProposalEligibilityFilter;
use ic_query::sns::SnsProposalFailureReason;
use ic_query::sns::SnsProposalReport;
use ic_query::sns::SnsProposalRequest;
use ic_query::sns::SnsProposalRow;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsProposalSource;
use ic_query::sns::SnsProposalStatusFilter;
use ic_query::sns::SnsProposalTally;
use ic_query::sns::SnsProposalTopicFilter;
use ic_query::sns::SnsProposalsReport;
use ic_query::sns::SnsProposalsRequest;
use ic_query::sns::SnsProposalsSort;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsProposalsSource;

#[cfg(feature = "sns-host")]
use ic_query::sns::SnsSnapshotRefreshReport;
#[cfg(feature = "sns-host")]
use ic_query::sns::SnsSnapshotRefreshRequest;
use ic_query::sns::SnsSwapComponent;
use ic_query::sns::SnsSwapDerivedState;
use ic_query::sns::SnsSwapLifecycle;
use ic_query::sns::SnsSwapNeuronBasketConstructionParameters;
use ic_query::sns::SnsSwapQueryGap;
use ic_query::sns::SnsSwapReport;
use ic_query::sns::SnsSwapSaleParameters;
use ic_query::sns::SnsTokenReport;
use ic_query::sns::SnsTreasuryMetricRow;
use ic_query::sns::SnsUpgradeQueryGap;
use ic_query::sns::SnsUpgradeReport;
use ic_query::sns::SnsVersion;
use ic_query::sns::SnsVotingPowerMetrics;
use ic_query::sns::SnsVotingRewardsParameters;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_list_report;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_list_report_from_cache;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_list_report_from_cache_or_refresh;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_neuron_detail_report_with_source;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_neurons_cache_list_report;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_neurons_cache_status_report;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_neurons_report_with_source;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_proposal_report_with_source;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_proposals_cache_list_report;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_proposals_cache_status_report;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_proposals_report;
#[cfg(feature = "sns-host")]
use ic_query::sns::build_sns_proposals_report_with_source_and_progress;

#[cfg(feature = "sns-host")]
use ic_query::sns::refresh_sns_neurons_cache;
#[cfg(feature = "sns-host")]
use ic_query::sns::refresh_sns_neurons_cache_with_source_and_progress;
#[cfg(feature = "sns-host")]
use ic_query::sns::refresh_sns_proposals_cache;
#[cfg(feature = "sns-host")]
use ic_query::sns::refresh_sns_proposals_cache_with_source_and_progress;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_catalog_cache_path;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_catalog_refresh_lock_path;
use ic_query::sns::sns_neuron_permission_name;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_neurons_cache_path;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_neurons_refresh_attempt_path;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_neurons_refresh_lock_path;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_proposals_cache_path;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_proposals_refresh_attempt_path;
#[cfg(feature = "sns-host")]
use ic_query::sns::sns_proposals_refresh_lock_path;

use ic_query_text::sns::sns_canister_report_text;
#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_catalog_refresh_report_text;
use ic_query_text::sns::sns_info_report_text;
use ic_query_text::sns::sns_list_report_text;
use ic_query_text::sns::sns_metrics_report_text;
use ic_query_text::sns::sns_neuron_detail_report_text;
#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_neurons_cache_list_report_text;
#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_neurons_cache_status_report_text;
#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_neurons_report_text;
use ic_query_text::sns::sns_params_report_text;
use ic_query_text::sns::sns_proposal_report_text;
#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_proposals_cache_list_report_text;
#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_proposals_cache_status_report_text;
use ic_query_text::sns::sns_proposals_report_text;

#[cfg(feature = "sns-host")]
use ic_query_text::sns::sns_snapshot_refresh_report_text;
use ic_query_text::sns::sns_swap_report_text;
use ic_query_text::sns::sns_token_report_text;
use ic_query_text::sns::sns_upgrade_report_text;
use serde_json::json;
#[cfg(feature = "sns-host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

const SAMPLE_SNS_ROOT_CANISTER_ID: &str = "be2us-64aaa-aaaaa-qaabq-cai";
const SAMPLE_SNS_GOVERNANCE_CANISTER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";
const SAMPLE_SNS_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
const SAMPLE_SNS_SWAP_CANISTER_ID: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
const SAMPLE_SNS_INDEX_CANISTER_ID: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";
const SAMPLE_SNS_FETCHED_AT: &str = "2023-11-14T22:13:20Z";
const SAMPLE_SNS_NEURON_ID: &str =
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[cfg(feature = "sns-host")]
type SnsBuilder<Request, Report> =
    for<'a> fn(&'a Request) -> ic_query::report::SourceFuture<'a, Result<Report, SnsHostError>>;

#[test]
fn public_sns_neuron_detail_models_are_constructible_and_renderable() {
    let report = SnsNeuronDetailReport {
        schema_version: 1,
        context: sample_sns_context(),

        neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
        data_source: ReportDataSource::Live,
        detail: sample_sns_neuron_detail(),
    };

    assert_eq!(sns_neuron_permission_name(7), "merge_maturity");
    assert_eq!(sns_neuron_permission_name(99), "unknown");
    assert_eq!(
        report.detail.derived_policy_observations().0,
        SnsPolicyObservationStatus::Violated
    );
    assert!(sns_neuron_detail_report_text(&report).contains("7:merge_maturity"));
    let value = serde_json::to_value(&report).expect("detail JSON");
    assert_eq!(value["neuron_id"], SAMPLE_SNS_NEURON_ID);
    assert_eq!(value["network"], "ic");
    assert_eq!(
        value["governance_canister_id"],
        SAMPLE_SNS_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(value.get("context"), None);
    assert_eq!(
        value["detail"]["permissions"][0]["permission_types"],
        serde_json::json!([7])
    );
    for invalid_codes in [
        serde_json::json!([{"code": 7, "name": "merge_maturity"}]),
        serde_json::json!(["7"]),
        serde_json::json!([2_147_483_648_u64]),
    ] {
        let mut invalid = value.clone();
        invalid["detail"]["permissions"][0]["permission_types"] = invalid_codes;
        assert!(serde_json::from_value::<SnsNeuronDetailReport>(invalid).is_err());
    }
    let decoded: SnsNeuronDetailReport =
        serde_json::from_value(value.clone()).expect("detail round trip");
    assert_eq!(decoded, report);
    let duplicate = serde_json::to_string(&report).unwrap().replace(
        "\"root_canister_id\":",
        "\"root_canister_id\":\"aaaaa-aa\",\"root_canister_id\":",
    );
    assert!(serde_json::from_str::<SnsNeuronDetailReport>(&duplicate).is_err());
    let mut invalid = value;
    invalid["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SnsNeuronDetailReport>(invalid).is_err());
}

#[test]
fn public_sns_list_api_is_constructible_and_renderable() {
    let request = SnsListRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        all_lifecycles: false,

        sort: SnsListSort::Id,
    };

    assert_eq!(request.sort.as_str(), "id");

    let mut report = SnsListReport {
        provenance: ic_query::report::ReportProvenance::Live,
        schema_version: 1,
        network: request.network.clone(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        all_lifecycles: request.all_lifecycles,

        sort: request.sort.as_str().to_string(),
        catalog_sns_count: 0,
        excluded_sns_count: 0,
        metadata_error_count: 0,
        lifecycle_error_count: 0,
        sns_instances: Vec::new(),
    };

    let text = sns_list_report_text(&report, ic_query_text::TextOptions::default());

    assert!(text.contains("network: ic"));
    assert!(text.contains("sns_count: 0"));
    let failed = ic_query::sns::SnsListRow {
        id: 1,
        name: "error\n\u{1b}[31m".to_string(),
        canisters: MainnetSnsCanisters {
            root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
            governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
            ledger_canister_id: SAMPLE_SNS_LEDGER_CANISTER_ID.to_string(),
            swap_canister_id: SAMPLE_SNS_SWAP_CANISTER_ID.to_string(),
            index_canister_id: SAMPLE_SNS_INDEX_CANISTER_ID.to_string(),
        },
        metadata_error: Some("metadata unavailable".to_string()),
        lifecycle: ic_query::sns::SnsLifecycleOutcome::Value { code: 2 },
    };
    let mut ready = failed.clone();
    ready.id = 2;
    ready.metadata_error = None;
    report.sns_instances = vec![failed, ready];

    report.catalog_sns_count = 2;
    report.metadata_error_count = 1;
    let plain = sns_list_report_text(&report, ic_query_text::TextOptions::default());
    let mut statuses = Vec::new();
    let styled = ic_query_text::sns::sns_list_report_text_with_metadata_style(
        &report,
        ic_query_text::TextOptions::default(),
        |status, failed| {
            statuses.push((status.to_string(), failed));
            format!("\u{1b}[{}m{status}\u{1b}[0m", if failed { 31 } else { 32 })
        },
    );
    assert_eq!(
        statuses,
        [("error".to_string(), true), ("ok".to_string(), false)]
    );
    let stripped = styled
        .replace("\u{1b}[31m", "")
        .replace("\u{1b}[32m", "")
        .replace("\u{1b}[0m", "");
    assert_eq!(stripped, plain);
    assert!(styled.contains("error\\n\\u{1b}[31m"));
}

#[test]
fn public_sns_info_api_is_constructible_and_renderable() {
    let request = SnsLookupRequest::new(
        "ic".to_string(),
        "https://icp-api.io".to_string(),
        1_700_000_000,
        "1".to_string(),
    )
    .expect("valid SNS selector");

    let report = SnsInfoReport {
        schema_version: 1,
        network: request.network.clone(),
        sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        id: 1,
        name: "Example SNS".to_string(),
        description: Some("Example description".to_string()),
        url: None,
        canisters: ic_query::sns::MainnetSnsCanisters {
            root_canister_id: "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
            governance_canister_id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".to_string(),
            ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
            swap_canister_id: "br5f7-7uaaa-aaaaa-qaaca-cai".to_string(),
            index_canister_id: "bw4dl-smaaa-aaaaa-qaacq-cai".to_string(),
        },
        metadata_error: None,
    };

    let json = serde_json::to_value(&report).expect("serialize SNS identity");
    let identity = serde_json::to_value(&report.canisters).expect("serialize canonical identity");
    for (key, value) in identity.as_object().expect("identity object") {
        assert_eq!(&json[key], value);
    }
    let decoded: ic_query::sns::MainnetSnsCanisters =
        serde_json::from_value(identity).expect("read canonical identity");
    assert_eq!(decoded, report.canisters);

    assert_eq!(request.input().to_string(), "1");

    let text = sns_info_report_text(&report);

    assert!(text.contains("sns_id: 1"));
    assert!(text.contains("description: Example description"));
    assert!(text.contains("url: -"));
}

#[test]
fn public_sns_metrics_api_is_constructible_and_renderable() {
    let report = SnsMetricsReport {
        schema_version: 1,
        context: sample_sns_context().instance,

        metrics: MainnetSnsMetrics {
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
            treasury_metrics: vec![SnsTreasuryMetricRow {
                treasury: 1,

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
        },
    };

    let json = serde_json::to_value(&report).expect("serialize public SNS metrics report");
    assert_instance_json_fields(&json, &report.context);
    let text = sns_metrics_report_text(&report);

    assert_eq!(json["treasury_metrics"][0]["treasury"], 1);
    assert_eq!(json["treasury_metrics"][0]["treasury"], 1);
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
        context: sample_sns_context().instance,
        health_status_count: 1,
        reported_zero_cycles_count: 0,
        cycles_unavailable_count: 0,
        inventory: ic_query::sns::MainnetSnsCanisterInventory {
            inventory_method: SnsCanisterMethod::ListSnsCanisters,
            health_method: SnsCanisterMethod::GetSnsCanistersSummary,
            health_call_type: SnsCanisterCallType::IngressUpdate,
            health_update_canister_list: false,
            point_in_time_guaranteed: false,
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
        },
    };

    let json = serde_json::to_value(&report).expect("serialize public SNS canister report");
    assert_instance_json_fields(&json, &report.context);
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
    let request = SnsLookupRequest::new(
        "ic".to_string(),
        "https://icp-api.io".to_string(),
        1_700_000_000,
        "be2us-64aaa-aaaaa-qaabq-cai".to_string(),
    )
    .expect("valid SNS selector");

    let report = SnsTokenReport {
        schema_version: 1,
        context: ic_query::sns::SnsInstanceContext {
            network: request.network.clone(),
            fetched_at: "2023-11-14T22:13:20Z".to_string(),
            source_endpoint: request.source_endpoint.clone(),
            root_canister_id: request.input().to_string(),
            ..sample_sns_context().instance
        },
        ledger_canister_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        sns_index_canister_id: "bw4dl-smaaa-aaaaa-qaacq-cai".to_string(),
        ledger_index: ic_query::icrc::IcrcIndexData::Unavailable("not configured".to_string()),
        token: ic_query::icrc::IcrcTokenData {
            token_name: "Example Token".to_string(),
            token_symbol: "EXT".to_string(),
            decimals: 8,
            transfer_fee: "100_000_000".to_string(),
            total_supply: "1_000_000_000".to_string(),
            minting_account_owner: Some("aaaaa-aa".to_string()),
            minting_account_subaccount_hex: None,
            supported_standards: vec![IcrcTokenStandardRow {
                name: "ICRC-1".to_string(),
                url: "https://github.com/dfinity/ICRC-1".to_string(),
            }],
            metadata: vec![IcrcTokenMetadataRow {
                key: "icrc1:symbol".to_string(),
                value_type: IcrcMetadataValueKind::Text,
                value: json!("EXT"),
            }],
        },
    };

    let text = sns_token_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize SNS token report");
    assert_instance_json_fields(&json, &report.context);

    assert!(text.contains("token_symbol: EXT"));
    assert!(text.contains("transfer_fee: 1.00"));
    assert!(text.contains("total_supply: 10.00"));
    assert_eq!(json["transfer_fee"], "100_000_000");
    assert_eq!(json["total_supply"], "1_000_000_000");
    assert!(text.contains("ledger_index_error: not configured"));
    assert!(text.contains("ICRC-1"));
}

#[test]
fn public_sns_params_api_is_constructible_and_renderable() {
    let request = SnsLookupRequest::new(
        "ic".to_string(),
        "https://icp-api.io".to_string(),
        1_700_000_000,
        "1".to_string(),
    )
    .expect("valid SNS selector");

    let report = SnsParamsReport {
        schema_version: 1,
        context: ic_query::sns::SnsReportContext {
            instance: ic_query::sns::SnsInstanceContext {
                network: request.network.clone(),
                source_endpoint: request.source_endpoint,
                ..sample_sns_context().instance
            },
            ..sample_sns_context()
        },

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
        context: sample_sns_context().instance,
        component_query_count: 3,
        successful_component_query_count: 3,
        component_gap_count: 0,

        swap: MainnetSnsSwap {
            swap_canister_id: "br5f7-7uaaa-aaaaa-qaaca-cai".to_string(),
            lifecycle_method: SnsCanisterMethod::GetLifecycle,
            sale_parameters_method: SnsCanisterMethod::GetSaleParameters,
            derived_state_method: SnsCanisterMethod::GetDerivedState,
            point_in_time_guaranteed: false,
            lifecycle: Some(SnsSwapLifecycle {
                lifecycle: Some(2),

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
        },
    };
    let gap = SnsSwapQueryGap {
        component: SnsSwapComponent::DerivedState,
        method: SnsCanisterMethod::GetDerivedState,
        reason: "fixture rejection".to_string(),
    };

    let text = sns_swap_report_text(&report);
    let json = serde_json::to_value(&report).expect("serialize swap report");
    assert_instance_json_fields(&json, &report.context);

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
        context: sample_sns_context(),
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
    for (key, expected) in serde_json::to_value(&report.context)
        .unwrap()
        .as_object()
        .unwrap()
    {
        assert_eq!(
            json.get(key),
            Some(expected),
            "top-level context field {key}"
        );
    }

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
    let mut request = SnsProposalsRequest::new(
        ic_query::sns::SnsLookupRequest::new("ic", "https://icp-api.io", 1_700_000_000, "1")
            .unwrap(),
        10,
    );
    request.before_proposal_id = Some(100);
    request = request.with_status(SnsProposalStatusFilter::Any).unwrap();
    request.topic = SnsProposalTopicFilter::Governance;
    request.eligibility = SnsProposalEligibilityFilter::Yes;
    request.proposer_neuron_id = Some("010203".into());
    request.query = Some("upgrade".into());
    request = request.with_sort(SnsProposalsSort::Created);
    request.sort_direction = SortDirection::Desc;

    assert_eq!(request.sort.as_str(), "created");
    assert_eq!(request.topic.as_str(), "governance");

    let report = SnsProposalsReport {
        provenance: ic_query::report::ReportProvenance::Live,
        schema_version: 1,
        context: ic_query::sns::SnsReportContext {
            instance: ic_query::sns::SnsInstanceContext {
                network: request.lookup.network.clone(),
                source_endpoint: request.lookup.source_endpoint.clone(),
                ..sample_sns_context().instance
            },
            ..sample_sns_context()
        },

        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status().as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        eligibility_filter: request.eligibility.as_str().to_string(),
        proposer_filter: request.proposer_neuron_id,
        query_filter: request.query,
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),

        proposals: vec![sample_sns_proposal_row()],
    };

    let text = sns_proposals_report_text(&report, ic_query_text::TextOptions { verbose: true });
    let json = serde_json::to_value(&report).expect("serialize SNS proposals report");

    assert!(text.contains("proposal_count: 1"));
    assert!(text.contains("topic_filter: governance"));
    assert!(!text.contains("proposal_details:"));
    assert!(text.contains("Upgrade SNS"));
    assert_eq!(json["proposals"][0]["action_id"], 7);
    assert_eq!(json["proposals"][0]["ballots"][0]["vote"], 1);
}

#[test]
fn public_sns_proposal_rows_preserve_unknown_native_codes() {
    let mut row = sample_sns_proposal_row();
    row.action_id = u64::MAX;
    row.ballots[0].vote = -7;
    let value = serde_json::to_value(&row).unwrap();
    assert_eq!(value["action_id"], u64::MAX);
    assert_eq!(value["ballots"][0]["vote"], -7);
    assert_eq!(
        serde_json::from_value::<SnsProposalRow>(value).unwrap(),
        row
    );
}

#[test]
fn public_sns_proposal_api_is_constructible_and_renderable() {
    let request = SnsProposalRequest {
        lookup: ic_query::sns::SnsLookupRequest::new(
            "ic".to_string(),
            "https://icp-api.io".to_string(),
            1_700_000_000,
            "1".to_string(),
        )
        .expect("valid SNS selector"),
        proposal_id: 42,
        cache_root: None,
    };

    let report = SnsProposalReport {
        provenance: ic_query::report::ReportProvenance::Live,
        schema_version: 1,
        context: ic_query::sns::SnsReportContext {
            instance: ic_query::sns::SnsInstanceContext {
                network: request.lookup.network.clone(),
                source_endpoint: request.lookup.source_endpoint.clone(),
                ..sample_sns_context().instance
            },
            ..sample_sns_context()
        },

        proposal_id: request.proposal_id,

        proposal: sample_sns_proposal_row(),
    };

    let text = sns_proposal_report_text(&report, ic_query_text::TextOptions { verbose: false });
    let json = serde_json::to_value(&report).expect("serialize SNS proposal report");

    assert!(text.contains("proposal_id: 42"));
    assert!(!text.contains("show_ballots:"));
    assert!(!text.contains("ballots:"));
    assert!(text.contains("Upgrade SNS"));
    assert_eq!(json["proposal"]["action_id"], 7);
    assert_eq!(json["proposal"]["ballots"][0]["vote"], 1);
}

#[cfg(feature = "sns-host")]
#[test]
fn public_sns_host_api_exposes_catalog_and_collection_entry_points() {
    accepts_public_function::<SnsBuilder<SnsListRequest, SnsListReport>>(|arg0| {
        Box::pin(build_sns_list_report(arg0))
    });
    accepts_public_function::<SnsBuilder<SnsProposalsRequest, SnsProposalsReport>>(|arg0| {
        Box::pin(build_sns_proposals_report(arg0))
    });
    accepts_public_function::<SnsBuilder<SnsSnapshotRefreshRequest, SnsSnapshotRefreshReport>>(
        |arg0| Box::pin(refresh_sns_neurons_cache(arg0)),
    );
    accepts_public_function::<SnsBuilder<SnsSnapshotRefreshRequest, SnsSnapshotRefreshReport>>(
        |arg0| Box::pin(refresh_sns_proposals_cache(arg0)),
    );
}

#[cfg(feature = "sns-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_sns_host_api_accepts_custom_proposal_source_adapters() -> Result<(), SnsHostError> {
    let source = FixtureSnsSource;
    let detail_request = SnsProposalRequest::new(
        ic_query::sns::SnsLookupRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1")
            .expect("valid SNS selector"),
        42,
    );
    let detail = build_sns_proposal_report_with_source(&detail_request, &source).await?;
    assert_eq!(detail.proposal.proposal_id, 42);
    assert_eq!(detail.provenance.data_source().as_str(), "live");

    let mut list_request = SnsProposalsRequest::new(
        ic_query::sns::SnsLookupRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1")
            .expect("valid SNS selector"),
        10,
    );
    list_request.before_proposal_id = Some(99);
    list_request = list_request
        .with_status(SnsProposalStatusFilter::Open)
        .unwrap();
    list_request.topic = SnsProposalTopicFilter::Governance;

    let list =
        build_sns_proposals_report_with_source_and_progress(&list_request, &source, &mut |_| {})
            .await?;
    assert_eq!(list.proposals.len(), 1);
    assert_eq!(list.proposals[0].title, "Upgrade SNS");

    let cache_root = proposal_source_cache_root();
    let _ = fs::remove_dir_all(&cache_root);
    let mut refresh_request = SnsSnapshotRefreshRequest::new(
        cache_root.clone(),
        ic_query::sns::SnsLookupRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1")
            .expect("valid SNS selector"),
        100,
    );
    refresh_request.max_pages = Some(1);

    let refresh = refresh_sns_proposals_cache_with_source_and_progress(
        &refresh_request,
        &source,
        &mut |_| {},
    )
    .await?;
    assert_eq!(refresh.collection, "proposals");
    assert_eq!(refresh.row_count, 1);
    assert!(refresh.complete);
    let _ = fs::remove_dir_all(cache_root);

    Ok(())
}

#[cfg(feature = "sns-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_sns_host_api_accepts_custom_neuron_source_adapters() -> Result<(), SnsHostError> {
    let source = FixtureSnsSource;
    let detail = build_sns_neuron_detail_report_with_source(
        &SnsNeuronRequest::new(
            ic_query::sns::SnsLookupRequest::new(
                "ic",
                DEFAULT_SNS_SOURCE_ENDPOINT,
                1_700_000_000,
                "1",
            )
            .expect("valid SNS selector"),
            SAMPLE_SNS_NEURON_ID,
        ),
        &source,
    )
    .await?;
    assert_eq!(detail.neuron_id, SAMPLE_SNS_NEURON_ID);
    assert_eq!(detail.detail.permissions.len(), 1);

    let report = build_sns_neurons_report_with_source(
        &SnsNeuronsRequest::live(
            ic_query::sns::SnsLookupRequest::new(
                "ic",
                DEFAULT_SNS_SOURCE_ENDPOINT,
                1_700_000_000,
                "1",
            )
            .expect("valid SNS selector"),
            50,
            Some(SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string()),
        ),
        &source,
    )
    .await?;
    assert_eq!(report.neurons.len(), 1);
    assert_eq!(report.neurons[0].neuron_id, SAMPLE_SNS_NEURON_ID);

    let cache_root = neuron_source_cache_root();
    let _ = fs::remove_dir_all(&cache_root);
    let mut refresh_request = SnsSnapshotRefreshRequest::new(
        cache_root.clone(),
        ic_query::sns::SnsLookupRequest::new("ic", DEFAULT_SNS_SOURCE_ENDPOINT, 1_700_000_000, "1")
            .expect("valid SNS selector"),
        100,
    );
    refresh_request.max_pages = Some(1);

    let refresh =
        refresh_sns_neurons_cache_with_source_and_progress(&refresh_request, &source, &mut |_| {})
            .await?;
    assert_eq!(refresh.collection, "neurons");
    assert_eq!(refresh.row_count, 1);
    assert!(refresh.complete);
    let _ = fs::remove_dir_all(cache_root);

    Ok(())
}

#[cfg(feature = "sns-host")]
#[test]
fn public_sns_host_api_exposes_catalog_cache_contract() {
    type ListProgressBuilder =
        for<'a> fn(
            &'a SnsListRequest,
            &'a Path,
            &'a mut dyn ic_query::QueryProgress,
        )
            -> ic_query::report::SourceFuture<'a, Result<SnsListReport, SnsHostError>>;

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
    let _: ListProgressBuilder = |arg0, arg1, arg2| {
        Box::pin(build_sns_list_report_from_cache_or_refresh(
            arg0, arg1, arg2,
        ))
    };
    assert!(matches!(
        build_sns_list_report_from_cache(&list_request, &cache_root),
        Err(SnsHostError::MissingCatalogCache { .. })
    ));

    let refresh_request =
        SnsCatalogRefreshRequest::new(&cache_root, "ic", DEFAULT_SNS_SOURCE_ENDPOINT, 30 * 60);
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

#[cfg(feature = "sns-host")]
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
    assert_eq!(neurons_list_report.caches.len(), 0);
    assert!(sns_neurons_cache_list_report_text(&neurons_list_report).contains("cache_count: 0"));

    let neurons_status_request =
        SnsCacheStatusRequest::new(cache_root.clone(), "ic", SAMPLE_SNS_ROOT_CANISTER_ID);
    assert_eq!(neurons_status_request.cache_root(), cache_root.as_path());
    let neurons_status_report = build_sns_neurons_cache_status_report(&neurons_status_request)?;
    assert!(!neurons_status_report.found);
    let expected_neurons_cache_path = neurons_cache_path.display().to_string();
    assert_eq!(
        neurons_status_report.expected_cache_path.as_str(),
        expected_neurons_cache_path.as_str()
    );
    assert!(sns_neurons_cache_status_report_text(&neurons_status_report).contains("found: no"));

    let proposals_list_request = SnsCacheListRequest::new(cache_root.clone(), "ic");
    assert_eq!(proposals_list_request.cache_root(), cache_root.as_path());
    let proposals_list_report = build_sns_proposals_cache_list_report(&proposals_list_request)?;
    assert_eq!(proposals_list_report.caches.len(), 0);
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
        proposals_status_report.expected_cache_path.as_str(),
        expected_proposals_cache_path.as_str()
    );
    assert!(sns_proposals_cache_status_report_text(&proposals_status_report).contains("found: no"));

    Ok(())
}

#[cfg(feature = "sns-host")]
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

    let mut neurons_refresh_request = SnsSnapshotRefreshRequest::new(
        cache_root.clone(),
        ic_query::sns::SnsLookupRequest::new(
            "ic",
            DEFAULT_SNS_SOURCE_ENDPOINT,
            1_700_000_000,
            SAMPLE_SNS_ROOT_CANISTER_ID,
        )
        .expect("valid SNS selector"),
        500,
    );
    neurons_refresh_request.max_pages = Some(2);

    assert_eq!(neurons_refresh_request.max_pages, Some(2));

    let mut proposals_refresh_request = SnsSnapshotRefreshRequest::new(
        cache_root,
        ic_query::sns::SnsLookupRequest::new(
            "ic",
            DEFAULT_SNS_SOURCE_ENDPOINT,
            1_700_000_000,
            SAMPLE_SNS_ROOT_CANISTER_ID,
        )
        .expect("valid SNS selector"),
        100,
    );
    proposals_refresh_request.max_pages = Some(3);

    assert_eq!(proposals_refresh_request.max_pages, Some(3));

    let neurons_report = sample_sns_neurons_report();
    assert!(
        sns_neurons_report_text(&neurons_report, ic_query_text::TextOptions::default())
            .contains("neuron_count: 1")
    );
    let neurons_json = serde_json::to_value(&neurons_report).expect("serialize neurons report");
    assert_eq!(neurons_json["schema_version"], 1);
    assert_eq!(neurons_json["neurons"][0]["source_nns_neuron_id"], 42);
    assert_eq!(neurons_json["neurons"][0]["auto_stake_maturity"], true);
    assert_eq!(
        neurons_json["neurons"][0]["dissolve_state"],
        json!({"kind": "dissolve_delay_seconds", "value": 31_536_000})
    );
    assert_eq!(neurons_json["neurons"][0]["neuron_fees_e8s"], 10);
    for (collection, cache_path, lock_path, attempt_path) in [
        (
            "neurons",
            &neurons_cache_path,
            &neurons_lock_path,
            &neurons_attempt_path,
        ),
        (
            "proposals",
            &proposals_cache_path,
            &proposals_lock_path,
            &proposals_attempt_path,
        ),
    ] {
        let report = sample_sns_snapshot_refresh_report(
            collection,
            100,
            cache_path,
            lock_path,
            attempt_path,
        );
        let text = sns_snapshot_refresh_report_text(&report);
        assert!(text.contains(&format!("collection: {collection}")));
        assert!(text.contains("row_count: 1"));
        assert!(text.contains("wrote_cache: yes"));
        let json = serde_json::to_value(&report).expect("serialize snapshot refresh report");
        for (key, expected) in serde_json::to_value(&report.context)
            .unwrap()
            .as_object()
            .unwrap()
        {
            assert_eq!(
                json.get(key),
                Some(expected),
                "top-level context field {key}"
            );
        }
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["collection"], collection);
        assert_eq!(json["row_count"], 1);
    }
}

#[cfg(feature = "sns-host")]
fn accepts_public_function<T>(_function: T) {}

#[cfg(feature = "sns-host")]
struct FixtureSnsSource;

#[cfg(feature = "sns-host")]
impl SnsDiscoverySource for FixtureSnsSource {
    fn fetch_sns_inventory<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<MainnetSnsInventory, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(request.endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
            Ok(sample_mainnet_sns_inventory(request))
        })
    }

    fn fetch_sns_metadata<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
        targets: &'a [MainnetSnsCanisters],
    ) -> ic_query::report::SourceFuture<'a, Result<Vec<MainnetSnsMetadata>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(request.endpoint, DEFAULT_SNS_SOURCE_ENDPOINT);
            Ok(targets
                .iter()
                .map(|target| MainnetSnsMetadata {
                    root_canister_id: target.root_canister_id.clone(),
                    outcome: Ok(ic_query::sns::MainnetSnsMetadataFields {
                        name: Some("Example SNS".to_string()),
                        description: Some("Example description".to_string()),
                        url: Some("https://example.com/sns".to_string()),
                    }),
                })
                .collect())
        })
    }
}

#[cfg(feature = "sns-host")]
impl SnsProposalSource for FixtureSnsSource {
    fn fetch_sns_proposal<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        proposal_id: u64,
    ) -> ic_query::report::SourceFuture<'a, Result<SnsProposalRow, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(
                sns.canisters.governance_canister_id,
                SAMPLE_SNS_GOVERNANCE_CANISTER_ID
            );
            assert_eq!(proposal_id, 42);
            Ok(sample_sns_proposal_row())
        })
    }
}

#[cfg(feature = "sns-host")]
impl SnsProposalsSource for FixtureSnsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &'a [i32],
        topic: SnsProposalTopicFilter,
    ) -> ic_query::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(
                sns.canisters.governance_canister_id,
                SAMPLE_SNS_GOVERNANCE_CANISTER_ID
            );
            assert_eq!(limit, 10);
            assert_eq!(before_proposal_id, Some(99));
            assert_eq!(include_status, &[1]);
            assert_eq!(topic, SnsProposalTopicFilter::Governance);
            Ok(vec![sample_sns_proposal_row()])
        })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> ic_query::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(
                sns.canisters.governance_canister_id,
                SAMPLE_SNS_GOVERNANCE_CANISTER_ID
            );
            assert_eq!(limit, 100);
            assert_eq!(before_proposal_id, None);
            Ok(vec![sample_sns_proposal_row()])
        })
    }
}

#[cfg(feature = "sns-host")]
impl SnsNeuronSource for FixtureSnsSource {
    fn fetch_sns_neuron<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        neuron_id: &'a str,
    ) -> ic_query::report::SourceFuture<'a, Result<SnsNeuronDetail, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(
                sns.canisters.governance_canister_id,
                SAMPLE_SNS_GOVERNANCE_CANISTER_ID
            );
            assert_eq!(neuron_id, SAMPLE_SNS_NEURON_ID);
            Ok(sample_sns_neuron_detail())
        })
    }
}

#[cfg(feature = "sns-host")]
impl SnsNeuronsSource for FixtureSnsSource {
    fn fetch_sns_neurons<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        owner_principal_id: Option<&'a str>,
    ) -> ic_query::report::SourceFuture<'a, Result<Vec<SnsNeuronRow>, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(
                sns.canisters.governance_canister_id,
                SAMPLE_SNS_GOVERNANCE_CANISTER_ID
            );
            assert_eq!(limit, 50);
            assert_eq!(owner_principal_id, Some(SAMPLE_SNS_GOVERNANCE_CANISTER_ID));
            Ok(vec![sample_sns_neuron_row()])
        })
    }

    fn fetch_sns_neuron_page<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        start_page_at: Option<&'a SnsNeuronId>,
        owner_principal_id: Option<&'a str>,
    ) -> ic_query::report::SourceFuture<'a, Result<MainnetSnsNeuronPage, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(
                sns.canisters.governance_canister_id,
                SAMPLE_SNS_GOVERNANCE_CANISTER_ID
            );
            assert_eq!(limit, 100);
            assert!(start_page_at.is_none());
            assert_eq!(owner_principal_id, None);
            Ok(MainnetSnsNeuronPage {
                neurons: vec![sample_sns_neuron_row()],
                last_cursor: Some(SnsNeuronId {
                    id: (0..32).collect(),
                }),
            })
        })
    }
}

#[cfg(feature = "sns-host")]
fn proposal_source_cache_root() -> PathBuf {
    PathBuf::from(format!(
        "target/ic-query-sns-public-api-proposal-source-{}",
        std::process::id()
    ))
}

#[cfg(feature = "sns-host")]
fn neuron_source_cache_root() -> PathBuf {
    PathBuf::from(format!(
        "target/ic-query-sns-public-api-neuron-source-{}",
        std::process::id()
    ))
}

#[cfg(feature = "sns-host")]
fn sample_mainnet_sns_inventory(request: &CollectionSourceRequest) -> MainnetSnsInventory {
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

#[cfg(feature = "sns-host")]
fn sample_sns_neuron_row() -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
        cached_neuron_stake_e8s: 100_000_000,
        maturity_e8s_equivalent: 10_000_000,
        staked_maturity_e8s_equivalent: Some(5_000_000),
        created_timestamp_seconds: 1_700_000_000,

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

#[cfg(feature = "sns-host")]
fn sample_sns_neurons_report() -> SnsNeuronsReport {
    SnsNeuronsReport {
        provenance: ic_query::report::ReportProvenance::Live,
        schema_version: 1,
        context: sample_sns_context(),

        requested_limit: 1,
        owner_principal_id: None,

        sort: "api".to_string(),
        total_neuron_count: 1,

        neurons: vec![SnsNeuronRow {
            neuron_id: SAMPLE_SNS_NEURON_ID.to_string(),
            cached_neuron_stake_e8s: 100_000_000,
            maturity_e8s_equivalent: 10_000_000,
            staked_maturity_e8s_equivalent: Some(5_000_000),
            created_timestamp_seconds: 1_700_000_000,

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

#[cfg(feature = "sns-host")]
fn sample_sns_snapshot_refresh_report(
    collection: &str,
    page_size: u32,
    cache_path: &Path,
    refresh_lock_path: &Path,
    refresh_attempt_path: &Path,
) -> SnsSnapshotRefreshReport {
    SnsSnapshotRefreshReport {
        schema_version: 1,
        context: sample_sns_context(),
        cache_path: cache_path.display().to_string(),
        refresh_lock_path: refresh_lock_path.display().to_string(),
        refresh_attempt_path: refresh_attempt_path.display().to_string(),
        collection: collection.to_string(),
        page_size,
        page_count: 1,
        row_count: 1,
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

        title: "Upgrade SNS".to_string(),
        summary: "Upgrade the SNS controlled canister.".to_string(),
        url: Some("https://example.com/proposal/42".to_string()),

        status: Some(1),
        topic: Some("governance".to_string()),
        reject_cost_e8s: 100_000_000,
        proposal_creation_timestamp_seconds: 1_700_000_000,

        decided_timestamp_seconds: None,

        executed_timestamp_seconds: None,

        failed_timestamp_seconds: None,

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

        ballots: vec![SnsProposalBallotRow {
            neuron_id: "0102030405060708".to_string(),
            vote: 1,

            cast_timestamp_seconds: 1_700_000_200,

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
            permission_types: vec![7],
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
    }
}

fn sample_sns_context() -> ic_query::sns::SnsReportContext {
    ic_query::sns::SnsReportContext {
        instance: ic_query::sns::SnsInstanceContext {
            network: "ic".to_string(),
            sns_wasm_canister_id: "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string(),
            fetched_at: SAMPLE_SNS_FETCHED_AT.to_string(),
            source_endpoint: "https://icp-api.io".to_string(),
            fetched_by: "ic-query".to_string(),
            id: 1,
            name: "Example SNS".to_string(),
            root_canister_id: SAMPLE_SNS_ROOT_CANISTER_ID.to_string(),
        },
        governance_canister_id: SAMPLE_SNS_GOVERNANCE_CANISTER_ID.to_string(),
    }
}

fn assert_instance_json_fields(
    json: &serde_json::Value,
    context: &ic_query::sns::SnsInstanceContext,
) {
    let fields = serde_json::to_value(context).expect("instance JSON");
    for (key, value) in fields.as_object().unwrap() {
        assert_eq!(json.get(key), Some(value), "flat instance field {key}");
    }
}

#[test]
fn public_sns_contexts_round_trip_the_flat_identity_contract() {
    let context = sample_sns_context();
    let instance_json = serde_json::to_value(&context.instance).unwrap();
    let expected = serde_json::json!({
        "network": "ic",
        "sns_wasm_canister_id": "qaa6y-5yaaa-aaaaa-aaafa-cai",
        "fetched_at": SAMPLE_SNS_FETCHED_AT,
        "source_endpoint": "https://icp-api.io",
        "fetched_by": "ic-query",
        "id": 1,
        "name": "Example SNS",
        "root_canister_id": SAMPLE_SNS_ROOT_CANISTER_ID,
    });
    assert_eq!(instance_json, expected);
    let decoded_instance: ic_query::sns::SnsInstanceContext =
        serde_json::from_value(instance_json).unwrap();
    assert_eq!(decoded_instance, context.instance);

    let mut expected = expected;
    expected["governance_canister_id"] = serde_json::json!(SAMPLE_SNS_GOVERNANCE_CANISTER_ID);
    let json = serde_json::to_value(&context).unwrap();
    assert_eq!(json, expected);
    let decoded: ic_query::sns::SnsReportContext = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, context);
}
