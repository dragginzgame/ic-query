//! Module: nns::governance::wire
//!
//! Responsibility: decode and project native NNS Governance metric responses.
//! Does not own: transport, report provenance, or public rendering.
//! Boundary: exists only where unlabeled Candid buckets differ from named public JSON rows.

use super::{
    NnsGovernanceMaturityModulation, NnsGovernanceMetricBucket, NnsGovernanceMetrics,
    NnsGovernanceNeuronSubsetMetrics,
};
use candid::{CandidType, Deserialize};

///
/// GovernanceCachedMetrics
///
/// Native Candid cached Governance metrics response.
///

#[cfg_attr(test, derive(Default))]
#[derive(CandidType, Deserialize)]
pub(super) struct GovernanceCachedMetrics {
    pub(super) total_maturity_e8s_equivalent: u64,
    pub(super) not_dissolving_neurons_e8s_buckets: Vec<(u64, f64)>,
    pub(super) dissolving_neurons_staked_maturity_e8s_equivalent_sum: u64,
    pub(super) garbage_collectable_neurons_count: u64,
    pub(super) dissolving_neurons_staked_maturity_e8s_equivalent_buckets: Vec<(u64, f64)>,
    pub(super) neurons_with_invalid_stake_count: u64,
    pub(super) not_dissolving_neurons_count_buckets: Vec<(u64, u64)>,
    pub(super) ect_neuron_count: u64,
    pub(super) total_supply_icp: u64,
    pub(super) neurons_with_less_than_6_months_dissolve_delay_count: u64,
    pub(super) dissolved_neurons_count: u64,
    pub(super) community_fund_total_maturity_e8s_equivalent: u64,
    pub(super) total_staked_e8s_seed: u64,
    pub(super) total_staked_maturity_e8s_equivalent_ect: u64,
    pub(super) total_staked_e8s: u64,
    pub(super) not_dissolving_neurons_count: u64,
    pub(super) total_locked_e8s: u64,
    pub(super) neurons_fund_total_active_neurons: u64,
    pub(super) total_voting_power_non_self_authenticating_controller: Option<u64>,
    pub(super) total_staked_maturity_e8s_equivalent: u64,
    pub(super) not_dissolving_neurons_e8s_buckets_ect: Vec<(u64, f64)>,
    pub(super) total_staked_e8s_ect: u64,
    pub(super) not_dissolving_neurons_staked_maturity_e8s_equivalent_sum: u64,
    pub(super) dissolved_neurons_e8s: u64,
    pub(super) total_staked_e8s_non_self_authenticating_controller: Option<u64>,
    pub(super) dissolving_neurons_e8s_buckets_seed: Vec<(u64, f64)>,
    pub(super) neurons_with_less_than_6_months_dissolve_delay_e8s: u64,
    pub(super) not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets: Vec<(u64, f64)>,
    pub(super) dissolving_neurons_count_buckets: Vec<(u64, u64)>,
    pub(super) dissolving_neurons_e8s_buckets_ect: Vec<(u64, f64)>,
    pub(super) dissolving_neurons_count: u64,
    pub(super) dissolving_neurons_e8s_buckets: Vec<(u64, f64)>,
    pub(super) total_staked_maturity_e8s_equivalent_seed: u64,
    pub(super) community_fund_total_staked_e8s: u64,
    pub(super) not_dissolving_neurons_e8s_buckets_seed: Vec<(u64, f64)>,
    pub(super) timestamp_seconds: u64,
    pub(super) seed_neuron_count: u64,
    pub(super) spawning_neurons_count: u64,
    pub(super) total_maturity_disbursements_in_progress_e8s_equivalent: u64,
    pub(super) non_self_authenticating_controller_neuron_subset_metrics:
        Option<NeuronSubsetMetrics>,
    pub(super) public_neuron_subset_metrics: Option<NeuronSubsetMetrics>,
    pub(super) declining_voting_power_neuron_subset_metrics: Option<NeuronSubsetMetrics>,
    pub(super) fully_lost_voting_power_neuron_subset_metrics: Option<NeuronSubsetMetrics>,
}

///
/// NeuronSubsetMetrics
///
/// Native Candid metrics for one neuron subset.
///

