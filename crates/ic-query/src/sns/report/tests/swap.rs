use super::{fixtures::*, *};
use crate::sns::report::sns_swap_lifecycle_name;

#[tokio::test(flavor = "current_thread")]
async fn sns_swap_preserves_native_bounded_components_and_renders_them() {
    let report = build_sns_swap_report_with_source(&swap_request("1"), &FixtureSnsSwapSource)
        .await
        .expect("sns swap report");
    let text = sns_swap_report_text(&report);

    assert_eq!(report.schema_version, SNS_SWAP_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.swap.swap_canister_id, SWAP_A);
    assert_eq!(report.component_query_count, 3);
    assert_eq!(report.successful_component_query_count, 3);
    assert_eq!(report.component_gap_count, 0);
    assert!(!report.swap.point_in_time_guaranteed);
    assert_eq!(
        report
            .swap
            .lifecycle
            .as_ref()
            .and_then(|lifecycle| lifecycle.lifecycle),
        Some(2)
    );
    assert_eq!(
        report
            .swap
            .lifecycle
            .as_ref()
            .and_then(|lifecycle| sns_swap_lifecycle_name(lifecycle.lifecycle)),
        Some("open")
    );
    assert_eq!(
        report
            .swap
            .sale_parameters
            .as_ref()
            .map(|params| params.min_participants),
        Some(25)
    );
    assert_eq!(
        report
            .swap
            .derived_state
            .as_ref()
            .and_then(|state| state.direct_participant_count),
        Some(120)
    );
    assert!(text.contains("swap_canister_id: br5f7-7uaaa-aaaaa-qaaca-cai"));
    assert!(text.contains("point_in_time_guaranteed: no"));
    assert!(text.contains("get_lifecycle"));
    assert!(text.contains("lifecycle_name"));
    assert!(text.contains("open"));
    assert!(text.contains("min_direct_participation_icp_e8s"));
    assert!(text.contains("direct_participant_count"));
}

#[tokio::test(flavor = "current_thread")]
async fn sns_swap_retains_partial_query_failure_as_a_typed_gap() {
    let report =
        build_sns_swap_report_with_source(&swap_request("1"), &PartialFixtureSnsSwapSource)
            .await
            .expect("partial sns swap report");

    assert_eq!(report.component_query_count, 3);
    assert_eq!(report.successful_component_query_count, 2);
    assert_eq!(report.component_gap_count, 1);
    assert!(report.swap.lifecycle.is_some());
    assert!(report.swap.sale_parameters.is_some());
    assert!(report.swap.derived_state.is_none());
    assert_eq!(
        report.swap.gaps[0].component,
        SnsSwapComponent::DerivedState
    );
    assert_eq!(
        report.swap.gaps[0].method,
        SnsCanisterMethod::GetDerivedState
    );
    assert!(report.swap.gaps[0].reason.contains("query rejected"));
}

#[tokio::test(flavor = "current_thread")]
async fn sns_swap_preserves_successful_empty_sale_parameters_without_a_gap() {
    let report = build_sns_swap_report_with_source(
        &swap_request("1"),
        &MutatingFixtureSnsSwapSource(clear_sale_parameters),
    )
    .await
    .expect("successful empty sale parameters");

    assert_eq!(report.component_query_count, 3);
    assert_eq!(report.successful_component_query_count, 3);
    assert_eq!(report.component_gap_count, 0);
    assert!(report.swap.sale_parameters.is_none());
    assert_eq!(report.swap.gaps, Vec::new());
}

