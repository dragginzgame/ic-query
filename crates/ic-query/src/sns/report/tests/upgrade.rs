use super::{fixtures::*, *};

#[test]
fn sns_upgrade_preserves_native_versions_and_renders_comparison() {
    let report =
        build_sns_upgrade_report_with_source(&upgrade_request("1"), &FixtureSnsUpgradeSource)
            .expect("sns upgrade report");
    let text = sns_upgrade_report_text(&report);

    assert_eq!(report.schema_version, SNS_UPGRADE_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.component_query_count, 2);
    assert_eq!(report.successful_component_query_count, 2);
    assert_eq!(report.component_gap_count, 0);
    assert!(!report.point_in_time_guaranteed);
    assert_eq!(report.deployed_version, fixture_sns_version(1));
    assert_eq!(report.next_version, Some(fixture_sns_version(21)));
    assert_eq!(
        report
            .pending_upgrade
            .as_ref()
            .map(|pending| pending.proposal_id),
        Some(42)
    );
    assert!(text.contains("get_running_sns_version"));
    assert!(text.contains("get_next_sns_version"));
    assert!(text.contains("next_version: available"));
    assert!(text.contains("DEPLOYED"));
    assert!(text.contains("CHANGED"));
    assert!(text.contains("proposal_id"));
}

#[test]
fn sns_upgrade_distinguishes_successful_no_successor_from_a_query_gap() {
    let report = build_sns_upgrade_report_with_source(
        &upgrade_request("1"),
        &MutatingFixtureSnsUpgradeSource(clear_next_version),
    )
    .expect("successful empty next version");

    assert_eq!(report.successful_component_query_count, 2);
    assert_eq!(report.component_gap_count, 0);
    assert!(report.next_version.is_none());
    assert!(report.next_version_gap.is_none());
    assert!(sns_upgrade_report_text(&report).contains("no blessed successor"));
}

#[test]
fn sns_upgrade_retains_next_version_failure_as_a_typed_gap() {
    let report = build_sns_upgrade_report_with_source(
        &upgrade_request("1"),
        &MutatingFixtureSnsUpgradeSource(fail_next_version),
    )
    .expect("partial upgrade report");

    assert_eq!(report.component_query_count, 2);
    assert_eq!(report.successful_component_query_count, 1);
    assert_eq!(report.component_gap_count, 1);
    assert!(report.next_version.is_none());
    assert_eq!(
        report
            .next_version_gap
            .as_ref()
            .map(|gap| gap.method.as_str()),
        Some("get_next_sns_version")
    );
    assert!(sns_upgrade_report_text(&report).contains("query failed"));
}

#[test]
fn live_sns_upgrade_source_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new(
        "local",
        "not a valid endpoint",
        "2026-08-01T00:00:00Z",
        "test",
    );

    let error = LiveSnsSource
        .fetch_sns_upgrade(&request, &fixture_sns_a())
        .expect_err("non-mainnet must fail");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn sns_upgrade_rejects_invalid_custom_source_evidence() {
    for (mutate, expected_reason) in [
        (
            wrong_governance_target as fn(&mut MainnetSnsUpgrade),
            "governance_canister_id",
        ),
        (wrong_running_method, "running_version_method"),
        (claim_atomic_snapshot, "point-in-time guarantee"),
        (uppercase_hash, "lowercase even-length hexadecimal"),
        (value_and_gap, "both a value and a query gap"),
        (untrimmed_gap_reason, "surrounding whitespace"),
    ] {
        let error = build_sns_upgrade_report_with_source(
            &upgrade_request("1"),
            &MutatingFixtureSnsUpgradeSource(mutate),
        )
        .expect_err("invalid custom upgrade evidence must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS upgrade",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

fn clear_next_version(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.next_version = None;
}

fn fail_next_version(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.next_version = None;
    upgrade.next_version_gap = Some(SnsUpgradeQueryGap {
        method: "get_next_sns_version".to_string(),
        reason: "fixture query rejected".to_string(),
    });
}

fn wrong_governance_target(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.governance_canister_id = INDEX_A.to_string();
}

fn wrong_running_method(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.running_version_method = "get_upgrade_journal".to_string();
}

const fn claim_atomic_snapshot(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.point_in_time_guaranteed = true;
}

fn uppercase_hash(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.deployed_version.root_wasm_hash_hex = "AB".to_string();
}

fn value_and_gap(upgrade: &mut MainnetSnsUpgrade) {
    upgrade.next_version_gap = Some(SnsUpgradeQueryGap {
        method: "get_next_sns_version".to_string(),
        reason: "fixture query rejected".to_string(),
    });
}

fn untrimmed_gap_reason(upgrade: &mut MainnetSnsUpgrade) {
    fail_next_version(upgrade);
    upgrade
        .next_version_gap
        .as_mut()
        .expect("fixture gap")
        .reason = " fixture query rejected ".to_string();
}
