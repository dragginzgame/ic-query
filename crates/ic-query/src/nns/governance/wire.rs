//! Module: nns::governance::wire
//!
//! Responsibility: share external NNS Governance identifiers, errors and metrics projections.
//! Does not own: transport, report provenance, or public rendering.
//! Boundary: preserves official Candid records and converts unnamed metric buckets to report rows.

use super::{NnsGovernanceMetricBucket, NnsGovernanceMetrics, NnsGovernanceNeuronSubsetMetrics};
use ic_platform::candid::contract::{CandidType, serde::Deserialize};

///
/// GovernanceCachedMetrics
///
/// Native Candid cached Governance metrics response.
///

#[cfg_attr(test, derive(Default))]
#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
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
#[candid_path("ic_platform::candid::contract")]
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
/// NnsGovernanceNeuronId
///
/// External NNS Governance neuron identifier shared by neuron and proposal queries.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns) struct NnsGovernanceNeuronId {
    pub(in crate::nns) id: u64,
}

///
/// GovernanceError
///
/// Native Candid Governance application error.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns) struct GovernanceError {
    pub(in crate::nns) error_message: String,
    pub(in crate::nns) error_type: i32,
}

///
/// GetMetricsResult
///
/// Native Candid result returned by the Governance metrics query.
///

pub(super) type GetMetricsResult = Result<Box<GovernanceCachedMetrics>, GovernanceError>;

///
/// GetMaturityModulationRequest
///
/// Native empty Candid request for current maturity modulation.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetMaturityModulationRequest {}

///
/// GetMaturityModulationResponse
///
/// Native Candid response containing optional current maturity modulation.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetMaturityModulationResponse {
    pub(super) maturity_modulation: Option<MaturityModulation>,
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

///
/// NetworkEconomics
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct NetworkEconomics {
    pub(super) neuron_minimum_stake_e8s: u64,
    pub(super) max_proposals_to_keep_per_topic: u32,
    pub(super) neuron_management_fee_per_proposal_e8s: u64,
    pub(super) reject_cost_e8s: u64,
    pub(super) transaction_fee_e8s: u64,
    pub(super) neuron_spawn_dissolve_delay_seconds: u64,
    pub(super) minimum_icp_xdr_rate: u64,
    pub(super) maximum_node_provider_rewards_e8s: u64,
    pub(super) neurons_fund_economics: Option<NeuronsFundEconomics>,
    pub(super) voting_power_economics: Option<VotingPowerEconomics>,
}

impl From<NetworkEconomics> for super::NnsGovernanceEconomics {
    fn from(value: NetworkEconomics) -> Self {
        Self {
            neuron_minimum_stake_e8s: value.neuron_minimum_stake_e8s,
            max_proposals_to_keep_per_topic: value.max_proposals_to_keep_per_topic,
            neuron_management_fee_per_proposal_e8s: value.neuron_management_fee_per_proposal_e8s,
            reject_cost_e8s: value.reject_cost_e8s,
            transaction_fee_e8s: value.transaction_fee_e8s,
            neuron_spawn_dissolve_delay_seconds: value.neuron_spawn_dissolve_delay_seconds,
            minimum_icp_xdr_rate: value.minimum_icp_xdr_rate,
            maximum_node_provider_rewards_e8s: value.maximum_node_provider_rewards_e8s,
            neurons_fund_economics: value.neurons_fund_economics.map(Into::into),
            voting_power_economics: value.voting_power_economics.map(Into::into),
        }
    }
}

///
/// NeuronsFundEconomics
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct NeuronsFundEconomics {
    pub(super) maximum_icp_xdr_rate: Option<Percentage>,
    pub(super) neurons_fund_matched_funding_curve_coefficients:
        Option<MatchedFundingCurveCoefficients>,
    pub(super) max_theoretical_neurons_fund_participation_amount_xdr: Option<Decimal>,
    pub(super) minimum_icp_xdr_rate: Option<Percentage>,
}

impl From<NeuronsFundEconomics> for super::NnsNeuronsFundEconomics {
    fn from(value: NeuronsFundEconomics) -> Self {
        Self {
            maximum_icp_xdr_rate: value.maximum_icp_xdr_rate.map(Into::into),
            neurons_fund_matched_funding_curve_coefficients: value
                .neurons_fund_matched_funding_curve_coefficients
                .map(Into::into),
            max_theoretical_neurons_fund_participation_amount_xdr: value
                .max_theoretical_neurons_fund_participation_amount_xdr
                .map(Into::into),
            minimum_icp_xdr_rate: value.minimum_icp_xdr_rate.map(Into::into),
        }
    }
}

