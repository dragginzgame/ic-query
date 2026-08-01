//! Module: sns::report::assemble::canisters
//!
//! Responsibility: assemble SNS Root canister report DTOs.
//! Does not own: lookup, Root transport, source conversion, or rendering.
//! Boundary: combines discovery provenance with joined inventory and health evidence.

use crate::sns::report::{
    JoinedMainnetSnsInventory, MainnetSns, MainnetSnsCanisterInventory,
    SNS_CANISTER_REPORT_SCHEMA_VERSION, SnsCanisterReport,
};

pub(in crate::sns::report) fn sns_canister_report_from_parts(
    list: JoinedMainnetSnsInventory,
    id: usize,
    sns: MainnetSns,
    inventory: MainnetSnsCanisterInventory,
) -> SnsCanisterReport {
    let health_status_count = inventory
        .canisters
        .iter()
        .filter(|canister| canister.status.is_some())
        .count();
    SnsCanisterReport {
        schema_version: SNS_CANISTER_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        inventory_method: inventory.inventory_method,
        health_method: inventory.health_method,
        health_call_type: inventory.health_call_type,
        health_update_canister_list: inventory.health_update_canister_list,
        point_in_time_guaranteed: inventory.point_in_time_guaranteed,
        canister_count: inventory.canisters.len(),
        health_status_count,
        gap_count: inventory.gaps.len(),
        canisters: inventory.canisters,
        gaps: inventory.gaps,
    }
}
