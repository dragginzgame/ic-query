//! Module: sns::report::build::params
//!
//! Responsibility: build SNS governance parameter reports.
//! Does not own: command parsing, governance transport internals, or rendering.
//! Boundary: resolves SNS identity, fetches parameters, and assembles the report.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, SNS_PARAMS_REPORT_SCHEMA_VERSION,
    SnsGovernanceParameters, SnsHostError, SnsLookupRequest, SnsParamsReport, SnsSourceRequest,
    live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsParamsSource,
};

pub async fn build_sns_params_report(
    request: &SnsLookupRequest,
) -> Result<SnsParamsReport, SnsHostError> {
    build_sns_params_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_params_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsParamsSource,
) -> Result<SnsParamsReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    let parameters = source
        .fetch_sns_params(&lookup.fetch_request, &lookup.sns)
        .await?;
    Ok(sns_params_report_from_parts(
        lookup.fetch_request,
        lookup.sns,
        parameters,
    ))
}

fn sns_params_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    parameters: SnsGovernanceParameters,
) -> SnsParamsReport {
    SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.canisters.root_canister_id,
        governance_canister_id: sns.canisters.governance_canister_id,
        parameters,
    }
}
