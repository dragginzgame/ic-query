//! Module: sns::report::build::upgrade
//!
//! Responsibility: build bounded SNS upgrade reports.
//! Does not own: command parsing, transport internals, or rendering.
//! Boundary: resolves SNS identity, validates one source result, and assembles the report.

use crate::sns::report::{
    SNS_UPGRADE_QUERY_COUNT, SNS_UPGRADE_REPORT_SCHEMA_VERSION, SnsHostError, SnsLookupRequest,
    SnsReportContext, SnsUpgradeReport, SnsUpgradeSource, live::LiveSnsSource,
    lookup::resolve_sns_lookup, source::canonicalize_mainnet_sns_upgrade,
};

/// Build a live bounded upgrade report for one deployed SNS.
pub async fn build_sns_upgrade_report(
    request: &SnsLookupRequest,
) -> Result<SnsUpgradeReport, SnsHostError> {
    build_sns_upgrade_report_with_source(request, &LiveSnsSource::default()).await
}

/// Build a bounded upgrade report through a custom SNS source capability.
pub async fn build_sns_upgrade_report_with_source(
    request: &SnsLookupRequest,
    source: &dyn SnsUpgradeSource,
) -> Result<SnsUpgradeReport, SnsHostError> {
    let lookup = resolve_sns_lookup(request, source).await?;
    let upgrade = source
        .fetch_sns_upgrade(&lookup.fetch_request, &lookup.sns)
        .await?;
    canonicalize_mainnet_sns_upgrade(&upgrade, &lookup.sns.canisters.governance_canister_id)?;
    let component_gap_count = usize::from(upgrade.next_version_gap.is_some());
    Ok(SnsUpgradeReport {
        schema_version: SNS_UPGRADE_REPORT_SCHEMA_VERSION,
        context: SnsReportContext::from_live(lookup.fetch_request, lookup.sns),
        running_version_method: upgrade.running_version_method,
        next_version_method: upgrade.next_version_method,
        point_in_time_guaranteed: upgrade.point_in_time_guaranteed,
        component_query_count: SNS_UPGRADE_QUERY_COUNT,
        successful_component_query_count: SNS_UPGRADE_QUERY_COUNT - component_gap_count,
        component_gap_count,
        deployed_version: upgrade.deployed_version,
        pending_upgrade: upgrade.pending_upgrade,
        next_version: upgrade.next_version,
        next_version_gap: upgrade.next_version_gap,
    })
}
