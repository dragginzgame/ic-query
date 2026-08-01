//! Module: sns::report::live::convert::metrics
//!
//! Responsibility: convert SNS Governance metrics wire values into source DTOs.
//! Does not own: live calls, source validation, lookup, or rendering.
//! Boundary: preserves native optional values and cached timestamps without inference.

use super::common::clean_optional_text;
use crate::{
    hex::hex_bytes,
    sns::report::{
        MainnetSnsMetrics, SnsTreasuryMetricRow, SnsVotingPowerMetrics,
        live::types::{MetricsWire, TreasuryMetricsWire, VotingPowerMetricsWire},
        source::{SNS_METRICS_CALL_TYPE, SNS_METRICS_METHOD, sns_treasury_kind},
    },
};

pub(in crate::sns::report::live) fn mainnet_sns_metrics(
    governance_canister_id: String,
    time_window_seconds: u64,
    metrics: MetricsWire,
) -> MainnetSnsMetrics {
    MainnetSnsMetrics {
        governance_canister_id,
        method: SNS_METRICS_METHOD.to_string(),
        call_type: SNS_METRICS_CALL_TYPE.to_string(),
        time_window_seconds,
        point_in_time_guaranteed: false,
        treasury_metrics_cached: true,
        num_recently_submitted_proposals: metrics.num_recently_submitted_proposals,
        num_recently_executed_proposals: metrics.num_recently_executed_proposals,
        last_ledger_block_timestamp: metrics.last_ledger_block_timestamp,
        genesis_timestamp_seconds: metrics.genesis_timestamp_seconds,
        treasury_metrics: metrics
            .treasury_metrics
            .unwrap_or_default()
            .into_iter()
            .map(treasury_metric_row)
            .collect(),
        voting_power_metrics: metrics.voting_power_metrics.map(voting_power_metrics),
    }
}

fn treasury_metric_row(metric: TreasuryMetricsWire) -> SnsTreasuryMetricRow {
    let (account_owner, account_subaccount_hex) = metric.account.map_or((None, None), |account| {
        (
            account.owner.map(|owner| owner.to_text()),
            account
                .subaccount
                .map(|subaccount| hex_bytes(&subaccount.subaccount)),
        )
    });
    SnsTreasuryMetricRow {
        treasury: metric.treasury,
        treasury_kind: sns_treasury_kind(metric.treasury),
        name: clean_optional_text(metric.name),
        ledger_canister_id: metric
            .ledger_canister_id
            .map(|canister_id| canister_id.to_text()),
        account_owner,
        account_subaccount_hex,
        amount_e8s: metric.amount_e8s,
        original_amount_e8s: metric.original_amount_e8s,
        timestamp_seconds: metric.timestamp_seconds,
    }
}

const fn voting_power_metrics(metric: VotingPowerMetricsWire) -> SnsVotingPowerMetrics {
    SnsVotingPowerMetrics {
        governance_total_potential_voting_power: metric.governance_total_potential_voting_power,
        timestamp_seconds: metric.timestamp_seconds,
    }
}