#[tokio::test(flavor = "current_thread")]
async fn sns_swap_preserves_three_independent_component_failures() {
    let report = build_sns_swap_report_with_source(
        &swap_request("1"),
        &MutatingFixtureSnsSwapSource(fail_all_components),
    )
    .await
    .expect("all component failures remain a report");

    assert_eq!(report.component_query_count, 3);
    assert_eq!(report.successful_component_query_count, 0);
    assert_eq!(report.component_gap_count, 3);
    assert!(report.swap.lifecycle.is_none());
    assert!(report.swap.sale_parameters.is_none());
    assert!(report.swap.derived_state.is_none());
    assert_eq!(
        report
            .swap
            .gaps
            .iter()
            .map(|gap| gap.component)
            .collect::<Vec<_>>(),
        vec![
            SnsSwapComponent::Lifecycle,
            SnsSwapComponent::SaleParameters,
            SnsSwapComponent::DerivedState,
        ]
    );
}

#[tokio::test(flavor = "current_thread")]
async fn sns_swap_rejects_a_custom_source_target_mismatch() {
    let error =
        build_sns_swap_report_with_source(&swap_request("1"), &WrongTargetFixtureSnsSwapSource)
            .await
            .expect_err("mismatched swap target must fail");

    assert!(matches!(
        error,
        SnsHostError::InvalidSourceData {
            capability: "SNS swap",
            reason,
        } if reason.contains("swap_canister_id") && reason.contains("expected")
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn live_sns_swap_source_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new(
        "local",
        "not a valid endpoint",
        "2026-08-01T00:00:00Z",
        "test",
    );
    let sns = fixture_sns_a();

    let error = LiveSnsSource::default()
        .fetch_sns_swap(&request, &sns)
        .await
        .expect_err("non-mainnet must fail");

    assert!(matches!(
        error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn sns_swap_rejects_invalid_custom_source_evidence() {
    for (mutate, expected_reason) in [
        (
            wrong_lifecycle_method as fn(&mut MainnetSnsSwap),
            "lifecycle_method",
        ),
        (claim_atomic_snapshot, "point-in-time guarantee"),
        (negative_derived_rate, "finite non-negative"),
        (value_and_gap_for_component, "both a value and a query gap"),
    ] {
        let error = build_sns_swap_report_with_source(
            &swap_request("1"),
            &MutatingFixtureSnsSwapSource(mutate),
        )
        .await
        .expect_err("invalid custom swap evidence must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS swap",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

const fn wrong_lifecycle_method(swap: &mut MainnetSnsSwap) {
    swap.lifecycle_method = SnsCanisterMethod::GetDerivedState;
}

const fn claim_atomic_snapshot(swap: &mut MainnetSnsSwap) {
    swap.point_in_time_guaranteed = true;
}

const fn negative_derived_rate(swap: &mut MainnetSnsSwap) {
    swap.derived_state
        .as_mut()
        .expect("fixture derived state")
        .sns_tokens_per_icp = Some(-1.0);
}

fn value_and_gap_for_component(swap: &mut MainnetSnsSwap) {
    swap.gaps.push(SnsSwapQueryGap {
        component: SnsSwapComponent::Lifecycle,
        method: SnsCanisterMethod::GetLifecycle,
        reason: "fixture failure".to_string(),
    });
}

const fn clear_sale_parameters(swap: &mut MainnetSnsSwap) {
    swap.sale_parameters = None;
}

fn fail_all_components(swap: &mut MainnetSnsSwap) {
    swap.lifecycle = None;
    swap.sale_parameters = None;
    swap.derived_state = None;
    swap.gaps = vec![
        SnsSwapQueryGap {
            component: SnsSwapComponent::Lifecycle,
            method: SnsCanisterMethod::GetLifecycle,
            reason: "fixture lifecycle failure".to_string(),
        },
        SnsSwapQueryGap {
            component: SnsSwapComponent::SaleParameters,
            method: SnsCanisterMethod::GetSaleParameters,
            reason: "fixture sale parameters failure".to_string(),
        },
        SnsSwapQueryGap {
            component: SnsSwapComponent::DerivedState,
            method: SnsCanisterMethod::GetDerivedState,
            reason: "fixture derived state failure".to_string(),
        },
    ];
}