///
/// MatchedFundingCurveCoefficients
///
/// External NNS Governance Candid record projected into Query report data.
///

#[expect(
    clippy::struct_field_names,
    reason = "Field names follow the external NNS Governance Candid contract."
)]
#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct MatchedFundingCurveCoefficients {
    pub(super) contribution_threshold_xdr: Option<Decimal>,
    pub(super) one_third_participation_milestone_xdr: Option<Decimal>,
    pub(super) full_participation_milestone_xdr: Option<Decimal>,
}

impl From<MatchedFundingCurveCoefficients>
    for super::NnsNeuronsFundMatchedFundingCurveCoefficients
{
    fn from(value: MatchedFundingCurveCoefficients) -> Self {
        Self {
            contribution_threshold_xdr: value.contribution_threshold_xdr.map(Into::into),
            one_third_participation_milestone_xdr: value
                .one_third_participation_milestone_xdr
                .map(Into::into),
            full_participation_milestone_xdr: value
                .full_participation_milestone_xdr
                .map(Into::into),
        }
    }
}

///
/// Percentage
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct Percentage {
    pub(super) basis_points: Option<u64>,
}

impl From<Percentage> for super::NnsGovernancePercentage {
    fn from(value: Percentage) -> Self {
        Self {
            basis_points: value.basis_points,
        }
    }
}

///
/// Decimal
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct Decimal {
    pub(super) human_readable: Option<String>,
}

impl From<Decimal> for super::NnsGovernanceDecimal {
    fn from(value: Decimal) -> Self {
        Self {
            human_readable: value.human_readable,
        }
    }
}

///
/// VotingPowerEconomics
///
/// External NNS Governance Candid record projected into Query report data.
///

#[expect(
    clippy::struct_field_names,
    reason = "Field names follow the external NNS Governance Candid contract."
)]
#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct VotingPowerEconomics {
    pub(super) start_reducing_voting_power_after_seconds: Option<u64>,
    pub(super) clear_following_after_seconds: Option<u64>,
    pub(super) neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
}

impl From<VotingPowerEconomics> for super::NnsVotingPowerEconomics {
    fn from(value: VotingPowerEconomics) -> Self {
        Self {
            start_reducing_voting_power_after_seconds: value
                .start_reducing_voting_power_after_seconds,
            clear_following_after_seconds: value.clear_following_after_seconds,
            neuron_minimum_dissolve_delay_to_vote_seconds: value
                .neuron_minimum_dissolve_delay_to_vote_seconds,
        }
    }
}

///
/// RewardEvent
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct RewardEvent {
    pub(super) rounds_since_last_distribution: Option<u64>,
    pub(super) day_after_genesis: u64,
    pub(super) actual_timestamp_seconds: u64,
    pub(super) total_available_e8s_equivalent: u64,
    pub(super) latest_round_available_e8s_equivalent: Option<u64>,
    pub(super) distributed_e8s_equivalent: u64,
    pub(super) settled_proposals: Vec<ProposalId>,
}

impl From<RewardEvent> for super::NnsGovernanceRewardEvent {
    fn from(value: RewardEvent) -> Self {
        Self {
            rounds_since_last_distribution: value.rounds_since_last_distribution,
            day_after_genesis: value.day_after_genesis,
            actual_timestamp_seconds: value.actual_timestamp_seconds,
            total_available_e8s_equivalent: value.total_available_e8s_equivalent,
            latest_round_available_e8s_equivalent: value.latest_round_available_e8s_equivalent,
            distributed_e8s_equivalent: value.distributed_e8s_equivalent,
            settled_proposals: value
                .settled_proposals
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

///
/// ProposalId
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[candid_path("ic_platform::candid::contract")]
pub(in crate::nns) struct ProposalId {
    pub(in crate::nns) id: u64,
}

impl From<ProposalId> for super::NnsGovernanceProposalId {
    fn from(value: ProposalId) -> Self {
        Self { id: value.id }
    }
}

///
/// MaturityModulation
///
/// External NNS Governance Candid record projected into Query report data.
///

#[derive(CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct MaturityModulation {
    pub(super) current_value_permyriad: Option<i32>,
    pub(super) updated_at_timestamp_seconds: Option<u64>,
}

impl From<MaturityModulation> for super::NnsGovernanceMaturityModulation {
    fn from(value: MaturityModulation) -> Self {
        Self {
            current_value_permyriad: value.current_value_permyriad,
            updated_at_timestamp_seconds: value.updated_at_timestamp_seconds,
        }
    }
}
