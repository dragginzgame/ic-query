//! Module: sns::report::model::reports::proposals::row
//!
//! Responsibility: define SNS proposal row and nested value DTOs.
//! Does not own: source conversion, report-level metadata, or rendering.
//! Boundary: preserves proposal detail fields for cache snapshots and JSON output.

use serde::{Deserialize as SerdeDeserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::{borrow::Cow, fmt};

///
/// SnsProposalAction
///
/// Native SNS Governance proposal action, including generic and unknown action ids.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SnsProposalAction {
    /// No native action was specified.
    Unspecified,
    /// Adopt a governance motion.
    Motion,
    /// Change nervous-system parameters.
    ManageNervousSystemParameters,
    /// Upgrade an SNS-controlled canister.
    UpgradeSnsControlledCanister,
    /// Add a generic nervous-system function.
    AddGenericNervousSystemFunction,
    /// Remove a generic nervous-system function.
    RemoveGenericNervousSystemFunction,
    /// Execute a generic nervous-system function.
    ExecuteGenericNervousSystemFunction,
    /// Upgrade the SNS to its next blessed version.
    UpgradeSnsToNextVersion,
    /// Change SNS metadata.
    ManageSnsMetadata,
    /// Transfer SNS treasury funds.
    TransferSnsTreasuryFunds,
    /// Register dapp canisters with SNS Root.
    RegisterDappCanisters,
    /// Deregister dapp canisters from SNS Root.
    DeregisterDappCanisters,
    /// Mint SNS ledger tokens.
    MintSnsTokens,
    /// Change SNS ledger parameters.
    ManageLedgerParameters,
    /// Change dapp-canister settings.
    ManageDappCanisterSettings,
    /// Advance the SNS target version.
    AdvanceSnsTargetVersion,
    /// Assign topics to custom proposals.
    SetTopicsForCustomProposals,
    /// Register an SNS extension.
    RegisterExtension,
    /// Execute an SNS extension operation.
    ExecuteExtensionOperation,
    /// Upgrade an SNS extension.
    UpgradeExtension,
    /// A generic nervous-system function identified by its native action id.
    Generic(u64),
    /// An unrecognized native action id retained as evidence.
    Unknown(u64),
}

impl SnsProposalAction {
    /// Classify one raw native action id without discarding generic or unknown evidence.
    #[must_use]
    pub const fn from_id(id: u64) -> Self {
        match id {
            0 => Self::Unspecified,
            1 => Self::Motion,
            2 => Self::ManageNervousSystemParameters,
            3 => Self::UpgradeSnsControlledCanister,
            4 => Self::AddGenericNervousSystemFunction,
            5 => Self::RemoveGenericNervousSystemFunction,
            6 => Self::ExecuteGenericNervousSystemFunction,
            7 => Self::UpgradeSnsToNextVersion,
            8 => Self::ManageSnsMetadata,
            9 => Self::TransferSnsTreasuryFunds,
            10 => Self::RegisterDappCanisters,
            11 => Self::DeregisterDappCanisters,
            12 => Self::MintSnsTokens,
            13 => Self::ManageLedgerParameters,
            14 => Self::ManageDappCanisterSettings,
            15 => Self::AdvanceSnsTargetVersion,
            16 => Self::SetTopicsForCustomProposals,
            17 => Self::RegisterExtension,
            18 => Self::ExecuteExtensionOperation,
            19 => Self::UpgradeExtension,
            id if id >= 1_000 => Self::Generic(id),
            id => Self::Unknown(id),
        }
    }

