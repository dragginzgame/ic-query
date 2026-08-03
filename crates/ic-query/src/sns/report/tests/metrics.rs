use super::{fixtures::*, *};

#[test]
fn sns_metrics_preserves_native_cached_evidence_and_canonicalizes_rows() {
    let report =
        build_sns_metrics_report_with_source(&metrics_request("1"), &FixtureSnsMetricsSource)
            .expect("SNS metrics report");
    let text = sns_metrics_report_text(&report);

    assert_eq!(report.schema_version, SNS_METRICS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.method, "get_metrics");
    assert_eq!(report.call_type, SnsCanisterCallType::CompositeQuery);
    assert_eq!(report.time_window_seconds, 30 * 24 * 60 * 60);
    assert!(!report.point_in_time_guaranteed);
    assert!(report.treasury_metrics_cached);
    assert_eq!(report.treasury_metric_count, 2);
    assert_eq!(
        report
            .treasury_metrics
            .iter()
            .map(|row| row.treasury)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(report.num_recently_submitted_proposals, Some(12));
    assert_eq!(
        report
            .voting_power_metrics
            .as_ref()
            .and_then(|metrics| metrics.timestamp_seconds),
        Some(1_780_531_020)
    );
    assert!(text.contains("time_window: 30d"));
    assert!(text.contains("treasury_metrics_cached: yes"));
    assert!(text.contains("SNS token treasury"));
    assert!(text.contains("governance_total_potential_voting_power"));
}

#[test]
fn sns_metrics_preserves_unknown_native_treasury_codes() {
    let report = build_sns_metrics_report_with_source(
        &metrics_request("1"),
        &MutatingFixtureSnsMetricsSource(use_unknown_treasury_code),
    )
    .expect("unknown native code is preserved");

    assert_eq!(report.treasury_metrics[1].treasury, 99);
    assert_eq!(
        report.treasury_metrics[1].treasury_kind,
        SnsTreasuryKind::Unknown
    );
}

#[test]
fn sns_metrics_rejects_invalid_windows_before_source_access() {
    for seconds in [0, MAX_SNS_METRICS_TIME_WINDOW_SECONDS + 1] {
        let request = metrics_request("1").with_time_window_seconds(seconds);
        let error = build_sns_metrics_report_with_source(&request, &NoCallSnsMetricsSource)
            .expect_err("invalid window must fail before source access");

        assert!(matches!(
            error,
            SnsHostError::InvalidMetricsTimeWindow {
                seconds: actual,
                max_seconds: MAX_SNS_METRICS_TIME_WINDOW_SECONDS,
            } if actual == seconds
        ));
    }
}

#[test]
fn live_sns_metrics_source_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new(
        "local",
        "not a valid endpoint",
        "2026-08-01T00:00:00Z",
        "test",
    );
    let error = LiveSnsSource
        .fetch_sns_metrics(&request, &fixture_sns_a(), 86_400)
        .expect_err("non-mainnet must fail before endpoint parsing");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn live_sns_metrics_source_rejects_invalid_window_before_agent_construction() {
    let request =
        SnsSourceRequest::new("ic", "not a valid endpoint", "2026-08-01T00:00:00Z", "test");
    let error = LiveSnsSource
        .fetch_sns_metrics(&request, &fixture_sns_a(), 0)
        .expect_err("invalid window must fail before endpoint parsing");

    assert!(matches!(
        error,
        SnsHostError::InvalidMetricsTimeWindow { seconds: 0, .. }
    ));
}

#[test]
fn sns_metrics_rejects_invalid_custom_source_evidence() {
    for (mutate, expected_reason) in invalid_metrics_mutations() {
        let error = build_sns_metrics_report_with_source(
            &metrics_request("1"),
            &MutatingFixtureSnsMetricsSource(mutate),
        )
        .expect_err("invalid custom metrics evidence must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS metrics",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

type MetricsMutation = (fn(&mut MainnetSnsMetrics), &'static str);

fn invalid_metrics_mutations() -> [MetricsMutation; 12] {
    [
        (wrong_target, "governance_canister_id"),
        (wrong_method, "method"),
        (wrong_call_type, "call_type"),
        (wrong_window, "time_window_seconds"),
        (claim_atomic_snapshot, "point-in-time guarantee"),
        (claim_uncached_treasury, "treasury_metrics_cached"),
        (duplicate_treasury_code, "duplicate treasury code"),
        (wrong_treasury_kind, "expected"),
        (invalid_ledger_principal, "ledger_canister_id"),
        (untrimmed_name, "surrounding whitespace"),
        (invalid_subaccount, "32-byte lowercase hexadecimal"),
        (too_many_treasury_rows, "exceeds"),
    ]
}

fn use_unknown_treasury_code(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics[0].treasury = 99;
    metrics.treasury_metrics[0].treasury_kind = SnsTreasuryKind::Unknown;
}

fn wrong_target(metrics: &mut MainnetSnsMetrics) {
    metrics.governance_canister_id = ROOT_A.to_string();
}

fn wrong_method(metrics: &mut MainnetSnsMetrics) {
    metrics.method = "get_cached_metrics".to_string();
}

const fn wrong_call_type(metrics: &mut MainnetSnsMetrics) {
    metrics.call_type = SnsCanisterCallType::Query;
}

const fn wrong_window(metrics: &mut MainnetSnsMetrics) {
    metrics.time_window_seconds += 1;
}

const fn claim_atomic_snapshot(metrics: &mut MainnetSnsMetrics) {
    metrics.point_in_time_guaranteed = true;
}

const fn claim_uncached_treasury(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics_cached = false;
}

fn duplicate_treasury_code(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics[1].treasury = 2;
    metrics.treasury_metrics[1].treasury_kind = SnsTreasuryKind::SnsToken;
}

fn wrong_treasury_kind(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics[0].treasury_kind = SnsTreasuryKind::Icp;
}

fn invalid_ledger_principal(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics[0].ledger_canister_id = Some("not-a-principal".to_string());
}

fn untrimmed_name(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics[0].name = Some(" SNS treasury".to_string());
}

fn invalid_subaccount(metrics: &mut MainnetSnsMetrics) {
    metrics.treasury_metrics[0].account_subaccount_hex = Some("AB".repeat(32));
}

fn too_many_treasury_rows(metrics: &mut MainnetSnsMetrics) {
    let template = metrics.treasury_metrics[0].clone();
    metrics.treasury_metrics = (0_i32..17)
        .map(|treasury| SnsTreasuryMetricRow {
            treasury,
            treasury_kind: match treasury {
                0 => SnsTreasuryKind::Unspecified,
                1 => SnsTreasuryKind::Icp,
                2 => SnsTreasuryKind::SnsToken,
                _ => SnsTreasuryKind::Unknown,
            },
            ..template.clone()
        })
        .collect();
}
