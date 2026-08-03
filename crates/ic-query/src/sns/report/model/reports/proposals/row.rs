//! Module: sns::report::model::reports::proposals::row
//!
//! Responsibility: define SNS proposal row and nested value DTOs.
//! Does not own: source conversion, report-level metadata, or rendering.
//! Boundary: preserves proposal detail fields for cache snapshots and JSON output.

use serde::{Deserialize as SerdeDeserialize, Deserializer, Serialize};

///
/// SnsProposalDecisionState
///
/// Derived lifecycle state for one SNS Governance proposal.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsProposalDecisionState {
    /// The proposal has not reached a decision.
    Open,
    /// The proposal was decided without an execution or failure timestamp.
    Decided,
    /// The proposal has an execution timestamp.
    Executed,
    /// The proposal has a failure timestamp.
    Failed,
}

impl SnsProposalDecisionState {
    /// Return the stable cache, JSON, and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Decided => "decided",
            Self::Executed => "executed",
            Self::Failed => "failed",
        }
    }
}

///
/// SnsProposalRow
///
/// Serializable row for one SNS governance proposal.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalRow {
    pub proposal_id: u64,
    pub action_id: u64,
    pub action: String,
    pub title: String,
    pub summary: String,
    pub url: Option<String>,
    pub decision_state: SnsProposalDecisionState,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub status: Option<i32>,
    #[serde(deserialize_with = "deserialize_required_option")]
    pub topic: Option<String>,
    pub reject_cost_e8s: u64,
    pub proposal_creation_timestamp_seconds: u64,
    pub created_at: String,
    pub decided_timestamp_seconds: Option<u64>,
    pub decided_at: Option<String>,
    pub executed_timestamp_seconds: Option<u64>,
    pub executed_at: Option<String>,
    pub failed_timestamp_seconds: Option<u64>,
    pub failed_at: Option<String>,
    pub failure_reason: Option<SnsProposalFailureReason>,
    pub reward_event_round: u64,
    pub reward_event_end_timestamp_seconds: Option<u64>,
    pub is_eligible_for_rewards: bool,
    pub latest_tally: Option<SnsProposalTally>,
    pub ballot_count: usize,
    pub ballots: Vec<SnsProposalBallotRow>,
    pub payload_text_rendering: Option<String>,
    pub proposer_neuron_id: Option<String>,
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: SerdeDeserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

///
/// SnsProposalBallotRow
///
/// Serializable row for one proposal ballot.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalBallotRow {
    pub neuron_id: String,
    pub vote: i32,
    pub vote_text: String,
    pub cast_timestamp_seconds: u64,
    pub cast_at: Option<String>,
    pub voting_power: u64,
}

///
/// SnsProposalFailureReason
///
/// Serializable SNS governance failure reason attached to a proposal.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalFailureReason {
    pub error_type: i32,
    pub error_message: String,
}

///
/// SnsProposalTally
///
/// Serializable SNS proposal vote tally.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalTally {
    pub timestamp_seconds: u64,
    pub yes: u64,
    pub no: u64,
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_decision_state_labels_round_trip() {
        for (state, label) in [
            (SnsProposalDecisionState::Open, "open"),
            (SnsProposalDecisionState::Decided, "decided"),
            (SnsProposalDecisionState::Executed, "executed"),
            (SnsProposalDecisionState::Failed, "failed"),
        ] {
            assert_eq!(
                serde_json::to_string(&state).unwrap(),
                format!("\"{label}\"")
            );
            assert_eq!(
                serde_json::from_str::<SnsProposalDecisionState>(&format!("\"{label}\"")).unwrap(),
                state
            );
        }
        assert!(serde_json::from_str::<SnsProposalDecisionState>("\"unknown\"").is_err());
    }
}
