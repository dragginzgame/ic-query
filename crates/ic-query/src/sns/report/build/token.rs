//! Module: sns::report::build::token
//!
//! Responsibility: build SNS token reports.
//! Does not own: command parsing, ledger transport internals, or rendering.
//! Boundary: resolves SNS identity, fetches token data, and assembles the report.

use crate::icrc::IcrcIndexData;
use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSnsToken, SNS_TOKEN_REPORT_SCHEMA_VERSION, SnsHostError,
    SnsLookupRequest, SnsTokenReport, live::LiveSnsSource, lookup::resolve_sns_lookup,
    source::SnsTokenSource,
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
    let MainnetSnsToken {
        token,
        ledger_index,
    } = token;
    let (ledger_index_canister_id, ledger_index_error) = match ledger_index {
        IcrcIndexData::Found(principal) => (Some(principal), None),
        IcrcIndexData::Unavailable(error) => (None, Some(error)),
    };
    Ok(SnsTokenReport {
        schema_version: SNS_TOKEN_REPORT_SCHEMA_VERSION,
        network: lookup.fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: lookup.fetch_request.fetched_at,
        source_endpoint: lookup.fetch_request.endpoint,
        fetched_by: lookup.fetch_request.fetched_by,
        id: lookup.sns.id,
        name: lookup.sns.name,
        root_canister_id: lookup.sns.canisters.root_canister_id,
        ledger_canister_id: lookup.sns.canisters.ledger_canister_id,
        sns_index_canister_id: lookup.sns.canisters.index_canister_id,
        token,
        ledger_index_canister_id,
        ledger_index_error,
    })
}
