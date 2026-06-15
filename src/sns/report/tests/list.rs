use super::{fixtures::*, *};

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
