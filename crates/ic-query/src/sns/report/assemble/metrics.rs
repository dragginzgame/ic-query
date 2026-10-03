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
    SnsMetricsReport {
        schema_version: SNS_METRICS_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: metrics.governance_canister_id,
        method: metrics.method,
        call_type: metrics.call_type,
        time_window_seconds: metrics.time_window_seconds,
        point_in_time_guaranteed: metrics.point_in_time_guaranteed,
        treasury_metrics_cached: metrics.treasury_metrics_cached,
        num_recently_submitted_proposals: metrics.num_recently_submitted_proposals,
        num_recently_executed_proposals: metrics.num_recently_executed_proposals,
        last_ledger_block_timestamp: metrics.last_ledger_block_timestamp,
        genesis_timestamp_seconds: metrics.genesis_timestamp_seconds,
        treasury_metric_count: metrics.treasury_metrics.len(),
        treasury_metrics: metrics.treasury_metrics,
        voting_power_metrics: metrics.voting_power_metrics,
    }
}
