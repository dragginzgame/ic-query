//! Module: sns::report::model::reports::proposals::row
//!
//! Responsibility: define SNS proposal row and nested value DTOs.
//! Does not own: source conversion, report-level metadata, or rendering.
//! Boundary: preserves proposal detail fields for cache snapshots and JSON output.

use serde::{Deserialize as SerdeDeserialize, Deserializer, Serialize};
use std::borrow::Cow;

/// Label a native SNS action id without replacing its original numeric evidence.
pub(in crate::sns::report) fn sns_proposal_action_label(id: u64) -> Cow<'static, str> {
    match id {
        0 => Cow::Borrowed("unspecified"),
        1 => Cow::Borrowed("motion"),
        2 => Cow::Borrowed("manage_nervous_system_parameters"),
        3 => Cow::Borrowed("upgrade_sns_controlled_canister"),
        4 => Cow::Borrowed("add_generic_nervous_system_function"),
        5 => Cow::Borrowed("remove_generic_nervous_system_function"),
        6 => Cow::Borrowed("execute_generic_nervous_system_function"),
        7 => Cow::Borrowed("upgrade_sns_to_next_version"),
        8 => Cow::Borrowed("manage_sns_metadata"),
        9 => Cow::Borrowed("transfer_sns_treasury_funds"),
        10 => Cow::Borrowed("register_dapp_canisters"),
        11 => Cow::Borrowed("deregister_dapp_canisters"),
        12 => Cow::Borrowed("mint_sns_tokens"),
        13 => Cow::Borrowed("manage_ledger_parameters"),
        14 => Cow::Borrowed("manage_dapp_canister_settings"),
        15 => Cow::Borrowed("advance_sns_target_version"),
        16 => Cow::Borrowed("set_topics_for_custom_proposals"),
        17 => Cow::Borrowed("register_extension"),
        18 => Cow::Borrowed("execute_extension_operation"),
        19 => Cow::Borrowed("upgrade_extension"),
        id if id >= 1_000 => Cow::Owned(format!("generic:{id}")),
        id => Cow::Owned(format!("unknown:{id}")),
    }
}

/// Label a native SNS ballot code, preserving unknown codes for display.
pub(in crate::sns::report) fn sns_proposal_vote_label(code: i32) -> Cow<'static, str> {
    match code {
        0 => Cow::Borrowed("unspecified"),
        1 => Cow::Borrowed("yes"),
        2 => Cow::Borrowed("no"),
        code => Cow::Owned(format!("unknown:{code}")),
    }
}

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
#[serde(deny_unknown_fields)]
pub struct SnsProposalRow {
    pub proposal_id: u64,
    pub action_id: u64,
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
    pub decided_timestamp_seconds: Option<u64>,
    pub executed_timestamp_seconds: Option<u64>,
    pub failed_timestamp_seconds: Option<u64>,
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
#[serde(deny_unknown_fields)]
pub struct SnsProposalBallotRow {
    pub neuron_id: String,
    pub vote: i32,
    pub cast_timestamp_seconds: u64,
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

    #[test]
    fn proposal_action_labels_cover_native_generic_and_unknown_ids() {
        for (id, label) in [
            (0, "unspecified"),
            (1, "motion"),
            (2, "manage_nervous_system_parameters"),
            (3, "upgrade_sns_controlled_canister"),
            (4, "add_generic_nervous_system_function"),
            (5, "remove_generic_nervous_system_function"),
            (6, "execute_generic_nervous_system_function"),
            (7, "upgrade_sns_to_next_version"),
            (8, "manage_sns_metadata"),
            (9, "transfer_sns_treasury_funds"),
            (10, "register_dapp_canisters"),
            (11, "deregister_dapp_canisters"),
            (12, "mint_sns_tokens"),
            (13, "manage_ledger_parameters"),
            (14, "manage_dapp_canister_settings"),
            (15, "advance_sns_target_version"),
            (16, "set_topics_for_custom_proposals"),
            (17, "register_extension"),
            (18, "execute_extension_operation"),
            (19, "upgrade_extension"),
            (20, "unknown:20"),
            (1_000, "generic:1000"),
            (u64::MAX, "generic:18446744073709551615"),
        ] {
            assert_eq!(sns_proposal_action_label(id), label);
        }
    }

    #[test]
    fn proposal_vote_labels_preserve_known_and_unknown_codes() {
        for (code, label) in [
            (0, "unspecified"),
            (1, "yes"),
            (2, "no"),
            (99, "unknown:99"),
            (-1, "unknown:-1"),
        ] {
            assert_eq!(sns_proposal_vote_label(code), label);
        }
    }
}
