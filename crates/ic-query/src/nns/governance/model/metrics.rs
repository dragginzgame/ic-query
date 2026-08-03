//! Module: nns::governance::model::metrics
//!
//! Responsibility: native NNS Governance cached-metrics report contracts.
//! Does not own: economics, reward events, maturity modulation, transport, or rendering.
//! Boundary: preserves metric buckets, aggregate values, and native neuron subsets.

use super::NnsGovernanceReportContext;
use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// NnsGovernanceMetricBucket
///
/// One native Governance metric bucket represented as a named key/value row.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceMetricBucket<Value> {
    /// Raw unlabeled Candid bucket key.
    pub key: u64,
    /// Raw unlabeled Candid bucket value.
    pub value: Value,
}

///
/// NnsGovernanceMetricsReport
///
/// Serializable live snapshot of cached NNS Governance metrics.
///

#[derive(Clone, Debug, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceMetricsReport {
    /// Shared Governance query provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Native Governance metrics.
    pub metrics: NnsGovernanceMetrics,
}

///
/// NnsGovernanceMetrics
///
/// Native cached metrics returned by the NNS Governance canister.
///

#[derive(Clone, Debug, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceMetrics {
    /// Total maturity in e8s-equivalent.
    pub total_maturity_e8s_equivalent: u64,
    /// Non-dissolving neuron stake buckets.
    pub not_dissolving_neurons_e8s_buckets: Vec<NnsGovernanceMetricBucket<f64>>,
    /// Staked maturity of dissolving neurons in e8s-equivalent.
    pub dissolving_neurons_staked_maturity_e8s_equivalent_sum: u64,
    /// Number of garbage-collectable neurons.
    pub garbage_collectable_neurons_count: u64,
    /// Staked-maturity buckets for dissolving neurons.
    pub dissolving_neurons_staked_maturity_e8s_equivalent_buckets:
        Vec<NnsGovernanceMetricBucket<f64>>,
    /// Number of neurons with invalid stake.
    pub neurons_with_invalid_stake_count: u64,
    /// Count buckets for non-dissolving neurons.
    pub not_dissolving_neurons_count_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Number of early-contributor-token neurons.
    pub ect_neuron_count: u64,
    /// Total ICP supply reported by Governance.
    pub total_supply_icp: u64,
    /// Number of neurons with less than six months dissolve delay.
    pub neurons_with_less_than_6_months_dissolve_delay_count: u64,
    /// Number of dissolved neurons.
    pub dissolved_neurons_count: u64,
    /// Community Fund maturity in e8s-equivalent.
    pub community_fund_total_maturity_e8s_equivalent: u64,
    /// Total seed-neuron stake in e8s.
    pub total_staked_e8s_seed: u64,
    /// Total staked maturity of early-contributor-token neurons.
    pub total_staked_maturity_e8s_equivalent_ect: u64,
    /// Total neuron stake in e8s.
    pub total_staked_e8s: u64,
    /// Number of non-dissolving neurons.
    pub not_dissolving_neurons_count: u64,
    /// Total locked stake in e8s.
    pub total_locked_e8s: u64,
    /// Number of active Neurons' Fund neurons.
    pub neurons_fund_total_active_neurons: u64,
    /// Voting power controlled by non-self-authenticating principals.
    pub total_voting_power_non_self_authenticating_controller: Option<u64>,
    /// Total staked maturity in e8s-equivalent.
    pub total_staked_maturity_e8s_equivalent: u64,
    /// Non-dissolving early-contributor-token neuron stake buckets.
    pub not_dissolving_neurons_e8s_buckets_ect: Vec<NnsGovernanceMetricBucket<f64>>,
    /// Total stake of early-contributor-token neurons in e8s.
    pub total_staked_e8s_ect: u64,
    /// Staked maturity of non-dissolving neurons in e8s-equivalent.
    pub not_dissolving_neurons_staked_maturity_e8s_equivalent_sum: u64,
    /// Total dissolved-neuron stake in e8s.
    pub dissolved_neurons_e8s: u64,
    /// Stake controlled by non-self-authenticating principals.
    pub total_staked_e8s_non_self_authenticating_controller: Option<u64>,
    /// Dissolving seed-neuron stake buckets.
    pub dissolving_neurons_e8s_buckets_seed: Vec<NnsGovernanceMetricBucket<f64>>,
    /// Stake of neurons with less than six months dissolve delay.
    pub neurons_with_less_than_6_months_dissolve_delay_e8s: u64,
    /// Staked-maturity buckets for non-dissolving neurons.
    pub not_dissolving_neurons_staked_maturity_e8s_equivalent_buckets:
        Vec<NnsGovernanceMetricBucket<f64>>,
    /// Count buckets for dissolving neurons.
    pub dissolving_neurons_count_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Dissolving early-contributor-token neuron stake buckets.
    pub dissolving_neurons_e8s_buckets_ect: Vec<NnsGovernanceMetricBucket<f64>>,
    /// Number of dissolving neurons.
    pub dissolving_neurons_count: u64,
    /// Dissolving neuron stake buckets.
    pub dissolving_neurons_e8s_buckets: Vec<NnsGovernanceMetricBucket<f64>>,
    /// Total staked maturity of seed neurons.
    pub total_staked_maturity_e8s_equivalent_seed: u64,
    /// Total Community Fund stake in e8s.
    pub community_fund_total_staked_e8s: u64,
    /// Non-dissolving seed-neuron stake buckets.
    pub not_dissolving_neurons_e8s_buckets_seed: Vec<NnsGovernanceMetricBucket<f64>>,
    /// Governance metric collection timestamp in Unix seconds.
    pub timestamp_seconds: u64,
    /// Number of seed neurons.
    pub seed_neuron_count: u64,
    /// Number of spawning neurons.
    pub spawning_neurons_count: u64,
    /// Maturity disbursements currently in progress.
    pub total_maturity_disbursements_in_progress_e8s_equivalent: u64,
    /// Metrics for neurons controlled by non-self-authenticating principals.
    pub non_self_authenticating_controller_neuron_subset_metrics:
        Option<NnsGovernanceNeuronSubsetMetrics>,
    /// Metrics for publicly visible neurons.
    pub public_neuron_subset_metrics: Option<NnsGovernanceNeuronSubsetMetrics>,
    /// Metrics for neurons with declining voting power.
    pub declining_voting_power_neuron_subset_metrics: Option<NnsGovernanceNeuronSubsetMetrics>,
    /// Metrics for neurons that have fully lost voting power.
    pub fully_lost_voting_power_neuron_subset_metrics: Option<NnsGovernanceNeuronSubsetMetrics>,
}

