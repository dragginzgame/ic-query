//! Module: sns::report::text::reward
//!
//! Responsibility: render one SNS reward checkpoint summary for humans.
//! Does not own: neuron collection, checkpoint validation, or JSON serialization.
//! Boundary: keeps potentially large raw row evidence in JSON while text remains bounded.

use ic_host_text::escape::terminal_text;

use crate::{
    sns::report::{SnsRewardCheckpointReport, text::common::optional_bool_text},
    token_amount::e8s_decimal_text,
};

/// Render one API-exhausted SNS reward checkpoint as bounded human-readable text.
#[must_use]
pub fn sns_reward_checkpoint_report_text(report: &SnsRewardCheckpointReport) -> String {
    [
        format!("network: {}", terminal_text(&report.network)),
        format!("sns_id: {}", report.id),
        format!("name: {}", terminal_text(&report.name)),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("collection_status: {}", report.collection_status.as_str()),
        format!(
            "point_in_time_guaranteed: {}",
            optional_bool_text(Some(report.point_in_time_guaranteed))
        ),
        format!("collection_started_at: {}", report.collection_started_at),
        format!(
            "collection_completed_at: {}",
            report.collection_completed_at
        ),
        format!("page_count: {}", report.page_count),
        format!("row_count: {}", report.row_count),
        format!("collection_row_ceiling: {}", report.collection_row_ceiling),
        format!("client_query_count: {}", report.client_query_count),
        format!(
            "reward_event_end_timestamp_seconds: {}",
            report
                .reward_event_after
                .end_timestamp_seconds
                .map_or_else(|| "-".to_string(), |value| value.to_string())
        ),
        format!("reward_event_round: {}", report.reward_event_after.round),
        format!(
            "reward_event_distributed: {}",
            e8s_decimal_text(report.reward_event_after.distributed_e8s_equivalent)
        ),
        format!(
            "aggregate_unstaked_maturity: {}",
            e8s_decimal_text(report.summary.aggregate_maturity_e8s_equivalent)
        ),
        format!(
            "aggregate_staked_maturity: {}",
            e8s_decimal_text(report.summary.aggregate_staked_maturity_e8s_equivalent)
        ),
        format!(
            "aggregate_combined_maturity: {}",
            e8s_decimal_text(report.summary.aggregate_combined_maturity_e8s_equivalent)
        ),
        format!(
            "permission_entry_count: {}",
            report.summary.permission_entry_count
        ),
        format!(
            "pending_maturity_disbursement_count: {}",
            report.summary.pending_maturity_disbursement_count
        ),
        format!(
            "unassessable_permission_code_count: {}",
            report.summary.unassessable_permission_code_count
        ),
        format!(
            "manage_principals_grantable: {}",
            optional_bool_text(report.summary.manage_principals_grantable)
        ),
        format!(
            "maturity_mint_conversion_observed_disabled: {}",
            report
                .summary
                .maturity_mint_conversion_observed_disabled
                .as_str()
        ),
        format!(
            "manual_maturity_staking_observed_disabled: {}",
            report
                .summary
                .manual_maturity_staking_observed_disabled
                .as_str()
        ),
        format!(
            "maturity_conversion_policy_observed_status: {}",
            report
                .summary
                .maturity_conversion_policy_observed_status
                .as_str()
        ),
        format!(
            "source_endpoint: {}",
            terminal_text(&report.source_endpoint)
        ),
    ]
    .join("\n")
}
