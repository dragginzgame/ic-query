//! Module: sns::report::build::upgrade
//!
//! Responsibility: build bounded SNS upgrade reports.
//! Does not own: command parsing, transport internals, DTO assembly, or rendering.
//! Boundary: resolves SNS identity, validates one source result, and delegates assembly.

use crate::sns::report::{
    SnsHostError, SnsLookupRequest, SnsUpgradeReport, SnsUpgradeSource,
    assemble::sns_upgrade_report_from_parts, live::LiveSnsSource, lookup::resolve_sns_lookup,
    source::canonicalize_mainnet_sns_upgrade,
};

/// Build a live bounded upgrade report for one deployed SNS.
pub fn build_sns_upgrade_report(
    request: &SnsLookupRequest,
) -> Result<SnsUpgradeReport, SnsHostError> {
    build_sns_upgrade_report_with_source(request, &LiveSnsSource)
}

/// Build a bounded upgrade report through a custom SNS source capability.
pub fn build_sns_upgrade_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsUpgradeSource,
) -> Result<SnsUpgradeReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source)?;
    let upgrade = source.fetch_sns_upgrade(&lookup.fetch_request, &lookup.sns)?;
    canonicalize_mainnet_sns_upgrade(&upgrade, &lookup.sns.governance_canister_id)?;
    Ok(sns_upgrade_report_from_parts(
        lookup.list,
        lookup.id,
        lookup.sns,
        upgrade,
    ))
}