#[cfg_attr(test, derive(Default))]
#[derive(CandidType, Deserialize)]
pub(super) struct NeuronSubsetMetrics {
    pub(super) count: Option<u64>,
    pub(super) total_staked_e8s: Option<u64>,
    pub(super) total_maturity_e8s_equivalent: Option<u64>,
    pub(super) total_staked_maturity_e8s_equivalent: Option<u64>,
    pub(super) total_voting_power: Option<u64>,
    pub(super) total_deciding_voting_power: Option<u64>,
    pub(super) total_potential_voting_power: Option<u64>,
    pub(super) count_buckets: Vec<(u64, u64)>,
    pub(super) staked_e8s_buckets: Vec<(u64, u64)>,
    pub(super) maturity_e8s_equivalent_buckets: Vec<(u64, u64)>,
    pub(super) staked_maturity_e8s_equivalent_buckets: Vec<(u64, u64)>,
    pub(super) voting_power_buckets: Vec<(u64, u64)>,
    pub(super) deciding_voting_power_buckets: Vec<(u64, u64)>,
    pub(super) potential_voting_power_buckets: Vec<(u64, u64)>,
}

///
/// GovernanceError
///
/// Native Candid Governance application error.
///

#[derive(CandidType, Deserialize)]
pub(super) struct GovernanceError {
    pub(super) error_message: String,
    pub(super) error_type: i32,
}

///
/// GetMetricsResult
///
/// Native Candid result returned by the Governance metrics query.
///

#[derive(CandidType, Deserialize)]
pub(super) enum GetMetricsResult {
    Ok(Box<GovernanceCachedMetrics>),
    Err(GovernanceError),
}

///
/// GetMaturityModulationRequest
///
/// Native empty Candid request for current maturity modulation.
///

#[derive(CandidType, Deserialize)]
pub(super) struct GetMaturityModulationRequest {}

///
/// GetMaturityModulationResponse
///
/// Native Candid response containing optional current maturity modulation.
///

#[derive(CandidType, Deserialize)]
pub(super) struct GetMaturityModulationResponse {
    pub(super) maturity_modulation: Option<NnsGovernanceMaturityModulation>,
}

