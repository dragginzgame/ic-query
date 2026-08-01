//! Module: sns::report::assemble::upgrade
//!
//! Responsibility: assemble bounded SNS upgrade report DTOs.
//! Does not own: lookup, live calls, source validation, or rendering.
//! Boundary: combines discovery provenance with deployed, pending, and next version evidence.

use crate::sns::report::{
    JoinedMainnetSnsInventory, MainnetSns, MainnetSnsUpgrade, SNS_UPGRADE_QUERY_COUNT,
    SNS_UPGRADE_REPORT_SCHEMA_VERSION, SnsUpgradeReport,
};

pub(in crate::sns::report) fn sns_upgrade_report_from_parts(
    list: JoinedMainnetSnsInventory,
    id: usize,
    sns: MainnetSns,
    upgrade: MainnetSnsUpgrade,
) -> SnsUpgradeReport {
    let component_gap_count = usize::from(upgrade.next_version_gap.is_some());
    SnsUpgradeReport {
        schema_version: SNS_UPGRADE_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: upgrade.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: upgrade.governance_canister_id,
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
    }
}
