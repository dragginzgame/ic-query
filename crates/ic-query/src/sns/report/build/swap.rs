//! Module: sns::report::build::swap
//!
//! Responsibility: build bounded SNS swap reports.
//! Does not own: command parsing, swap transport internals, or rendering.
//! Boundary: resolves SNS identity, validates one source result, and assembles the report.

use crate::sns::report::{
    SNS_SWAP_QUERY_COUNT, SNS_SWAP_REPORT_SCHEMA_VERSION, SnsHostError, SnsLookupRequest,
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
    let component_gap_count = swap.gaps.len();
    Ok(SnsSwapReport {
        schema_version: SNS_SWAP_REPORT_SCHEMA_VERSION,
        context: crate::sns::SnsInstanceContext::from_live(
            lookup.fetch_request,
            lookup.sns.id,
            lookup.sns.name,
            lookup.sns.canisters.root_canister_id,
        ),
        component_query_count: SNS_SWAP_QUERY_COUNT,
        successful_component_query_count: SNS_SWAP_QUERY_COUNT - component_gap_count,
        component_gap_count,
        swap,
    })
}
