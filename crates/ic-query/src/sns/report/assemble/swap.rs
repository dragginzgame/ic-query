//! Module: sns::report::assemble::swap
//!
//! Responsibility: assemble SNS swap report DTOs.
//! Does not own: lookup, swap calls, source validation, or rendering.
//! Boundary: combines discovery provenance with bounded component values and typed gaps.

use crate::sns::report::{
    JoinedMainnetSnsInventory, MainnetSns, MainnetSnsSwap, SNS_SWAP_QUERY_COUNT,
    SNS_SWAP_REPORT_SCHEMA_VERSION, SnsSwapReport,
};

pub(in crate::sns::report) fn sns_swap_report_from_parts(
    list: JoinedMainnetSnsInventory,
    id: usize,
    sns: MainnetSns,
    swap: MainnetSnsSwap,
) -> SnsSwapReport {
    let component_gap_count = swap.gaps.len();
    SnsSwapReport {
        schema_version: SNS_SWAP_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        swap_canister_id: swap.swap_canister_id,
        lifecycle_method: swap.lifecycle_method,
        sale_parameters_method: swap.sale_parameters_method,
        derived_state_method: swap.derived_state_method,
        point_in_time_guaranteed: swap.point_in_time_guaranteed,
        component_query_count: SNS_SWAP_QUERY_COUNT,
        successful_component_query_count: SNS_SWAP_QUERY_COUNT - component_gap_count,
        component_gap_count,
        lifecycle: swap.lifecycle,
        sale_parameters: swap.sale_parameters,
        derived_state: swap.derived_state,
        gaps: swap.gaps,
    }
}
