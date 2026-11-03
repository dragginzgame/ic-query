//! Module: sns::report::build::canisters
//!
//! Responsibility: build SNS Root canister inventory and health reports.
//! Does not own: command parsing, Root transport internals, or rendering.
//! Boundary: resolves SNS identity, calls one source capability, and assembles the report.

use crate::sns::report::{
    SNS_CANISTER_REPORT_SCHEMA_VERSION, SnsCanisterCycleBalanceStatus, SnsCanisterReport,
    SnsCanisterSource, SnsHostError, SnsLookupRequest, live::LiveSnsSource,
    lookup::resolve_sns_lookup, source::canonicalize_mainnet_sns_canister_inventory,
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
    let health_status_count = inventory
        .canisters
        .iter()
        .filter(|canister| canister.status.is_some())
        .count();
    let reported_zero_cycles_count = inventory
        .canisters
        .iter()
        .filter(|canister| {
            canister.cycle_balance_status == SnsCanisterCycleBalanceStatus::ReportedZero
        })
        .count();
    let cycles_unavailable_count = inventory
        .canisters
        .iter()
        .filter(|canister| {
            canister.cycle_balance_status == SnsCanisterCycleBalanceStatus::Unavailable
        })
        .count();
    Ok(SnsCanisterReport {
        schema_version: SNS_CANISTER_REPORT_SCHEMA_VERSION,
        context: crate::sns::SnsInstanceContext::from_live(
            lookup.fetch_request,
            lookup.sns.id,
            lookup.sns.name,
            lookup.sns.canisters.root_canister_id,
        ),
        health_status_count,
        reported_zero_cycles_count,
        cycles_unavailable_count,
        inventory,
    })
}
