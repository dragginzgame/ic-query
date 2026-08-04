//! Module: sns::report::model::reports::swap
//!
//! Responsibility: SNS decentralization-swap report DTOs.
//! Does not own: swap canister calls, source validation, lookup, or rendering.
//! Boundary: preserves native swap lifecycle, sale parameters, derived state, and query gaps.

use super::invocation::SnsCanisterMethod;
use serde::Serialize;

///
/// SnsSwapComponent
///
/// Native swap query component represented by a report value or typed gap.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsSwapComponent {
    /// Swap lifecycle state and lifecycle timestamps.
    Lifecycle,
    /// Decentralization-sale parameters.
    SaleParameters,
    /// Participation totals and derived token rate.
    DerivedState,
}

impl SnsSwapComponent {
    /// Return the stable lowercase component label used in text reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::SaleParameters => "sale_parameters",
            Self::DerivedState => "derived_state",
        }
    }
}

///
/// SnsSwapQueryGap
///
/// One swap query that failed while the other bounded components were retained.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsSwapQueryGap {
    /// Typed swap component whose query failed.
    pub component: SnsSwapComponent,
    /// Native query method that failed.
    pub method: SnsCanisterMethod,
    /// Transport, encoding, or decoding failure retained for diagnostics.
    pub reason: String,
}

///
/// SnsSwapLifecycle
///
/// Raw lifecycle code and timestamps returned by the SNS swap canister.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsSwapLifecycle {
    /// Optional native lifecycle numeric discriminant.
    pub lifecycle: Option<i32>,
    /// Native lifecycle label, or `unknown` for an unrecognized numeric discriminant.
    pub lifecycle_name: Option<String>,
    /// Timestamp at which the decentralization sale opened, when returned.
    pub decentralization_sale_open_timestamp_seconds: Option<u64>,
    /// Timestamp at which the decentralization swap terminated, when returned.
    pub decentralization_swap_termination_timestamp_seconds: Option<u64>,
}

///
/// SnsSwapNeuronBasketConstructionParameters
///
/// Native neuron-basket construction parameters returned by the swap canister.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsSwapNeuronBasketConstructionParameters {
    /// Number of neurons created in each participant basket.
    pub count: u64,
    /// Dissolve-delay interval between neurons in a basket, in seconds.
    pub dissolve_delay_interval_seconds: u64,
}

///
/// SnsSwapSaleParameters
///
/// Native decentralization-sale parameters returned by the swap canister.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsSwapSaleParameters {
    /// Minimum legacy total ICP target in e8s.
    pub min_icp_e8s: u64,
    /// Maximum legacy total ICP target in e8s.
    pub max_icp_e8s: u64,
    /// Minimum direct participation target in ICP e8s, when returned.
    pub min_direct_participation_icp_e8s: Option<u64>,
    /// Maximum direct participation target in ICP e8s, when returned.
    pub max_direct_participation_icp_e8s: Option<u64>,
    /// SNS tokens offered by the sale in e8s.
    pub sns_token_e8s: u64,
    /// Minimum number of direct participants.
    pub min_participants: u32,
    /// Minimum ICP contribution per participant in e8s.
    pub min_participant_icp_e8s: u64,
    /// Maximum ICP contribution per participant in e8s.
    pub max_participant_icp_e8s: u64,
    /// Scheduled swap deadline in Unix seconds.
    pub swap_due_timestamp_seconds: u64,
    /// Optional delay before the sale opens, in seconds.
    pub sale_delay_seconds: Option<u64>,
    /// Neuron-basket construction parameters, when configured.
    pub neuron_basket_construction_parameters: Option<SnsSwapNeuronBasketConstructionParameters>,
}

///
/// SnsSwapDerivedState
///
/// Native aggregate participation values returned by the swap canister.
///

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SnsSwapDerivedState {
    /// Derived SNS-token amount per ICP, when returned.
    pub sns_tokens_per_icp: Option<f64>,
    /// Total buyer ICP in e8s, when returned.
    pub buyer_total_icp_e8s: Option<u64>,
    /// Direct-participation ICP in e8s, when returned.
    pub direct_participation_icp_e8s: Option<u64>,
    /// Neurons' Fund participation ICP in e8s, when returned.
    pub neurons_fund_participation_icp_e8s: Option<u64>,
    /// Number of direct participants, when returned.
    pub direct_participant_count: Option<u64>,
    /// Number of Community Fund participants, when returned by older swap state.
    pub cf_participant_count: Option<u64>,
    /// Number of Community Fund neurons, when returned by older swap state.
    pub cf_neuron_count: Option<u64>,
}

///
/// SnsSwapReport
///
/// Bounded live report for one resolved SNS decentralization swap.
///

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SnsSwapReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Requested IC network identity.
    pub network: String,
    /// Mainnet SNS-W canister used to resolve the SNS.
    pub sns_wasm_canister_id: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// IC API endpoint used for SNS-W and swap calls.
    pub source_endpoint: String,
    /// Collector identity recorded by the source request.
    pub fetched_by: String,
    /// SNS-W list id assigned to this deployed SNS.
    pub id: usize,
    /// SNS name resolved during discovery.
    pub name: String,
    /// Root canister identity used to resolve this SNS.
    pub root_canister_id: String,
    /// Swap canister queried for lifecycle and sale state.
    pub swap_canister_id: String,
    /// Native method used for lifecycle state.
    pub lifecycle_method: SnsCanisterMethod,
    /// Native method used for sale parameters.
    pub sale_parameters_method: SnsCanisterMethod,
    /// Native method used for derived participation state.
    pub derived_state_method: SnsCanisterMethod,
    /// Whether all component values represent one authoritative point in time.
    pub point_in_time_guaranteed: bool,
    /// Fixed number of bounded swap component queries attempted.
    pub component_query_count: usize,
    /// Number of component queries that returned successfully.
    pub successful_component_query_count: usize,
    /// Number of typed component query gaps.
    pub component_gap_count: usize,
    /// Lifecycle response, absent only when its query failed.
    pub lifecycle: Option<SnsSwapLifecycle>,
    /// Sale parameters; `None` can also be a successful response with no configured parameters.
    pub sale_parameters: Option<SnsSwapSaleParameters>,
    /// Derived-state response, absent only when its query failed.
    pub derived_state: Option<SnsSwapDerivedState>,
    /// Canonically ordered component query failures.
    pub gaps: Vec<SnsSwapQueryGap>,
}
