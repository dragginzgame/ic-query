//! Module: sns::report::assemble::canisters
//!
//! Responsibility: assemble SNS Root canister report DTOs.
//! Does not own: lookup, Root transport, source conversion, or rendering.
//! Boundary: combines discovery provenance with joined inventory and health evidence.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsCanisterInventory,
    SNS_CANISTER_REPORT_SCHEMA_VERSION, SnsCanisterCycleBalanceStatus, SnsCanisterReport,
    SnsSourceRequest,
};

pub(in crate::sns::report) fn sns_canister_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    inventory: MainnetSnsCanisterInventory,
) -> SnsCanisterReport {
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
    SnsCanisterReport {
        schema_version: SNS_CANISTER_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        inventory_method: inventory.inventory_method,
        health_method: inventory.health_method,
        health_call_type: inventory.health_call_type,
        health_update_canister_list: inventory.health_update_canister_list,
        point_in_time_guaranteed: inventory.point_in_time_guaranteed,
        canister_count: inventory.canisters.len(),
        health_status_count,
        reported_zero_cycles_count,
        cycles_unavailable_count,
        gap_count: inventory.gaps.len(),
        health_query_gap: inventory.health_query_gap,
        canisters: inventory.canisters,
        gaps: inventory.gaps,
    }
}