impl From<GovernanceCachedMetrics> for NnsGovernanceMetrics {
    fn from(value: GovernanceCachedMetrics) -> Self {
        Self {
            total_maturity_e8s_equivalent: value.total_maturity_e8s_equivalent,
            not_dissolving_neurons_e8s_buckets: metric_buckets(
                value.not_dissolving_neurons_e8s_buckets,
            ),
            dissolving_neurons_staked_maturity_e8s_equivalent_sum: value
                .dissolving_neurons_staked_maturity_e8s_equivalent_sum,
            garbage_collectable_neurons_count: value.garbage_collectable_neurons_count,
            dissolving_neurons_staked_maturity_e8s_equivalent_buckets: metric_buckets(
                value.dissolving_neurons_staked_maturity_e8s_equivalent_buckets,
            ),
            neurons_with_invalid_stake_count: value.neurons_with_invalid_stake_count,
            not_dissolving_neurons_count_buckets: metric_buckets(
                value.not_dissolving_neurons_count_buckets,
            ),
            ect_neuron_count: value.ect_neuron_count,
            total_supply_icp: value.total_supply_icp,
            neurons_with_less_than_6_months_dissolve_delay_count: value
                .neurons_with_less_than_6_months_dissolve_delay_count,
            dissolved_neurons_count: value.dissolved_neurons_count,
            community_fund_total_maturity_e8s_equivalent: value
                .community_fund_total_maturity_e8s_equivalent,
            total_staked_e8s_seed: value.total_staked_e8s_seed,
            total_staked_maturity_e8s_equivalent_ect: value
                .total_staked_maturity_e8s_equivalent_ect,
            total_staked_e8s: value.total_staked_e8s,
            not_dissolving_neurons_count: value.not_dissolving_neurons_count,
            total_locked_e8s: value.total_locked_e8s,
            neurons_fund_total_active_neurons: value.neurons_fund_total_active_neurons,
            total_voting_power_non_self_authenticating_controller: value
                .total_voting_power_non_self_authenticating_controller,
            total_staked_maturity_e8s_equivalent: value.total_staked_maturity_e8s_equivalent,
            not_dissolving_neurons_e8s_buckets_ect: metric_buckets(
                value.not_dissolving_neurons_e8s_buckets_ect,
            ),
            total_staked_e8s_ect: value.total_staked_e8s_ect,
            not_dissolving_neurons_staked_maturity_e8s_equivalent_sum: value
                .not_dissolving_neurons_staked_maturity_e8s_equivalent_sum,
            dissolved_neurons_e8s: value.dissolved_neurons_e8s,
            total_staked_e8s_non_self_authenticating_controller: value
                .total_staked_e8s_non_self_authenticating_controller,
            dissolving_neurons_e8s_buckets_seed: metric_buckets(
                value.dissolving_neurons_e8s_buckets_seed,
            ),
            neurons_with_less_than_6_months_dissolve_delay_e8s: value
                .neurons_with_less_than_6_months_dissolve_delay_e8s,
            not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets: metric_buckets(
                value.not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets,
            ),
            dissolving_neurons_count_buckets: metric_buckets(
                value.dissolving_neurons_count_buckets,
            ),
            dissolving_neurons_e8s_buckets_ect: metric_buckets(
                value.dissolving_neurons_e8s_buckets_ect,
            ),
            dissolving_neurons_count: value.dissolving_neurons_count,
            dissolving_neurons_e8s_buckets: metric_buckets(value.dissolving_neurons_e8s_buckets),
            total_staked_maturity_e8s_equivalent_seed: value
                .total_staked_maturity_e8s_equivalent_seed,
            community_fund_total_staked_e8s: value.community_fund_total_staked_e8s,
            not_dissolving_neurons_e8s_buckets_seed: metric_buckets(
                value.not_dissolving_neurons_e8s_buckets_seed,
            ),
            timestamp_seconds: value.timestamp_seconds,
            seed_neuron_count: value.seed_neuron_count,
            spawning_neurons_count: value.spawning_neurons_count,
            total_maturity_disbursements_in_progress_e8s_equivalent: value
                .total_maturity_disbursements_in_progress_e8s_equivalent,
            non_self_authenticating_controller_neuron_subset_metrics: value
                .non_self_authenticating_controller_neuron_subset_metrics
                .map(Into::into),
            public_neuron_subset_metrics: value.public_neuron_subset_metrics.map(Into::into),
            declining_voting_power_neuron_subset_metrics: value
                .declining_voting_power_neuron_subset_metrics
                .map(Into::into),
            fully_lost_voting_power_neuron_subset_metrics: value
                .fully_lost_voting_power_neuron_subset_metrics
                .map(Into::into),
        }
    }
}

impl From<NeuronSubsetMetrics> for NnsGovernanceNeuronSubsetMetrics {
    fn from(value: NeuronSubsetMetrics) -> Self {
        Self {
            count: value.count,
            total_staked_e8s: value.total_staked_e8s,
            total_maturity_e8s_equivalent: value.total_maturity_e8s_equivalent,
            total_staked_maturity_e8s_equivalent: value.total_staked_maturity_e8s_equivalent,
            total_voting_power: value.total_voting_power,
            total_deciding_voting_power: value.total_deciding_voting_power,
            total_potential_voting_power: value.total_potential_voting_power,
            count_buckets: metric_buckets(value.count_buckets),
            staked_e8s_buckets: metric_buckets(value.staked_e8s_buckets),
            maturity_e8s_equivalent_buckets: metric_buckets(value.maturity_e8s_equivalent_buckets),
            staked_maturity_e8s_equivalent_buckets: metric_buckets(
                value.staked_maturity_e8s_equivalent_buckets,
            ),
            voting_power_buckets: metric_buckets(value.voting_power_buckets),
            deciding_voting_power_buckets: metric_buckets(value.deciding_voting_power_buckets),
            potential_voting_power_buckets: metric_buckets(value.potential_voting_power_buckets),
        }
    }
}

fn metric_buckets<Value>(values: Vec<(u64, Value)>) -> Vec<NnsGovernanceMetricBucket<Value>> {
    values
        .into_iter()
        .map(|(key, value)| NnsGovernanceMetricBucket { key, value })
        .collect()
}