    /// Return the raw native action id represented by this classification.
    #[must_use]
    pub const fn id(self) -> u64 {
        match self {
            Self::Unspecified => 0,
            Self::Motion => 1,
            Self::ManageNervousSystemParameters => 2,
            Self::UpgradeSnsControlledCanister => 3,
            Self::AddGenericNervousSystemFunction => 4,
            Self::RemoveGenericNervousSystemFunction => 5,
            Self::ExecuteGenericNervousSystemFunction => 6,
            Self::UpgradeSnsToNextVersion => 7,
            Self::ManageSnsMetadata => 8,
            Self::TransferSnsTreasuryFunds => 9,
            Self::RegisterDappCanisters => 10,
            Self::DeregisterDappCanisters => 11,
            Self::MintSnsTokens => 12,
            Self::ManageLedgerParameters => 13,
            Self::ManageDappCanisterSettings => 14,
            Self::AdvanceSnsTargetVersion => 15,
            Self::SetTopicsForCustomProposals => 16,
            Self::RegisterExtension => 17,
            Self::ExecuteExtensionOperation => 18,
            Self::UpgradeExtension => 19,
            Self::Generic(id) | Self::Unknown(id) => id,
        }
    }

    /// Return the stable cache, JSON, and text label.
    #[must_use]
    pub fn label(self) -> Cow<'static, str> {
        match self {
            Self::Unspecified => Cow::Borrowed("unspecified"),
            Self::Motion => Cow::Borrowed("motion"),
            Self::ManageNervousSystemParameters => {
                Cow::Borrowed("manage_nervous_system_parameters")
            }
            Self::UpgradeSnsControlledCanister => Cow::Borrowed("upgrade_sns_controlled_canister"),
            Self::AddGenericNervousSystemFunction => {
                Cow::Borrowed("add_generic_nervous_system_function")
            }
            Self::RemoveGenericNervousSystemFunction => {
                Cow::Borrowed("remove_generic_nervous_system_function")
            }
            Self::ExecuteGenericNervousSystemFunction => {
                Cow::Borrowed("execute_generic_nervous_system_function")
            }
            Self::UpgradeSnsToNextVersion => Cow::Borrowed("upgrade_sns_to_next_version"),
            Self::ManageSnsMetadata => Cow::Borrowed("manage_sns_metadata"),
            Self::TransferSnsTreasuryFunds => Cow::Borrowed("transfer_sns_treasury_funds"),
            Self::RegisterDappCanisters => Cow::Borrowed("register_dapp_canisters"),
            Self::DeregisterDappCanisters => Cow::Borrowed("deregister_dapp_canisters"),
            Self::MintSnsTokens => Cow::Borrowed("mint_sns_tokens"),
            Self::ManageLedgerParameters => Cow::Borrowed("manage_ledger_parameters"),
            Self::ManageDappCanisterSettings => Cow::Borrowed("manage_dapp_canister_settings"),
            Self::AdvanceSnsTargetVersion => Cow::Borrowed("advance_sns_target_version"),
            Self::SetTopicsForCustomProposals => Cow::Borrowed("set_topics_for_custom_proposals"),
            Self::RegisterExtension => Cow::Borrowed("register_extension"),
            Self::ExecuteExtensionOperation => Cow::Borrowed("execute_extension_operation"),
            Self::UpgradeExtension => Cow::Borrowed("upgrade_extension"),
            Self::Generic(id) => Cow::Owned(format!("generic:{id}")),
            Self::Unknown(id) => Cow::Owned(format!("unknown:{id}")),
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        let action = match label {
            "unspecified" => Self::Unspecified,
            "motion" => Self::Motion,
            "manage_nervous_system_parameters" => Self::ManageNervousSystemParameters,
            "upgrade_sns_controlled_canister" => Self::UpgradeSnsControlledCanister,
            "add_generic_nervous_system_function" => Self::AddGenericNervousSystemFunction,
            "remove_generic_nervous_system_function" => Self::RemoveGenericNervousSystemFunction,
            "execute_generic_nervous_system_function" => Self::ExecuteGenericNervousSystemFunction,
            "upgrade_sns_to_next_version" => Self::UpgradeSnsToNextVersion,
            "manage_sns_metadata" => Self::ManageSnsMetadata,
            "transfer_sns_treasury_funds" => Self::TransferSnsTreasuryFunds,
            "register_dapp_canisters" => Self::RegisterDappCanisters,
            "deregister_dapp_canisters" => Self::DeregisterDappCanisters,
            "mint_sns_tokens" => Self::MintSnsTokens,
            "manage_ledger_parameters" => Self::ManageLedgerParameters,
            "manage_dapp_canister_settings" => Self::ManageDappCanisterSettings,
            "advance_sns_target_version" => Self::AdvanceSnsTargetVersion,
            "set_topics_for_custom_proposals" => Self::SetTopicsForCustomProposals,
            "register_extension" => Self::RegisterExtension,
            "execute_extension_operation" => Self::ExecuteExtensionOperation,
            "upgrade_extension" => Self::UpgradeExtension,
            _ => {
                let id = label
                    .strip_prefix("generic:")
                    .or_else(|| label.strip_prefix("unknown:"))?
                    .parse::<u64>()
                    .ok()?;
                Self::from_id(id)
            }
        };
        (action.label().as_ref() == label).then_some(action)
    }
}

