//! Module: sns::report::build::params
//!
//! Responsibility: build SNS governance parameter reports.
//! Does not own: command parsing, governance transport internals, or rendering.
//! Boundary: resolves SNS identity, fetches parameters, and assembles the report.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, SNS_PARAMS_REPORT_SCHEMA_VERSION, SnsHostError, SnsLookupRequest,
    SnsParamsReport, live::LiveSnsSource, lookup::resolve_sns_lookup, source::SnsParamsSource,
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
    Ok(SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: lookup.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: lookup.fetch_request.fetched_at,
        source_endpoint: lookup.fetch_request.endpoint,
        fetched_by: lookup.fetch_request.fetched_by,
        id: lookup.sns.id,
        name: lookup.sns.name,
        root_canister_id: lookup.sns.canisters.root_canister_id,
        governance_canister_id: lookup.sns.canisters.governance_canister_id,
        parameters,
    })
}
