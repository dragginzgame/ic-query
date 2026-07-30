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
pub fn build_sns_canister_report(
    request: &SnsLookupRequest,
) -> Result<SnsCanisterReport, SnsHostError> {
    build_sns_canister_report_with_source(request, &LiveSnsSource)
}

/// Build an SNS Root canister report through a custom source capability.
pub fn build_sns_canister_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsCanisterSource,
) -> Result<SnsCanisterReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let mut inventory = source.fetch_sns_canisters(&lookup.fetch_request, &lookup.sns)?;
    canonicalize_mainnet_sns_canister_inventory(&mut inventory)?;
    Ok(sns_canister_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        inventory,
    ))
}
