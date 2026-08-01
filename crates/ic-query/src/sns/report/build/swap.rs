//! Module: sns::report::build::swap
//!
//! Responsibility: build bounded SNS swap reports.
//! Does not own: command parsing, swap transport internals, DTO assembly, or rendering.
//! Boundary: resolves SNS identity, validates one source result, and delegates assembly.

use crate::sns::report::{
    SnsHostError, SnsLookupRequest, SnsSwapReport, SnsSwapSource,
    assemble::sns_swap_report_from_parts, live::LiveSnsSource, lookup::resolve_sns_lookup,
    source::canonicalize_mainnet_sns_swap,
};

/// Build a live bounded swap report for one deployed SNS.
pub fn build_sns_swap_report(request: &SnsLookupRequest) -> Result<SnsSwapReport, SnsHostError> {
    build_sns_swap_report_with_source(request, &LiveSnsSource)
}

/// Build a bounded swap report through a custom SNS source capability.
pub fn build_sns_swap_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsSwapSource,
) -> Result<SnsSwapReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let mut swap = source.fetch_sns_swap(&lookup.fetch_request, &lookup.sns)?;
    canonicalize_mainnet_sns_swap(&mut swap, &lookup.sns.swap_canister_id)?;
    Ok(sns_swap_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        swap,
    ))
}
