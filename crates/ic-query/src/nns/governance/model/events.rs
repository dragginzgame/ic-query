//! Module: nns::governance::model::events
//!
//! Responsibility: native NNS Governance reward-event and maturity-modulation contracts.
//! Does not own: economics, metrics, transport, caching, or rendering.
//! Boundary: preserves the two bounded Governance point-value response families.

use super::NnsGovernanceReportContext;
#[cfg(any(feature = "nns-host", feature = "canister"))]
use candid::CandidType;
use serde::{Deserialize as SerdeDeserialize, Serialize};

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

#[cfg_attr(any(feature = "nns-host", feature = "canister"), derive(CandidType))]
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

#[cfg_attr(any(feature = "nns-host", feature = "canister"), derive(CandidType))]
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

#[cfg_attr(any(feature = "nns-host", feature = "canister"), derive(CandidType))]
#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct NnsGovernanceMaturityModulation {
    /// Current signed modulation in permyriad when supplied.
    pub current_value_permyriad: Option<i32>,
    /// Last update timestamp in Unix seconds when supplied.
    pub updated_at_timestamp_seconds: Option<u64>,
}
