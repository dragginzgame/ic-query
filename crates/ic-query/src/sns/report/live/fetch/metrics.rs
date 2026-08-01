//! Module: sns::report::live::fetch::metrics
//!
//! Responsibility: fetch one bounded SNS Governance metrics response.
//! Does not own: discovery, source validation, report assembly, cache IO, or rendering.
//! Boundary: performs exactly one report-specific composite query.

use super::block_on_sns;
use crate::sns::report::{
    MainnetSns, MainnetSnsMetrics, SnsHostError, SnsSourceRequest,
    live::{
        convert::mainnet_sns_metrics,
        fetch::governance_canister,
        query::{query_canister, sns_agent},
        types::{GetMetricsRequest, GetMetricsResponse, GetMetricsResult},
    },
    model::validate_sns_metrics_time_window,
    source::SNS_METRICS_METHOD,
};

/// Fetch one native metrics response from a resolved SNS Governance canister.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_metrics(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    time_window_seconds: u64,
) -> Result<MainnetSnsMetrics, SnsHostError> {
    validate_sns_metrics_time_window(time_window_seconds)?;
    block_on_sns(fetch_mainnet_sns_metrics_async(
        request,
        sns,
        time_window_seconds,
    ))
}

async fn fetch_mainnet_sns_metrics_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
    time_window_seconds: u64,
) -> Result<MainnetSnsMetrics, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister = governance_canister(sns)?;
    let response: GetMetricsResponse = query_canister(
        &agent,
        &governance_canister,
        SNS_METRICS_METHOD,
        "GetMetricsRequest",
        "GetMetricsResponse",
        &GetMetricsRequest {
            time_window_seconds: Some(time_window_seconds),
        },
    )
    .await?;
    metrics_response(&sns.governance_canister_id, time_window_seconds, response)
}

fn metrics_response(
    governance_canister_id: &str,
    time_window_seconds: u64,
    response: GetMetricsResponse,
) -> Result<MainnetSnsMetrics, SnsHostError> {
    match response.get_metrics_result {
        Some(GetMetricsResult::Ok(metrics)) => Ok(mainnet_sns_metrics(
            governance_canister_id.to_string(),
            time_window_seconds,
            metrics,
        )),
        Some(GetMetricsResult::Err(error)) => Err(SnsHostError::GovernanceError {
            method: SNS_METRICS_METHOD,
            error_type: error.error_type,
            message: error.error_message,
        }),
        None => Err(SnsHostError::MissingGovernanceResult {
            method: SNS_METRICS_METHOD,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sns::report::live::types::SnsGovernanceError;

    const GOVERNANCE_CANISTER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";

    #[test]
    fn sns_metrics_fetch_preserves_native_governance_error() {
        let error = metrics_response(
            GOVERNANCE_CANISTER_ID,
            86_400,
            GetMetricsResponse {
                get_metrics_result: Some(GetMetricsResult::Err(SnsGovernanceError {
                    error_type: 7,
                    error_message: "metrics unavailable".to_string(),
                })),
            },
        )
        .expect_err("native Governance error must fail");

        assert!(matches!(
            error,
            SnsHostError::GovernanceError {
                method: SNS_METRICS_METHOD,
                error_type: 7,
                message,
            } if message == "metrics unavailable"
        ));
    }

    #[test]
    fn sns_metrics_fetch_rejects_missing_native_result() {
        let error = metrics_response(
            GOVERNANCE_CANISTER_ID,
            86_400,
            GetMetricsResponse {
                get_metrics_result: None,
            },
        )
        .expect_err("missing Governance result must fail");

        assert!(matches!(
            error,
            SnsHostError::MissingGovernanceResult {
                method: SNS_METRICS_METHOD,
            }
        ));
    }
}
