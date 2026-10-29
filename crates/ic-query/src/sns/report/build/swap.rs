//! Module: sns::report::build::swap
//!
//! Responsibility: build bounded SNS swap reports.
//! Does not own: command parsing, swap transport internals, or rendering.
//! Boundary: resolves SNS identity, validates one source result, and assembles the report.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsSwap, SNS_SWAP_QUERY_COUNT,
    SNS_SWAP_REPORT_SCHEMA_VERSION, SnsHostError, SnsLookupRequest, SnsSourceRequest,
    SnsSwapReport, SnsSwapSource, live::LiveSnsSource, lookup::resolve_sns_lookup,
    source::canonicalize_mainnet_sns_swap,
};

/// Build a live bounded swap report for one deployed SNS.
pub async fn build_sns_swap_report(
    request: &SnsLookupRequest,
) -> Result<SnsSwapReport, SnsHostError> {
    build_sns_swap_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build a bounded swap report through a custom SNS source capability.
pub async fn build_sns_swap_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsSwapSource,
) -> Result<SnsSwapReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    let mut swap = source
        .fetch_sns_swap(&lookup.fetch_request, &lookup.sns)
        .await?;
    canonicalize_mainnet_sns_swap(&mut swap, &lookup.sns.canisters.swap_canister_id)?;
    Ok(sns_swap_report_from_parts(
        lookup.fetch_request,
        lookup.sns,
        swap,
    ))
}

fn sns_swap_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    swap: MainnetSnsSwap,
) -> SnsSwapReport {
    let component_gap_count = swap.gaps.len();
    SnsSwapReport {
        schema_version: SNS_SWAP_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.canisters.root_canister_id,
        component_query_count: SNS_SWAP_QUERY_COUNT,
        successful_component_query_count: SNS_SWAP_QUERY_COUNT - component_gap_count,
        component_gap_count,
        swap,
    }
}
