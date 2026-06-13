use super::*;

const ROOT_A: &str = "be2us-64aaa-aaaaa-qaabq-cai";
const GOVERNANCE_A: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";
const LEDGER_A: &str = "bd3sg-teaaa-aaaaa-qaaba-cai";
const SWAP_A: &str = "br5f7-7uaaa-aaaaa-qaaca-cai";
const INDEX_A: &str = "bw4dl-smaaa-aaaaa-qaacq-cai";

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
    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "Fixture SNS");
    assert_eq!(report.sns_instances[0].root_canister_id, ROOT_A);
    assert!(text.contains("NAME"));
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
            sns_instances: vec![MainnetSns {
                name: "Fixture SNS".to_string(),
                description: Some("Fixture description".to_string()),
                url: Some("https://example.com".to_string()),
                root_canister_id: ROOT_A.to_string(),
                governance_canister_id: GOVERNANCE_A.to_string(),
                ledger_canister_id: LEDGER_A.to_string(),
                swap_canister_id: SWAP_A.to_string(),
                index_canister_id: INDEX_A.to_string(),
            }],
        })
    }
}
