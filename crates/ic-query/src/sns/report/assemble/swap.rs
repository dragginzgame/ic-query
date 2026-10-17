//! Module: sns::report::assemble::swap
//!
//! Responsibility: assemble SNS swap report DTOs.
//! Does not own: lookup, swap calls, source validation, or rendering.
//! Boundary: combines discovery provenance with bounded component values and typed gaps.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, MainnetSnsSwap, SNS_SWAP_QUERY_COUNT,
    SNS_SWAP_REPORT_SCHEMA_VERSION, SnsSourceRequest, SnsSwapReport,
};

pub(in crate::sns::report) fn sns_swap_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    swap: MainnetSnsSwap,
) -> SnsSwapReport {
    let component_gap_count = swap.gaps.len();
    SnsSwapReport {
        schema_version: SNS_SWAP_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.canisters.root_canister_id,
        component_query_count: SNS_SWAP_QUERY_COUNT,
        successful_component_query_count: SNS_SWAP_QUERY_COUNT - component_gap_count,
        component_gap_count,
        swap,
    }
}
