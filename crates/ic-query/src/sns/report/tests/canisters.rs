use super::{fixtures::*, *};

#[test]
fn sns_canister_report_resolves_inventory_health_and_typed_gaps() {
    let report =
        build_sns_canister_report_with_source(&info_request("1"), &FixtureSnsCanisterSource)
            .expect("sns canister report");
    let text = sns_canister_report_text(&report);

    assert_eq!(report.schema_version, SNS_CANISTER_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.root_canister_id, ROOT_A);
    assert_eq!(report.inventory_method, "list_sns_canisters");
    assert_eq!(report.health_method, "get_sns_canisters_summary");
    assert_eq!(report.health_call_type, "ingress_update");
    assert!(!report.health_update_canister_list);
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(report.canister_count, 2);
    assert_eq!(report.health_status_count, 1);
    assert_eq!(report.gap_count, 1);
    assert_eq!(report.canisters[0].role, SnsCanisterRole::Root);
    assert_eq!(report.canisters[1].role, SnsCanisterRole::Extension);
    assert_eq!(report.gaps[0].kind, SnsCanisterGapKind::HealthUnsupported);
    assert!(text.contains("health_call_type: ingress_update"));
    assert!(text.contains("health_update_canister_list: no"));
    assert!(text.contains("health_unsupported"));
}

#[test]
fn live_sns_canister_source_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new(
        "local",
        "not a valid endpoint",
        "2026-07-30T00:00:00Z",
        "test",
    );
    let sns = FixtureSnsListSource
        .fetch_deployed_snses(&SnsSourceRequest::new(
            MAINNET_NETWORK,
            DEFAULT_SNS_SOURCE_ENDPOINT,
            "2026-07-30T00:00:00Z",
            "test",
        ))
        .expect("fixture list")
        .sns_instances
        .remove(0);

    let error = LiveSnsSource
        .fetch_sns_canisters(&request, &sns)
        .expect_err("non-mainnet must fail");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}
