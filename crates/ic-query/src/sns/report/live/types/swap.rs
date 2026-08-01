//! Module: sns::report::live::types::swap
//!
//! Responsibility: SNS swap Candid request and response wire types.
//! Does not own: transport, source validation, report DTOs, or rendering.
//! Boundary: mirrors the bounded public swap query fields consumed by the live adapter.

use candid::{CandidType, Deserialize};

///
/// SnsSwapQueryRequest
///
/// Empty record accepted by the bounded SNS swap query methods.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsSwapQueryRequest {}

///
/// GetLifecycleResponse
///
/// Native `get_lifecycle` response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetLifecycleResponse {
    pub(in crate::sns::report::live) lifecycle: Option<i32>,
    pub(in crate::sns::report::live) decentralization_sale_open_timestamp_seconds: Option<u64>,
    pub(in crate::sns::report::live) decentralization_swap_termination_timestamp_seconds:
        Option<u64>,
}

///
/// GetSaleParametersResponse
///
/// Native `get_sale_parameters` response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetSaleParametersResponse {
    pub(in crate::sns::report::live) params: Option<SnsSwapParams>,
}

///
/// SnsSwapParams
///
/// Native swap sale parameters.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsSwapParams {
    pub(in crate::sns::report::live) min_participant_icp_e8s: u64,
    pub(in crate::sns::report::live) neuron_basket_construction_parameters:
        Option<SnsSwapNeuronBasketConstructionParameters>,
    pub(in crate::sns::report::live) max_icp_e8s: u64,
    pub(in crate::sns::report::live) swap_due_timestamp_seconds: u64,
    pub(in crate::sns::report::live) min_participants: u32,
    pub(in crate::sns::report::live) sns_token_e8s: u64,
    pub(in crate::sns::report::live) sale_delay_seconds: Option<u64>,
    pub(in crate::sns::report::live) max_participant_icp_e8s: u64,
    pub(in crate::sns::report::live) min_direct_participation_icp_e8s: Option<u64>,
    pub(in crate::sns::report::live) min_icp_e8s: u64,
    pub(in crate::sns::report::live) max_direct_participation_icp_e8s: Option<u64>,
}

///
/// SnsSwapNeuronBasketConstructionParameters
///
/// Native swap neuron-basket construction parameters.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsSwapNeuronBasketConstructionParameters {
    pub(in crate::sns::report::live) dissolve_delay_interval_seconds: u64,
    pub(in crate::sns::report::live) count: u64,
}

///
/// GetDerivedStateResponse
///
/// Native `get_derived_state` response.
///

#[derive(CandidType, Clone, Debug, Deserialize, PartialEq)]
pub(in crate::sns::report::live) struct GetDerivedStateResponse {
    pub(in crate::sns::report::live) sns_tokens_per_icp: Option<f64>,
    pub(in crate::sns::report::live) buyer_total_icp_e8s: Option<u64>,
    pub(in crate::sns::report::live) cf_participant_count: Option<u64>,
    pub(in crate::sns::report::live) neurons_fund_participation_icp_e8s: Option<u64>,
    pub(in crate::sns::report::live) direct_participation_icp_e8s: Option<u64>,
    pub(in crate::sns::report::live) direct_participant_count: Option<u64>,
    pub(in crate::sns::report::live) cf_neuron_count: Option<u64>,
}
