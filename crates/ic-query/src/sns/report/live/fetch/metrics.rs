//! Module: sns::report::live::fetch::metrics
//!
//! Responsibility: fetch one bounded SNS Governance metrics response.
//! Does not own: discovery, source validation, report assembly, cache IO, or rendering.
//! Boundary: performs exactly one report-specific composite query.
use crate::candid_call::query_canister;
use crate::report::CollectionSourceRequest;
use crate::sns::report::{
    MainnetSns, MainnetSnsMetrics, SnsCanisterMethod, SnsHostError,
    live::{
        convert::mainnet_sns_metrics,
        fetch::governance_canister,
        query::sns_agent,
        types::{GetMetricsRequest, GetMetricsResponse, GetMetricsResult},
    },
    model::validate_sns_metrics_time_window,
};

pub(in crate::sns::report::live) async fn fetch_mainnet_sns_metrics(
    source: &crate::sns::LiveSnsSource,
    request: &CollectionSourceRequest,
    sns: &MainnetSns,
    time_window_seconds: u64,
) -> Result<MainnetSnsMetrics, SnsHostError> {
    validate_sns_metrics_time_window(time_window_seconds)?;
    let agent = sns_agent(source, request)?;
    let governance_canister = governance_canister(sns)?;
    let response: GetMetricsResponse = query_canister::<_, _, SnsHostError>(
        agent,
        &governance_canister,
        SnsCanisterMethod::GetMetrics.as_str(),
        "GetMetricsRequest",
        "GetMetricsResponse",
        &GetMetricsRequest {
            time_window_seconds: Some(time_window_seconds),
        },
    )
    .await?;
    metrics_response(
        &sns.canisters.governance_canister_id,
        time_window_seconds,
        response,
    )
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
            method: SnsCanisterMethod::GetMetrics.as_str(),
            error_type: error.error_type,
            message: error.error_message,
        }),
        None => Err(SnsHostError::MissingGovernanceResult {
            method: SnsCanisterMethod::GetMetrics.as_str(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOVERNANCE_CANISTER_ID: &str = "bkyz2-fmaaa-aaaaa-qaaaq-cai";

    #[test]
    fn sns_metrics_fetch_preserves_native_governance_error() {
        // External get_metrics reply carrying a Governance error through the optional result.
        let bytes = b"\x44\x49\x44\x4c\x04\x6c\x02\x90\xc6\xc1\x96\x05\x71\xd1\x9b\xc2\x8f\x0e\x75\x6b\x01\xc5\xfe\xd2\x01\x00\x6e\x01\x6c\x01\xe2\xae\xb3\xe5\x0c\x02\x01\x03\x01\x00\x13\x6d\x65\x74\x72\x69\x63\x73\x20\x75\x6e\x61\x76\x61\x69\x6c\x61\x62\x6c\x65\x07\x00\x00\x00";
        let response = crate::candid_decode::decode_reply(bytes).expect("decode SNS metrics error");
        let error = metrics_response(GOVERNANCE_CANISTER_ID, 86_400, response)
            .expect_err("native Governance error must fail");

        assert!(matches!(
            error,
            SnsHostError::GovernanceError {
                method,
                error_type: 7,
                message,
            } if method == SnsCanisterMethod::GetMetrics.as_str()
                && message == "metrics unavailable"
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
                method,
            } if method == SnsCanisterMethod::GetMetrics.as_str()
        ));
    }
}
