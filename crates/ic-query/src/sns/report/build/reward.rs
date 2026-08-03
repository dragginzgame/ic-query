//! Module: sns::report::build::reward
//!
//! Responsibility: collect and validate one bracketed SNS reward checkpoint.
//! Does not own: CLI parsing, native wire decoding, filesystem persistence, or diffing.
//! Boundary: enforces call order, mandatory bounds, strict exhaustion, stable brackets, and summaries.

use crate::sns::report::{
    SNS_REWARD_CHECKPOINT_MAX_NEURONS, SNS_REWARD_CHECKPOINT_PAGE_SIZE, SnsGovernanceParameters,
    SnsHostError, SnsRewardCheckpointReport, SnsRewardCheckpointRequest, SnsRewardCollectionState,
    SnsRewardEvent, SnsRewardSource, SnsRunningVersionResponse,
    assemble::{SnsRewardCheckpointReportParts, sns_reward_checkpoint_report_from_parts},
    live::LiveSnsSource,
    lookup::{lookup_request_from_parts, resolve_sns_lookup},
    recompute_reward_checkpoint_summary, validate_sns_reward_checkpoint_parameter_evidence,
    validate_sns_reward_checkpoint_report, validate_sns_reward_event_evidence,
    validate_sns_reward_running_version_evidence,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Build one live API-exhausted observed SNS reward checkpoint.
pub fn build_sns_reward_checkpoint_report(
    request: &SnsRewardCheckpointRequest,
) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    build_sns_reward_checkpoint_report_with_source(request, &LiveSnsSource)
}

/// Build one SNS reward checkpoint through an explicit source adapter.
pub fn build_sns_reward_checkpoint_report_with_source(
    request: &SnsRewardCheckpointRequest,
    source: &dyn SnsRewardSource,
) -> Result<SnsRewardCheckpointReport, SnsHostError> {
    validate_request(request)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;

    let running_version_before =
        source.fetch_sns_reward_running_version(&lookup.fetch_request, &lookup.sns)?;
    let parameters_before =
        source.fetch_sns_reward_parameters(&lookup.fetch_request, &lookup.sns)?;
    let reward_event_before = source.fetch_sns_reward_event(&lookup.fetch_request, &lookup.sns)?;
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
    )?;

    let reward_event_after = source.fetch_sns_reward_event(&lookup.fetch_request, &lookup.sns)?;
    let parameters_after =
        source.fetch_sns_reward_parameters(&lookup.fetch_request, &lookup.sns)?;
    let running_version_after =
        source.fetch_sns_reward_running_version(&lookup.fetch_request, &lookup.sns)?;
    validate_stable_brackets(
        &parameters_before,
        &parameters_after,
        &reward_event_before,
        &reward_event_after,
        &running_version_before,
        &running_version_after,
    )?;

    let completed_at = checkpoint_completion_time(request.now_unix_secs)?;
    let page_count = collection.page_count();
    let rows = collection.into_rows();
    let summary = recompute_reward_checkpoint_summary(&parameters_before, &rows)
        .map_err(invalid_checkpoint)?;
    let report = sns_reward_checkpoint_report_from_parts(SnsRewardCheckpointReportParts {
        list: lookup.list,
        id: lookup.id,
        sns: lookup.sns,
        collection_started_at_unix_secs: request.now_unix_secs,
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
    })?;
    validate_sns_reward_checkpoint_report(&report)
        .map_err(|error| invalid_checkpoint(error.reason))?;
    Ok(report)
}

fn collect_reward_neurons(
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
        let page = source.fetch_sns_reward_neuron_page(
            source_request,
            sns,
            SNS_REWARD_CHECKPOINT_PAGE_SIZE,
            collection.next_cursor(),
        )?;
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

fn validate_stable_brackets(
    parameters_before: &SnsGovernanceParameters,
    parameters_after: &SnsGovernanceParameters,
    event_before: &SnsRewardEvent,
    event_after: &SnsRewardEvent,
    version_before: &SnsRunningVersionResponse,
    version_after: &SnsRunningVersionResponse,
) -> Result<(), SnsHostError> {
    for (component, stable) in [
        (
            "nervous-system parameters",
            parameters_before == parameters_after,
        ),
        ("reward event", event_before == event_after),
        ("running SNS version", version_before == version_after),
    ] {
        if !stable {
            return Err(SnsHostError::UnstableRewardCheckpoint { component });
        }
    }
    validate_parameters(parameters_after)?;
    validate_sns_reward_event_evidence(event_after).map_err(invalid_checkpoint)?;
    validate_sns_reward_running_version_evidence(version_after).map_err(invalid_checkpoint)
}

fn validate_parameters(parameters: &SnsGovernanceParameters) -> Result<u64, SnsHostError> {
    let Some(max_number_of_neurons) = parameters.max_number_of_neurons else {
        return Err(SnsHostError::InvalidRewardCheckpointCeiling {
            value: None,
            maximum: SNS_REWARD_CHECKPOINT_MAX_NEURONS,
        });
    };
    if max_number_of_neurons == 0 || max_number_of_neurons > SNS_REWARD_CHECKPOINT_MAX_NEURONS {
        return Err(SnsHostError::InvalidRewardCheckpointCeiling {
            value: Some(max_number_of_neurons),
            maximum: SNS_REWARD_CHECKPOINT_MAX_NEURONS,
        });
    }
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
