//! Module: nns::governance::model
//!
//! Responsibility: define direct NNS Governance report models.
//! Does not own: live transport, CLI parsing, caching, or text rendering.
//! Boundary: preserves native Governance values and explicit query provenance.

#[cfg(feature = "host")]
use candid::CandidType;
use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// NnsGovernanceReportContext
///
/// Shared provenance flattened into every direct NNS Governance report.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceReportContext {
    /// Report schema version.
    pub schema_version: u32,
    /// Queried network identity.
    pub network: String,
    /// NNS Governance canister principal.
    pub governance_canister_id: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Replica endpoint used for the query.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
}

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
/// NnsGovernanceEconomicsReport
///
/// Serializable live snapshot of the NNS Governance economics parameters.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceEconomicsReport {
    /// Shared Governance query provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Native Governance economics parameters.
    pub economics: NnsGovernanceEconomics,
}

///
/// NnsGovernanceEconomics
///
/// Native NNS Governance network economics parameters.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceEconomics {
    /// Minimum neuron stake in e8s.
    pub neuron_minimum_stake_e8s: u64,
    /// Maximum retained proposals per Governance topic.
    pub max_proposals_to_keep_per_topic: u32,
    /// Neuron-management proposal fee in e8s.
    pub neuron_management_fee_per_proposal_e8s: u64,
    /// Proposal rejection cost in e8s.
    pub reject_cost_e8s: u64,
    /// Ledger transaction fee in e8s.
    pub transaction_fee_e8s: u64,
    /// Spawned-neuron dissolve delay in seconds.
    pub neuron_spawn_dissolve_delay_seconds: u64,
    /// Raw minimum ICP/XDR rate.
    pub minimum_icp_xdr_rate: u64,
    /// Maximum node-provider rewards in e8s.
    pub maximum_node_provider_rewards_e8s: u64,
    /// Optional Neurons' Fund economics.
    pub neurons_fund_economics: Option<NnsNeuronsFundEconomics>,
    /// Optional voting-power economics.
    pub voting_power_economics: Option<NnsVotingPowerEconomics>,
}

///
/// NnsNeuronsFundEconomics
///
/// Native optional parameters governing Neurons' Fund participation.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsNeuronsFundEconomics {
    /// Maximum ICP/XDR rate.
    pub maximum_icp_xdr_rate: Option<NnsGovernancePercentage>,
    /// Matched-funding curve coefficients.
    pub neurons_fund_matched_funding_curve_coefficients:
        Option<NnsNeuronsFundMatchedFundingCurveCoefficients>,
    /// Maximum theoretical Neurons' Fund participation amount in XDR.
    pub max_theoretical_neurons_fund_participation_amount_xdr: Option<NnsGovernanceDecimal>,
    /// Minimum ICP/XDR rate.
    pub minimum_icp_xdr_rate: Option<NnsGovernancePercentage>,
}

///
/// NnsNeuronsFundMatchedFundingCurveCoefficients
///
/// Native decimal-string parameters for the Neurons' Fund matching curve.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsNeuronsFundMatchedFundingCurveCoefficients {
    /// Contribution threshold in XDR.
    pub contribution_threshold_xdr: Option<NnsGovernanceDecimal>,
    /// One-third participation milestone in XDR.
    pub one_third_participation_milestone_xdr: Option<NnsGovernanceDecimal>,
    /// Full participation milestone in XDR.
    pub full_participation_milestone_xdr: Option<NnsGovernanceDecimal>,
}

///
/// NnsGovernancePercentage
///
/// Native optional-basis-points percentage wrapper used by Governance.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernancePercentage {
    /// Percentage value in basis points when supplied.
    pub basis_points: Option<u64>,
}

///
/// NnsGovernanceDecimal
///
/// Native human-readable decimal wrapper used by Governance.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceDecimal {
    /// Decimal representation when supplied.
    pub human_readable: Option<String>,
}

///
/// NnsVotingPowerEconomics
///
/// Native optional parameters governing NNS neuron voting power.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsVotingPowerEconomics {
    /// Inactivity duration before voting power starts decreasing.
    pub start_reducing_voting_power_after_seconds: Option<u64>,
    /// Reduction duration after which following is cleared.
    pub clear_following_after_seconds: Option<u64>,
    /// Minimum dissolve delay required to vote.
    pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
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

///
/// NnsGovernanceRewardEventReport
///
/// Serializable live snapshot of the latest NNS voting reward event.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceRewardEventReport {
    /// Shared Governance query provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Latest native Governance reward event.
    pub reward_event: NnsGovernanceRewardEvent,
}

///
/// NnsGovernanceRewardEvent
///
/// Latest native NNS Governance voting reward event.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceRewardEvent {
    /// Rounds elapsed since the previous distribution when supplied.
    pub rounds_since_last_distribution: Option<u64>,
    /// Reward day after NNS genesis.
    pub day_after_genesis: u64,
    /// Actual reward-event timestamp in Unix seconds.
    pub actual_timestamp_seconds: u64,
    /// Total rewards available in e8s-equivalent.
    pub total_available_e8s_equivalent: u64,
    /// Rewards available in the latest round when supplied.
    pub latest_round_available_e8s_equivalent: Option<u64>,
    /// Rewards distributed in e8s-equivalent.
    pub distributed_e8s_equivalent: u64,
    /// Proposals settled by the event, in Governance order.
    pub settled_proposals: Vec<NnsGovernanceProposalId>,
}

///
/// NnsGovernanceProposalId
///
/// Native NNS Governance proposal identifier wrapper.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceProposalId {
    /// Governance proposal identifier.
    pub id: u64,
}

///
/// NnsGovernanceMaturityModulationReport
///
/// Serializable live snapshot of NNS maturity modulation.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceMaturityModulationReport {
    /// Shared Governance query provenance.
    #[serde(flatten)]
    pub context: NnsGovernanceReportContext,
    /// Current modulation when Governance supplies it.
    pub maturity_modulation: Option<NnsGovernanceMaturityModulation>,
}

///
/// NnsGovernanceMaturityModulation
///
/// Current native NNS Governance maturity-modulation value.
///

#[cfg_attr(feature = "host", derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceMaturityModulation {
    /// Current signed modulation in permyriad when supplied.
    pub current_value_permyriad: Option<i32>,
    /// Last update timestamp in Unix seconds when supplied.
    pub updated_at_timestamp_seconds: Option<u64>,
}
