//! Module: sns::report::assemble::metrics
//!
//! Responsibility: assemble bounded SNS metrics report DTOs.
//! Does not own: lookup, live calls, source validation, or rendering.
//! Boundary: combines discovery provenance with native Governance metrics evidence.

use crate::sns::report::{
    JoinedMainnetSnsInventory, MainnetSns, MainnetSnsMetrics, SNS_METRICS_REPORT_SCHEMA_VERSION,
    SnsMetricsReport,
};

pub(in crate::sns::report) fn sns_metrics_report_from_parts(
    list: JoinedMainnetSnsInventory,
    id: usize,
    sns: MainnetSns,
    metrics: MainnetSnsMetrics,
) -> SnsMetricsReport {
    SnsMetricsReport {
        schema_version: SNS_METRICS_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
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