impl fmt::Display for SnsProposalAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label().as_ref())
    }
}

impl Serialize for SnsProposalAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.label().as_ref())
    }
}

impl<'de> SerdeDeserialize<'de> for SnsProposalAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Self::from_label(&label)
            .ok_or_else(|| D::Error::custom(format!("invalid SNS proposal action {label:?}")))
    }
}

///
/// SnsProposalVote
///
/// Native SNS Governance ballot vote, retaining unknown raw codes.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SnsProposalVote {
    /// No native vote was specified.
    Unspecified,
    /// Affirmative ballot.
    Yes,
    /// Negative ballot.
    No,
    /// An unrecognized native vote code retained as evidence.
    Unknown(i32),
}

impl SnsProposalVote {
    /// Classify one raw native vote code without discarding unknown evidence.
    #[must_use]
    pub const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Unspecified,
            1 => Self::Yes,
            2 => Self::No,
            code => Self::Unknown(code),
        }
    }

    /// Return the raw native vote code represented by this classification.
    #[must_use]
    pub const fn code(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::Yes => 1,
            Self::No => 2,
            Self::Unknown(code) => code,
        }
    }

    /// Return the stable cache, JSON, and text label.
    #[must_use]
    pub fn label(self) -> Cow<'static, str> {
        match self {
            Self::Unspecified => Cow::Borrowed("unspecified"),
            Self::Yes => Cow::Borrowed("yes"),
            Self::No => Cow::Borrowed("no"),
            Self::Unknown(code) => Cow::Owned(format!("unknown:{code}")),
        }
    }

    fn from_label(label: &str) -> Option<Self> {
        let vote = match label {
            "unspecified" => Self::Unspecified,
            "yes" => Self::Yes,
            "no" => Self::No,
            _ => Self::from_code(label.strip_prefix("unknown:")?.parse::<i32>().ok()?),
        };
        (vote.label().as_ref() == label).then_some(vote)
    }
}

impl fmt::Display for SnsProposalVote {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label().as_ref())
    }
}

impl Serialize for SnsProposalVote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.label().as_ref())
    }
}

impl<'de> SerdeDeserialize<'de> for SnsProposalVote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Self::from_label(&label)
            .ok_or_else(|| D::Error::custom(format!("invalid SNS proposal vote {label:?}")))
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
pub struct SnsProposalRow {
    pub proposal_id: u64,
    pub action_id: u64,
    pub action: SnsProposalAction,
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
    pub vote_text: SnsProposalVote,
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

