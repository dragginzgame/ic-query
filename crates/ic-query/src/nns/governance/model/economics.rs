//! Module: nns::governance::model::economics
//!
//! Responsibility: native NNS Governance economics report contracts.
//! Does not own: metrics, reward events, maturity modulation, transport, or rendering.
//! Boundary: preserves the complete network-economics response and nested native wrappers.

use super::NnsGovernanceReportContext;
#[cfg(feature = "host")]
use candid::CandidType;
use serde::{Deserialize as SerdeDeserialize, Serialize};

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
