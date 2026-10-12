//! Module: sns::report::build::canisters
//!
//! Responsibility: build SNS Root canister inventory and health reports.
//! Does not own: command parsing, Root transport internals, DTO assembly, or rendering.
//! Boundary: resolves SNS identity, calls one source capability, and assembles the report.

use crate::sns::report::{
    SnsCanisterReport, SnsCanisterSource, SnsHostError, SnsLookupRequest,
    assemble::sns_canister_report_from_parts, live::LiveSnsSource, lookup::resolve_sns_lookup,
    source::canonicalize_mainnet_sns_canister_inventory,
};

/// Build a live SNS Root canister inventory and health report.
pub async fn build_sns_canister_report(
    request: &SnsLookupRequest,
) -> Result<SnsCanisterReport, SnsHostError> {
    build_sns_canister_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build an SNS Root canister report through a custom source capability.
pub async fn build_sns_canister_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsCanisterSource,
) -> Result<SnsCanisterReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    let mut inventory = source
        .fetch_sns_canisters(&lookup.fetch_request, &lookup.sns)
        .await?;
    canonicalize_mainnet_sns_canister_inventory(&mut inventory)?;
    Ok(sns_canister_report_from_parts(
        lookup.fetch_request,
        lookup.sns,
        inventory,
    ))
}
