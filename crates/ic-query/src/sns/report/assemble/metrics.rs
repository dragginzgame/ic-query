//! Module: sns::report::assemble::metrics
//!
//! Responsibility: assemble bounded SNS metrics report DTOs.
//! Does not own: lookup, live calls, source validation, or rendering.
//! Boundary: combines discovery provenance with native Governance metrics evidence.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsMetrics, SNS_METRICS_REPORT_SCHEMA_VERSION,
    SnsMetricsReport, SnsSourceRequest,
};

pub(in crate::sns::report) fn sns_metrics_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    metrics: MainnetSnsMetrics,
) -> SnsMetricsReport {
    let treasury_metric_count = metrics.treasury_metrics.len();
    SnsMetricsReport {
        schema_version: SNS_METRICS_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.canisters.root_canister_id,
        treasury_metric_count,
        metrics,
    }
}