    const NATIVE_ACTION_CASES: [(u64, SnsProposalAction, &str); 20] = [
        (0, SnsProposalAction::Unspecified, "unspecified"),
        (1, SnsProposalAction::Motion, "motion"),
        (
            2,
            SnsProposalAction::ManageNervousSystemParameters,
            "manage_nervous_system_parameters",
        ),
        (
            3,
            SnsProposalAction::UpgradeSnsControlledCanister,
            "upgrade_sns_controlled_canister",
        ),
        (
            4,
            SnsProposalAction::AddGenericNervousSystemFunction,
            "add_generic_nervous_system_function",
        ),
        (
            5,
            SnsProposalAction::RemoveGenericNervousSystemFunction,
            "remove_generic_nervous_system_function",
        ),
        (
            6,
            SnsProposalAction::ExecuteGenericNervousSystemFunction,
            "execute_generic_nervous_system_function",
        ),
        (
            7,
            SnsProposalAction::UpgradeSnsToNextVersion,
            "upgrade_sns_to_next_version",
        ),
        (
            8,
            SnsProposalAction::ManageSnsMetadata,
            "manage_sns_metadata",
        ),
        (
            9,
            SnsProposalAction::TransferSnsTreasuryFunds,
            "transfer_sns_treasury_funds",
        ),
        (
            10,
            SnsProposalAction::RegisterDappCanisters,
            "register_dapp_canisters",
        ),
        (
            11,
            SnsProposalAction::DeregisterDappCanisters,
            "deregister_dapp_canisters",
        ),
        (12, SnsProposalAction::MintSnsTokens, "mint_sns_tokens"),
        (
            13,
            SnsProposalAction::ManageLedgerParameters,
            "manage_ledger_parameters",
        ),
        (
            14,
            SnsProposalAction::ManageDappCanisterSettings,
            "manage_dapp_canister_settings",
        ),
        (
            15,
            SnsProposalAction::AdvanceSnsTargetVersion,
            "advance_sns_target_version",
        ),
        (
            16,
            SnsProposalAction::SetTopicsForCustomProposals,
            "set_topics_for_custom_proposals",
        ),
        (
            17,
            SnsProposalAction::RegisterExtension,
            "register_extension",
        ),
        (
            18,
            SnsProposalAction::ExecuteExtensionOperation,
            "execute_extension_operation",
        ),
        (19, SnsProposalAction::UpgradeExtension, "upgrade_extension"),
    ];

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
    fn proposal_action_labels_round_trip_native_generic_and_unknown_ids() {
        for (id, action, label) in NATIVE_ACTION_CASES.into_iter().chain([
            (20, SnsProposalAction::Unknown(20), "unknown:20"),
            (1_000, SnsProposalAction::Generic(1_000), "generic:1000"),
        ]) {
            assert_eq!(SnsProposalAction::from_id(id), action);
            assert_eq!(action.id(), id);
            assert_eq!(action.label(), label);
            assert_eq!(
                serde_json::to_string(&action).unwrap(),
                format!("\"{label}\"")
            );
            assert_eq!(
                serde_json::from_str::<SnsProposalAction>(&format!("\"{label}\"")).unwrap(),
                action
            );
        }
        for invalid in ["unknown:1", "generic:20", "unknown:020", "future"] {
            assert!(serde_json::from_str::<SnsProposalAction>(&format!("\"{invalid}\"")).is_err());
        }
    }

    #[test]
    fn proposal_vote_labels_round_trip_known_and_unknown_codes() {
        for (code, vote, label) in [
            (0, SnsProposalVote::Unspecified, "unspecified"),
            (1, SnsProposalVote::Yes, "yes"),
            (2, SnsProposalVote::No, "no"),
            (99, SnsProposalVote::Unknown(99), "unknown:99"),
            (-1, SnsProposalVote::Unknown(-1), "unknown:-1"),
        ] {
            assert_eq!(SnsProposalVote::from_code(code), vote);
            assert_eq!(vote.code(), code);
            assert_eq!(vote.label(), label);
            assert_eq!(
                serde_json::to_string(&vote).unwrap(),
                format!("\"{label}\"")
            );
            assert_eq!(
                serde_json::from_str::<SnsProposalVote>(&format!("\"{label}\"")).unwrap(),
                vote
            );
        }
        for invalid in ["unknown:1", "unknown:+3", "unknown:03", "maybe"] {
            assert!(serde_json::from_str::<SnsProposalVote>(&format!("\"{invalid}\"")).is_err());
        }
    }
}
