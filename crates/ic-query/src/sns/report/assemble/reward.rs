//! Module: sns::report::assemble::reward
//!
//! Responsibility: assemble one validated SNS reward checkpoint report.
//! Does not own: live calls, strict pagination, policy recomputation, or rendering.
//! Boundary: maps resolved target, stable brackets, rows, and summary into the public DTO.

use crate::{
    report::ReportDataSource,
    sns::report::{
        JoinedMainnetSnsInventory, MainnetSns, SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION,
        SnsGovernanceParameters, SnsHostError, SnsRewardCheckpointReport, SnsRewardCheckpointRow,
        SnsRewardCheckpointSummary, SnsRewardCollectionStatus, SnsRewardEvent,
        SnsRunningVersionResponse,
    },
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// SnsRewardCheckpointReportParts
///
/// Validated collection inputs needed to assemble one reward checkpoint.
///

pub(in crate::sns::report) struct SnsRewardCheckpointReportParts {
    pub(in crate::sns::report) list: JoinedMainnetSnsInventory,
    pub(in crate::sns::report) id: usize,
    pub(in crate::sns::report) sns: MainnetSns,
    pub(in crate::sns::report) collection_started_at_unix_secs: u64,
    pub(in crate::sns::report) collection_completed_at_unix_secs: u64,
    pub(in crate::sns::report) page_count: u32,
    pub(in crate::sns::report) collection_row_ceiling: u64,
    pub(in crate::sns::report) parameters_before: SnsGovernanceParameters,
    pub(in crate::sns::report) parameters_after: SnsGovernanceParameters,
    pub(in crate::sns::report) reward_event_before: SnsRewardEvent,
    pub(in crate::sns::report) reward_event_after: SnsRewardEvent,
    pub(in crate::sns::report) running_version_before: SnsRunningVersionResponse,
    pub(in crate::sns::report) running_version_after: SnsRunningVersionResponse,
    pub(in crate::sns::report) rows: Vec<SnsRewardCheckpointRow>,
    pub(in crate::sns::report) summary: SnsRewardCheckpointSummary,
}

pub(in crate::sns::report) fn sns_reward_checkpoint_report_from_parts(
    parts: SnsRewardCheckpointReportParts,
) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    let client_query_count =
        parts
            .page_count
            .checked_add(8)
            .ok_or(SnsHostError::RewardCheckpointArithmetic {
                field: "client_query_count",
            })?;
    let row_count = parts.rows.len();
    Ok(SnsRewardCheckpointReport {
        schema_version: SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        ledger_canister_id: parts.sns.ledger_canister_id,
        swap_canister_id: parts.sns.swap_canister_id,
        index_canister_id: parts.sns.index_canister_id,
        data_source: ReportDataSource::Live,
        collection_started_at_unix_secs: parts.collection_started_at_unix_secs,
        collection_started_at: format_utc_timestamp_secs(parts.collection_started_at_unix_secs),
        collection_completed_at_unix_secs: parts.collection_completed_at_unix_secs,
        collection_completed_at: format_utc_timestamp_secs(parts.collection_completed_at_unix_secs),
        page_size: crate::sns::report::SNS_REWARD_CHECKPOINT_PAGE_SIZE,
        page_count: parts.page_count,
        row_count,
        unique_neuron_id_count: row_count,
        collection_row_ceiling: parts.collection_row_ceiling,
        client_query_count,
        collection_status: SnsRewardCollectionStatus::ApiExhaustedObserved,
        point_in_time_guaranteed: false,
        parameters_before: parts.parameters_before,
        parameters_after: parts.parameters_after,
        reward_event_before: parts.reward_event_before,
        reward_event_after: parts.reward_event_after,
        running_version_before: parts.running_version_before,
        running_version_after: parts.running_version_after,
        aggregate_maturity_e8s_equivalent: parts.summary.aggregate_maturity_e8s_equivalent,
        aggregate_staked_maturity_e8s_equivalent: parts
            .summary
            .aggregate_staked_maturity_e8s_equivalent,
        aggregate_combined_maturity_e8s_equivalent: parts
            .summary
            .aggregate_combined_maturity_e8s_equivalent,
        permission_entry_count: parts.summary.permission_entry_count,
        unassessable_permission_code_count: parts.summary.unassessable_permission_code_count,
        pending_maturity_disbursement_count: parts.summary.pending_maturity_disbursement_count,
        auto_stake_maturity_enabled_count: parts.summary.auto_stake_maturity_enabled_count,
        auto_stake_maturity_disabled_count: parts.summary.auto_stake_maturity_disabled_count,
        auto_stake_maturity_unspecified_count: parts.summary.auto_stake_maturity_unspecified_count,
        manage_principals_grantable: parts.summary.manage_principals_grantable,
        maturity_mint_conversion_observed_disabled: parts
            .summary
            .maturity_mint_conversion_observed_disabled,
        manual_maturity_staking_observed_disabled: parts
            .summary
            .manual_maturity_staking_observed_disabled,
        maturity_conversion_policy_observed_status: parts
            .summary
            .maturity_conversion_policy_observed_status,
        rows: parts.rows,
    })
}
