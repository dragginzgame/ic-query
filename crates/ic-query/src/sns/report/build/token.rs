//! Module: sns::report::build::token
//!
//! Responsibility: build SNS token reports.
//! Does not own: command parsing, ledger transport internals, or rendering.
//! Boundary: resolves SNS identity, fetches token data, and assembles the report.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsToken, SNS_TOKEN_REPORT_SCHEMA_VERSION,
    SnsHostError, SnsLookupRequest, SnsSourceRequest, SnsTokenReport, live::LiveSnsSource,
    lookup::resolve_sns_lookup, source::SnsTokenSource,
};

pub async fn build_sns_token_report(
    request: &SnsLookupRequest,
) -> Result<SnsTokenReport, SnsHostError> {
    build_sns_token_report_with_source(request, &LiveSnsSource::default()).await
}

pub async fn build_sns_token_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsTokenSource,
) -> Result<SnsTokenReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    let token = source
        .fetch_sns_token(&lookup.fetch_request, &lookup.sns)
        .await?;
    Ok(sns_token_report_from_parts(
        lookup.fetch_request,
        lookup.sns,
        token,
    ))
}

fn sns_token_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    token: MainnetSnsToken,
) -> SnsTokenReport {
    let MainnetSnsToken {
        token,
        ledger_index_canister_id,
        ledger_index_error,
    } = token;
    SnsTokenReport {
        schema_version: SNS_TOKEN_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.canisters.root_canister_id,
        ledger_canister_id: sns.canisters.ledger_canister_id,
        sns_index_canister_id: sns.canisters.index_canister_id,
        token,
        ledger_index_canister_id,
        ledger_index_error,
    }
}
