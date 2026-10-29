//! Module: sns::report::build::reward
//!
//! Responsibility: collect and validate one bracketed SNS reward checkpoint.
//! Does not own: CLI parsing, native wire decoding, filesystem persistence, or diffing.
//! Boundary: enforces call order, mandatory bounds, strict exhaustion, stable brackets, and summaries.

use crate::sns::report::{
    SNS_REWARD_CHECKPOINT_MAX_NEURONS, SNS_REWARD_CHECKPOINT_PAGE_SIZE, SnsGovernanceParameters,
    SnsHostError, SnsRewardCheckpointReport, SnsRewardCheckpointRequest, SnsRewardCollectionState,
    SnsRewardSource,
    live::LiveSnsSource,
    lookup::resolve_sns_lookup,
    model::reports::neurons::reward::{
        recompute_reward_checkpoint_summary, reward_checkpoint_bracket_mismatch,
        reward_checkpoint_neuron_ceiling, validate_sns_reward_checkpoint_parameter_evidence,
        validate_sns_reward_event_evidence, validate_sns_reward_running_version_evidence,
    },
};
use crate::{
    report::ReportDataSource,
    sns::report::{
        MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION,
        SnsRewardCheckpointRow, SnsRewardCheckpointSummary, SnsRewardCollectionStatus,
        SnsRewardEvent, SnsRunningVersionResponse, SnsSourceRequest,
    },
};
use ic_host_text::timestamp::format_utc_timestamp_secs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Build one live API-exhausted observed SNS reward checkpoint.
pub async fn build_sns_reward_checkpoint_report(
    request: &SnsRewardCheckpointRequest,
) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    build_sns_reward_checkpoint_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build one SNS reward checkpoint through an explicit source adapter.
pub async fn build_sns_reward_checkpoint_report_with_source(
    request: &SnsRewardCheckpointRequest,
    source: &dyn SnsRewardSource,
) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    validate_request(request)?;
    let lookup = resolve_sns_lookup(&request.lookup, source).await?;

    let running_version_before = source
        .fetch_sns_reward_running_version(&lookup.fetch_request, &lookup.sns)
        .await?;
    let parameters_before = source
        .fetch_sns_reward_parameters(&lookup.fetch_request, &lookup.sns)
        .await?;
    let reward_event_before = source
        .fetch_sns_reward_event(&lookup.fetch_request, &lookup.sns)
        .await?;
    validate_sns_reward_running_version_evidence(&running_version_before)
        .map_err(invalid_checkpoint)?;
    validate_sns_reward_event_evidence(&reward_event_before).map_err(invalid_checkpoint)?;
    let collection_row_ceiling = validate_parameters(&parameters_before)?;

    let collection = collect_reward_neurons(
        request,
        source,
        &lookup.fetch_request,
        &lookup.sns,
        collection_row_ceiling,
    )
    .await?;

    let reward_event_after = source
        .fetch_sns_reward_event(&lookup.fetch_request, &lookup.sns)
        .await?;
    let parameters_after = source
        .fetch_sns_reward_parameters(&lookup.fetch_request, &lookup.sns)
        .await?;
    let running_version_after = source
        .fetch_sns_reward_running_version(&lookup.fetch_request, &lookup.sns)
        .await?;
    if let Some((component, _)) = reward_checkpoint_bracket_mismatch(
        (&parameters_before, &parameters_after),
        (&reward_event_before, &reward_event_after),
        (&running_version_before, &running_version_after),
    ) {
        return Err(SnsHostError::UnstableRewardCheckpoint { component });
    }

    let completed_at = checkpoint_completion_time(request.lookup.now_unix_secs)?;
    let page_count = collection.page_count();
    let rows = collection.into_rows();
    let summary = recompute_reward_checkpoint_summary(&parameters_before, &rows)
        .map_err(invalid_checkpoint)?;
    sns_reward_checkpoint_report_from_parts(SnsRewardCheckpointReportParts {
        fetch_request: lookup.fetch_request,
        sns: lookup.sns,
        collection_started_at_unix_secs: request.lookup.now_unix_secs,
        collection_completed_at_unix_secs: completed_at,
        page_count,
        collection_row_ceiling,
        parameters_before,
        parameters_after,
        reward_event_before,
        reward_event_after,
        running_version_before,
        running_version_after,
        rows,
        summary,
    })
}