///
/// NnsGovernanceNeuronSubsetMetrics
///
/// Native cached Governance metrics for one neuron subset.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceNeuronSubsetMetrics {
    /// Number of neurons in the subset.
    pub count: Option<u64>,
    /// Total subset stake in e8s.
    pub total_staked_e8s: Option<u64>,
    /// Total subset maturity in e8s-equivalent.
    pub total_maturity_e8s_equivalent: Option<u64>,
    /// Total subset staked maturity in e8s-equivalent.
    pub total_staked_maturity_e8s_equivalent: Option<u64>,
    /// Deprecated raw total voting power.
    pub total_voting_power: Option<u64>,
    /// Total deciding voting power.
    pub total_deciding_voting_power: Option<u64>,
    /// Total potential voting power.
    pub total_potential_voting_power: Option<u64>,
    /// Neuron-count buckets.
    pub count_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Stake buckets in e8s.
    pub staked_e8s_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Maturity buckets in e8s-equivalent.
    pub maturity_e8s_equivalent_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Staked-maturity buckets in e8s-equivalent.
    pub staked_maturity_e8s_equivalent_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Deprecated voting-power buckets.
    pub voting_power_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Deciding-voting-power buckets.
    pub deciding_voting_power_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
    /// Potential-voting-power buckets.
    pub potential_voting_power_buckets: Vec<NnsGovernanceMetricBucket<u64>>,
}
