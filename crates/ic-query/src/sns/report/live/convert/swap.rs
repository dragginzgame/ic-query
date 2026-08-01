//! Module: sns::report::live::convert::swap
//!
//! Responsibility: convert SNS swap wire responses into report-safe native DTOs.
//! Does not own: live calls, lookup, source validation, or rendering.
//! Boundary: preserves raw optional values and derives only the stable native lifecycle label.

use crate::sns::report::{
    SnsSwapDerivedState, SnsSwapLifecycle, SnsSwapNeuronBasketConstructionParameters,
    SnsSwapSaleParameters,
    live::types::{
        GetDerivedStateResponse, GetLifecycleResponse, GetSaleParametersResponse, SnsSwapParams,
    },
    source::sns_swap_lifecycle_name,
};

pub(in crate::sns::report::live) fn sns_swap_lifecycle(
    response: GetLifecycleResponse,
) -> SnsSwapLifecycle {
    SnsSwapLifecycle {
        lifecycle: response.lifecycle,
        lifecycle_name: sns_swap_lifecycle_name(response.lifecycle).map(str::to_string),
        decentralization_sale_open_timestamp_seconds: response
            .decentralization_sale_open_timestamp_seconds,
        decentralization_swap_termination_timestamp_seconds: response
            .decentralization_swap_termination_timestamp_seconds,
    }
}

pub(in crate::sns::report::live) fn sns_swap_sale_parameters(
    response: GetSaleParametersResponse,
) -> Option<SnsSwapSaleParameters> {
    response.params.map(sns_swap_params)
}

pub(in crate::sns::report::live) const fn sns_swap_derived_state(
    response: GetDerivedStateResponse,
) -> SnsSwapDerivedState {
    SnsSwapDerivedState {
        sns_tokens_per_icp: response.sns_tokens_per_icp,
        buyer_total_icp_e8s: response.buyer_total_icp_e8s,
        direct_participation_icp_e8s: response.direct_participation_icp_e8s,
        neurons_fund_participation_icp_e8s: response.neurons_fund_participation_icp_e8s,
        direct_participant_count: response.direct_participant_count,
        cf_participant_count: response.cf_participant_count,
        cf_neuron_count: response.cf_neuron_count,
    }
}

fn sns_swap_params(params: SnsSwapParams) -> SnsSwapSaleParameters {
    SnsSwapSaleParameters {
        min_icp_e8s: params.min_icp_e8s,
        max_icp_e8s: params.max_icp_e8s,
        min_direct_participation_icp_e8s: params.min_direct_participation_icp_e8s,
        max_direct_participation_icp_e8s: params.max_direct_participation_icp_e8s,
        sns_token_e8s: params.sns_token_e8s,
        min_participants: params.min_participants,
        min_participant_icp_e8s: params.min_participant_icp_e8s,
        max_participant_icp_e8s: params.max_participant_icp_e8s,
        swap_due_timestamp_seconds: params.swap_due_timestamp_seconds,
        sale_delay_seconds: params.sale_delay_seconds,
        neuron_basket_construction_parameters: params.neuron_basket_construction_parameters.map(
            |basket| SnsSwapNeuronBasketConstructionParameters {
                count: basket.count,
                dissolve_delay_interval_seconds: basket.dissolve_delay_interval_seconds,
            },
        ),
    }
}