async fn collect_reward_neurons(
    request: &SnsRewardCheckpointRequest,
    source: &dyn SnsRewardSource,
    source_request: &crate::sns::report::SnsSourceRequest,
    sns: &crate::sns::report::MainnetSns,
    collection_row_ceiling: u64,
) -> Result<SnsRewardCollectionState, SnsHostError> {
    let mandatory_page_limit = collection_row_ceiling
        .div_ceil(u64::from(SNS_REWARD_CHECKPOINT_PAGE_SIZE))
        .checked_add(1)
        .ok_or(SnsHostError::RewardCheckpointArithmetic {
            field: "mandatory_page_limit",
        })?;
    let mut collection = SnsRewardCollectionState::new();
    while !collection.exhausted() {
        enforce_page_limits(request, &collection, mandatory_page_limit)?;
        let page = source
            .fetch_sns_reward_neuron_page(
                source_request,
                sns,
                SNS_REWARD_CHECKPOINT_PAGE_SIZE,
                collection.next_cursor(),
            )
            .await?;
        collection.ingest_page(page)?;
        let row_count = u64::try_from(collection.row_count())
            .map_err(|_| SnsHostError::RewardCheckpointArithmetic { field: "row_count" })?;
        if row_count > collection_row_ceiling {
            return Err(invalid_checkpoint(format!(
                "collected {row_count} neurons above mandatory ceiling {collection_row_ceiling}"
            )));
        }
    }
    Ok(collection)
}

fn enforce_page_limits(
    request: &SnsRewardCheckpointRequest,
    collection: &SnsRewardCollectionState,
    mandatory_page_limit: u64,
) -> Result<(), SnsHostError> {
    if let Some(max_pages) = request.max_pages
        && collection.page_count() >= max_pages
    {
        return Err(SnsHostError::IncompleteRewardCheckpoint {
            pages_fetched: collection.page_count(),
            rows_fetched: collection.row_count(),
            reason: format!("diagnostic max_pages {max_pages} reached before API exhaustion"),
        });
    }
    if u64::from(collection.page_count()) >= mandatory_page_limit {
        return Err(SnsHostError::IncompleteRewardCheckpoint {
            pages_fetched: collection.page_count(),
            rows_fetched: collection.row_count(),
            reason: format!(
                "mandatory page bound {mandatory_page_limit} reached before API exhaustion"
            ),
        });
    }
    Ok(())
}

fn validate_request(request: &SnsRewardCheckpointRequest) -> Result<(), SnsHostError> {
    if request.max_pages == Some(0) {
        return Err(SnsHostError::InvalidRewardCheckpointPageCap { max_pages: 0 });
    }
    Ok(())
}

fn validate_parameters(parameters: &SnsGovernanceParameters) -> Result<u64, SnsHostError> {
    let max_number_of_neurons = reward_checkpoint_neuron_ceiling(parameters).ok_or(
        SnsHostError::InvalidRewardCheckpointCeiling {
            value: parameters.max_number_of_neurons,
            maximum: SNS_REWARD_CHECKPOINT_MAX_NEURONS,
        },
    )?;
    validate_sns_reward_checkpoint_parameter_evidence(parameters).map_err(invalid_checkpoint)?;
    Ok(max_number_of_neurons)
}

fn checkpoint_completion_time(started_at: u64) -> Result<u64, SnsHostError> {
    let completed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| SnsHostError::RewardCheckpointClock {
            reason: error.to_string(),
        })?
        .as_secs();
    if completed_at < started_at {
        return Err(SnsHostError::RewardCheckpointClock {
            reason: format!(
                "completion timestamp {completed_at} precedes supplied start {started_at}"
            ),
        });
    }
    Ok(completed_at)
}

fn invalid_checkpoint(reason: impl Into<String>) -> SnsHostError {
    SnsHostError::InvalidSourceData {
        capability: "SNS reward checkpoint",
        reason: reason.into(),
    }
}

///
/// SnsRewardCheckpointReportParts
///
/// Validated collection inputs needed to assemble one reward checkpoint.
///

struct SnsRewardCheckpointReportParts {
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    collection_started_at_unix_secs: u64,
    collection_completed_at_unix_secs: u64,
    page_count: u32,
    collection_row_ceiling: u64,
    parameters_before: SnsGovernanceParameters,
    parameters_after: SnsGovernanceParameters,
    reward_event_before: SnsRewardEvent,
    reward_event_after: SnsRewardEvent,
    running_version_before: SnsRunningVersionResponse,
    running_version_after: SnsRunningVersionResponse,
    rows: Vec<SnsRewardCheckpointRow>,
    summary: SnsRewardCheckpointSummary,
}

fn sns_reward_checkpoint_report_from_parts(
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
    let report = SnsRewardCheckpointReport {
        schema_version: SNS_REWARD_CHECKPOINT_REPORT_SCHEMA_VERSION,
        network: parts.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        source_endpoint: parts.fetch_request.endpoint,
        fetched_by: parts.fetch_request.fetched_by,
        id: parts.sns.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.canisters.root_canister_id,
        governance_canister_id: parts.sns.canisters.governance_canister_id,
        ledger_canister_id: parts.sns.canisters.ledger_canister_id,
        swap_canister_id: parts.sns.canisters.swap_canister_id,
        index_canister_id: parts.sns.canisters.index_canister_id,
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
    };
    report
        .validate_collected_summary(&parts.summary)
        .map_err(|error| SnsHostError::InvalidSourceData {
            capability: "SNS reward checkpoint",
            reason: error.reason,
        })?;
    Ok(report)
}
