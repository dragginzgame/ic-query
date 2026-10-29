//! Module: sns::report::build::metrics
//!
//! Responsibility: build bounded SNS Governance metrics reports.
//! Does not own: command parsing, transport internals, or rendering.
//! Boundary: validates the request, resolves one SNS, and validates source evidence.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_METRICS_REPORT_SCHEMA_VERSION, SnsHostError,
    SnsMetricsReport, SnsMetricsRequest, SnsMetricsSource, live::LiveSnsSource,
    lookup::resolve_sns_lookup, model::validate_sns_metrics_request,
    source::canonicalize_mainnet_sns_metrics,
};

/// Build a live bounded Governance metrics report for one deployed SNS.
pub async fn build_sns_metrics_report(
    request: &SnsMetricsRequest,
) -> Result<SnsMetricsReport, SnsHostError> {
    build_sns_metrics_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build a bounded Governance metrics report through a custom source capability.
pub async fn build_sns_metrics_report_with_source(
    request: &SnsMetricsRequest,
    source: &dyn SnsMetricsSource,
) -> Result<SnsMetricsReport, SnsHostError> {
    validate_sns_metrics_request(request)?;
    let lookup = resolve_sns_lookup(&request.lookup, source).await?;
    let mut metrics = source
        .fetch_sns_metrics(
            &lookup.fetch_request,
            &lookup.sns,
            request.time_window_seconds,
        )
        .await?;
    canonicalize_mainnet_sns_metrics(
        &mut metrics,
        &lookup.sns.canisters.governance_canister_id,
        request.time_window_seconds,
    )?;
    let treasury_metric_count = metrics.treasury_metrics.len();
    Ok(SnsMetricsReport {
        schema_version: SNS_METRICS_REPORT_SCHEMA_VERSION,
        network: lookup.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: lookup.fetch_request.fetched_at,
        source_endpoint: lookup.fetch_request.endpoint,
        fetched_by: lookup.fetch_request.fetched_by,
        id: lookup.sns.id,
        name: lookup.sns.name,
        root_canister_id: lookup.sns.canisters.root_canister_id,
        treasury_metric_count,
        metrics,
    })
}
