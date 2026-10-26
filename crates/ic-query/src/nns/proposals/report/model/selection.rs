//! Module: nns::proposals::report::model::selection
//!
//! Responsibility: NNS proposal filter, sort, and protocol vocabulary.
//! Does not own: request DTOs, serialized report DTOs, or report rendering.
//! Boundary: keeps public selection types and their canonical labels/codes together.

use serde::{Deserialize, Serialize};

macro_rules! governance_code_enum {
    (
        $(#[$enum_attr:meta])*
        pub enum $name:ident {
            $($(#[$variant_attr:meta])* $variant:ident = $code:literal => $label:literal,)*
        }
    ) => {
        $(#[$enum_attr])*
        pub enum $name {
            $($(#[$variant_attr])* #[serde(rename = $label)] $variant,)*
        }

        impl $name {
            /// Classify a native code, retaining the unspecified classification for unknown codes.
            #[must_use]
            pub const fn from_code(code: i32) -> Self {
                match code {
                    $($code => Self::$variant,)*
                    _ => Self::Unspecified,
                }
            }

            /// Return the canonical native code for this classification.
            #[must_use]
            pub const fn code(self) -> i32 {
                match self { $(Self::$variant => $code,)* }
            }

            /// Return the stable JSON and text label.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $label,)* }
            }
        }
    };
}

pub(in crate::nns) const NNS_PROPOSAL_SORT_API_LABEL: &str = "api";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ID_LABEL: &str = "id";
pub(in crate::nns) const NNS_PROPOSAL_SORT_STATUS_LABEL: &str = "status";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL: &str = "reward-status";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TOPIC_LABEL: &str = "topic";
pub(in crate::nns) const NNS_PROPOSAL_SORT_PROPOSER_LABEL: &str = "proposer";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TITLE_LABEL: &str = "title";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ACTION_LABEL: &str = "action";
pub(in crate::nns) const NNS_PROPOSAL_SORT_YES_LABEL: &str = "yes";
pub(in crate::nns) const NNS_PROPOSAL_SORT_NO_LABEL: &str = "no";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TOTAL_VOTES_LABEL: &str = "total-votes";
pub(in crate::nns) const NNS_PROPOSAL_SORT_TALLY_TIME_LABEL: &str = "tally-time";
pub(in crate::nns) const NNS_PROPOSAL_SORT_VOTING_POWER_LABEL: &str = "voting-power";
pub(in crate::nns) const NNS_PROPOSAL_SORT_BALLOTS_LABEL: &str = "ballots";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REJECT_COST_LABEL: &str = "reject-cost";
pub(in crate::nns) const NNS_PROPOSAL_SORT_REWARD_ROUND_LABEL: &str = "reward-round";
pub(in crate::nns) const NNS_PROPOSAL_SORT_PROPOSED_LABEL: &str = "proposed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DEADLINE_LABEL: &str = "deadline";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DECIDED_LABEL: &str = "decided";
pub(in crate::nns) const NNS_PROPOSAL_SORT_EXECUTED_LABEL: &str = "executed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_FAILED_LABEL: &str = "failed";
pub(in crate::nns) const NNS_PROPOSAL_SORT_ASC_LABEL: &str = "asc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_DESC_LABEL: &str = "desc";
pub(in crate::nns) const NNS_PROPOSAL_SORT_NONE_LABEL: &str = "none";

governance_code_enum! {
    ///
    /// NnsProposalStatus
    ///
    /// Native NNS Governance proposal decision status.
    ///

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
    pub enum NnsProposalStatus {
        /// Unspecified or unrecognized native status code.
        Unspecified = 0 => "unspecified",
        /// Proposal remains open for voting.
        Open = 1 => "open",
        /// Proposal was rejected.
        Rejected = 2 => "rejected",
        /// Proposal was adopted but has not completed execution.
        Adopted = 3 => "adopted",
        /// Proposal executed successfully.
        Executed = 4 => "executed",
        /// Proposal execution failed.
        Failed = 5 => "failed",
    }
}

governance_code_enum! {
    ///
    /// NnsProposalRewardStatus
    ///
    /// Native NNS Governance proposal reward-settlement status.
    ///

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
    pub enum NnsProposalRewardStatus {
        /// Unspecified or unrecognized native reward-status code.
        Unspecified = 0 => "unspecified",
        /// Proposal still accepts votes for reward purposes.
        AcceptVotes = 1 => "accept-votes",
        /// Proposal is ready for reward settlement.
        ReadyToSettle = 2 => "ready-to-settle",
        /// Proposal rewards have settled.
        Settled = 3 => "settled",
        /// Proposal is not eligible for voting rewards.
        Ineligible = 4 => "ineligible",
    }
}

governance_code_enum! {
    ///
    /// NnsProposalTopic
    ///
    /// Native NNS Governance proposal topic.
    ///

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
    pub enum NnsProposalTopic {
        /// Unspecified or unrecognized native topic code.
        Unspecified = 0 => "unspecified",
        /// Neuron management.
        NeuronManagement = 1 => "neuron-management",
        /// Exchange-rate management.
        ExchangeRate = 2 => "exchange-rate",
        /// Network economics.
        NetworkEconomics = 3 => "network-economics",
        /// Governance policy.
        Governance = 4 => "governance",
        /// Node administration.
        NodeAdmin = 5 => "node-admin",
        /// Participant management.
        ParticipantManagement = 6 => "participant-management",
        /// Subnet management.
        SubnetManagement = 7 => "subnet-management",
        /// Network-canister management.
        NetworkCanisterManagement = 8 => "network-canister-management",
        /// Know-your-customer policy.
        Kyc = 9 => "kyc",
        /// Node-provider rewards.
        NodeProviderRewards = 10 => "node-provider-rewards",
        /// IC OS version deployment.
        IcOsVersionDeployment = 12 => "ic-os-version-deployment",
        /// IC OS version election.
        IcOsVersionElection = 13 => "ic-os-version-election",
        /// SNS and Community Fund policy.
        SnsAndCommunityFund = 14 => "sns-and-community-fund",
        /// API boundary-node management.
        ApiBoundaryNodeManagement = 15 => "api-boundary-node-management",
        /// Subnet rental.
        SubnetRental = 16 => "subnet-rental",
        /// Application-canister management.
        ApplicationCanisterManagement = 17 => "application-canister-management",
        /// Protocol-canister management.
        ProtocolCanisterManagement = 18 => "protocol-canister-management",
    }
}

///
/// NnsProposalListSort
///
/// Report-model sort selector for NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalListSort {
    /// Preserve the descending proposal-id order returned by Governance.
    #[default]
    Api,
    /// Sort by proposal id.
    Id,
    /// Sort by the raw decision-status code.
    Status,
    /// Sort by the raw reward-settlement code.
    RewardStatus,
    /// Sort by the raw topic code.
    Topic,
    /// Sort by proposer neuron id.
    Proposer,
    /// Sort by proposal title.
    Title,
    /// Sort by the human-readable action classification.
    Action,
    /// Sort by affirmative voting power in the latest tally.
    Yes,
    /// Sort by negative voting power in the latest tally.
    No,
    /// Sort by total voting power in the latest tally.
    TotalVotes,
    /// Sort by the latest tally timestamp.
    TallyTime,
    /// Sort by total potential voting power.
    VotingPower,
    /// Sort by ballot count returned by Governance.
    Ballots,
    /// Sort by reject cost in ICP e8s.
    RejectCost,
    /// Sort by reward event round.
    RewardRound,
    /// Sort by proposal creation time.
    Proposed,
    /// Sort by voting deadline.
    Deadline,
    /// Sort by decision time.
    Decided,
    /// Sort by execution time.
    Executed,
    /// Sort by execution failure time.
    Failed,
}

impl NnsProposalListSort {
    /// Return the canonical filter or sort label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => NNS_PROPOSAL_SORT_API_LABEL,
            Self::Id => NNS_PROPOSAL_SORT_ID_LABEL,
            Self::Status => NNS_PROPOSAL_SORT_STATUS_LABEL,
            Self::RewardStatus => NNS_PROPOSAL_SORT_REWARD_STATUS_LABEL,
            Self::Topic => NNS_PROPOSAL_SORT_TOPIC_LABEL,
            Self::Proposer => NNS_PROPOSAL_SORT_PROPOSER_LABEL,
            Self::Title => NNS_PROPOSAL_SORT_TITLE_LABEL,
            Self::Action => NNS_PROPOSAL_SORT_ACTION_LABEL,
            Self::Yes => NNS_PROPOSAL_SORT_YES_LABEL,
            Self::No => NNS_PROPOSAL_SORT_NO_LABEL,
            Self::TotalVotes => NNS_PROPOSAL_SORT_TOTAL_VOTES_LABEL,
            Self::TallyTime => NNS_PROPOSAL_SORT_TALLY_TIME_LABEL,
            Self::VotingPower => NNS_PROPOSAL_SORT_VOTING_POWER_LABEL,
            Self::Ballots => NNS_PROPOSAL_SORT_BALLOTS_LABEL,
            Self::RejectCost => NNS_PROPOSAL_SORT_REJECT_COST_LABEL,
            Self::RewardRound => NNS_PROPOSAL_SORT_REWARD_ROUND_LABEL,
            Self::Proposed => NNS_PROPOSAL_SORT_PROPOSED_LABEL,
            Self::Deadline => NNS_PROPOSAL_SORT_DEADLINE_LABEL,
            Self::Decided => NNS_PROPOSAL_SORT_DECIDED_LABEL,
            Self::Executed => NNS_PROPOSAL_SORT_EXECUTED_LABEL,
            Self::Failed => NNS_PROPOSAL_SORT_FAILED_LABEL,
        }
    }

    /// Return the default direction for this sort key.
    #[must_use]
    pub const fn default_direction(self) -> NnsProposalSortDirection {
        match self {
            Self::Status
            | Self::RewardStatus
            | Self::Topic
            | Self::Proposer
            | Self::Title
            | Self::Action => NnsProposalSortDirection::Asc,
            _ => NnsProposalSortDirection::Desc,
        }
    }

    /// Return whether this key applies a local sort direction.
    #[must_use]
    pub const fn uses_local_direction(self) -> bool {
        !matches!(self, Self::Api)
    }

    /// Return `none` for API order or the selected local direction label.
    #[must_use]
    pub const fn direction_label(self, direction: NnsProposalSortDirection) -> &'static str {
        match self {
            Self::Api => NNS_PROPOSAL_SORT_NONE_LABEL,
            _ => direction.as_str(),
        }
    }
}

///
/// NnsProposalSortDirection
///
/// Report-model direction selector for local NNS proposal sorting.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalSortDirection {
    /// Order local sort values from lowest to highest.
    Asc,
    /// Order local sort values from highest to lowest.
    #[default]
    Desc,
}

impl NnsProposalSortDirection {
    /// Return the canonical filter or sort label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => NNS_PROPOSAL_SORT_ASC_LABEL,
            Self::Desc => NNS_PROPOSAL_SORT_DESC_LABEL,
        }
    }
}

///
/// NnsProposalStatusFilter
///
/// Report-model status filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalStatusFilter {
    /// Include proposals without restricting this classification.
    #[default]
    Any,
    /// Include only proposals still open for voting.
    Open,
    /// Include only rejected proposals.
    Rejected,
    /// Include only adopted proposals awaiting successful execution.
    Adopted,
    /// Include only successfully executed proposals.
    Executed,
    /// Include only proposals whose execution failed.
    Failed,
}

///
/// NnsProposalRewardStatusFilter
///
/// Report-model reward status filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalRewardStatusFilter {
    /// Include proposals without restricting this classification.
    #[default]
    Any,
    /// Include only proposals accepting votes for rewards.
    AcceptVotes,
    /// Include only proposals ready for reward settlement.
    ReadyToSettle,
    /// Include only proposals with settled rewards.
    Settled,
    /// Include only proposals ineligible for voting rewards.
    Ineligible,
}

impl NnsProposalRewardStatusFilter {
    /// Return the canonical filter or sort label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self.classification() {
            None => "any",
            Some(value) => value.as_str(),
        }
    }

    #[cfg(any(
        feature = "nns-host",
        all(feature = "canister", target_arch = "wasm32")
    ))]
    pub(in crate::nns) const fn governance_reward_status_code(self) -> Option<i32> {
        match self.classification() {
            None => None,
            Some(value) => Some(value.code()),
        }
    }

    const fn classification(self) -> Option<NnsProposalRewardStatus> {
        match self {
            Self::Any => None,
            Self::AcceptVotes => Some(NnsProposalRewardStatus::AcceptVotes),
            Self::ReadyToSettle => Some(NnsProposalRewardStatus::ReadyToSettle),
            Self::Settled => Some(NnsProposalRewardStatus::Settled),
            Self::Ineligible => Some(NnsProposalRewardStatus::Ineligible),
        }
    }
}

impl NnsProposalStatusFilter {
    /// Return the canonical filter or sort label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self.classification() {
            None => "any",
            Some(value) => value.as_str(),
        }
    }

    #[cfg(any(
        feature = "nns-host",
        all(feature = "canister", target_arch = "wasm32")
    ))]
    pub(in crate::nns) const fn governance_status_code(self) -> Option<i32> {
        match self.classification() {
            None => None,
            Some(value) => Some(value.code()),
        }
    }

    const fn classification(self) -> Option<NnsProposalStatus> {
        match self {
            Self::Any => None,
            Self::Open => Some(NnsProposalStatus::Open),
            Self::Rejected => Some(NnsProposalStatus::Rejected),
            Self::Adopted => Some(NnsProposalStatus::Adopted),
            Self::Executed => Some(NnsProposalStatus::Executed),
            Self::Failed => Some(NnsProposalStatus::Failed),
        }
    }
}

///
/// NnsProposalTopicFilter
///
/// Report-model topic filter for bounded NNS proposal listings.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NnsProposalTopicFilter {
    /// Include proposals without restricting this classification.
    #[default]
    Any,
    /// Include only proposals about neuron management.
    NeuronManagement,
    /// Include only proposals about exchange-rate management.
    ExchangeRate,
    /// Include only proposals about network economics.
    NetworkEconomics,
    /// Include only proposals about Governance policy.
    Governance,
    /// Include only proposals about node administration.
    NodeAdmin,
    /// Include only proposals about participant management.
    ParticipantManagement,
    /// Include only proposals about Subnet management.
    SubnetManagement,
    /// Include only proposals about network-canister management.
    NetworkCanisterManagement,
    /// Include only proposals about know-your-customer policy.
    Kyc,
    /// Include only proposals about node-provider rewards.
    NodeProviderRewards,
    /// Include only proposals about IC OS version deployment.
    IcOsVersionDeployment,
    /// Include only proposals about IC OS version election.
    IcOsVersionElection,
    /// Include only proposals about SNS and community-fund policy.
    SnsAndCommunityFund,
    /// Include only proposals about API boundary-node management.
    ApiBoundaryNodeManagement,
    /// Include only proposals about Subnet rental.
    SubnetRental,
    /// Include only proposals about application-canister management.
    ApplicationCanisterManagement,
    /// Include only proposals about protocol-canister management.
    ProtocolCanisterManagement,
}

impl NnsProposalTopicFilter {
    /// Return the canonical filter or sort label used in reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self.classification() {
            None => "any",
            Some(value) => value.as_str(),
        }
    }

    pub(in crate::nns) const fn topic_code(self) -> Option<i32> {
        match self.classification() {
            None => None,
            Some(value) => Some(value.code()),
        }
    }

    const fn classification(self) -> Option<NnsProposalTopic> {
        match self {
            Self::Any => None,
            Self::NeuronManagement => Some(NnsProposalTopic::NeuronManagement),
            Self::ExchangeRate => Some(NnsProposalTopic::ExchangeRate),
            Self::NetworkEconomics => Some(NnsProposalTopic::NetworkEconomics),
            Self::Governance => Some(NnsProposalTopic::Governance),
            Self::NodeAdmin => Some(NnsProposalTopic::NodeAdmin),
            Self::ParticipantManagement => Some(NnsProposalTopic::ParticipantManagement),
            Self::SubnetManagement => Some(NnsProposalTopic::SubnetManagement),
            Self::NetworkCanisterManagement => Some(NnsProposalTopic::NetworkCanisterManagement),
            Self::Kyc => Some(NnsProposalTopic::Kyc),
            Self::NodeProviderRewards => Some(NnsProposalTopic::NodeProviderRewards),
            Self::IcOsVersionDeployment => Some(NnsProposalTopic::IcOsVersionDeployment),
            Self::IcOsVersionElection => Some(NnsProposalTopic::IcOsVersionElection),
            Self::SnsAndCommunityFund => Some(NnsProposalTopic::SnsAndCommunityFund),
            Self::ApiBoundaryNodeManagement => Some(NnsProposalTopic::ApiBoundaryNodeManagement),
            Self::SubnetRental => Some(NnsProposalTopic::SubnetRental),
            Self::ApplicationCanisterManagement => {
                Some(NnsProposalTopic::ApplicationCanisterManagement)
            }
            Self::ProtocolCanisterManagement => Some(NnsProposalTopic::ProtocolCanisterManagement),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_status_codes_and_labels_round_trip() {
        for (status, code, label) in [
            (NnsProposalStatus::Unspecified, 0, "unspecified"),
            (NnsProposalStatus::Open, 1, "open"),
            (NnsProposalStatus::Rejected, 2, "rejected"),
            (NnsProposalStatus::Adopted, 3, "adopted"),
            (NnsProposalStatus::Executed, 4, "executed"),
            (NnsProposalStatus::Failed, 5, "failed"),
        ] {
            assert_eq!(status.code(), code);
            assert_eq!(NnsProposalStatus::from_code(code), status);
            assert_json_label(status, label);
        }
        assert_eq!(
            NnsProposalStatus::from_code(99),
            NnsProposalStatus::Unspecified
        );
    }

    #[test]
    fn proposal_reward_status_codes_and_labels_round_trip() {
        for (status, code, label) in [
            (NnsProposalRewardStatus::Unspecified, 0, "unspecified"),
            (NnsProposalRewardStatus::AcceptVotes, 1, "accept-votes"),
            (NnsProposalRewardStatus::ReadyToSettle, 2, "ready-to-settle"),
            (NnsProposalRewardStatus::Settled, 3, "settled"),
            (NnsProposalRewardStatus::Ineligible, 4, "ineligible"),
        ] {
            assert_eq!(status.code(), code);
            assert_eq!(NnsProposalRewardStatus::from_code(code), status);
            assert_json_label(status, label);
        }
        assert_eq!(
            NnsProposalRewardStatus::from_code(99),
            NnsProposalRewardStatus::Unspecified
        );
    }

    #[test]
    fn proposal_topic_codes_and_labels_round_trip() {
        for (topic, code, label) in [
            (NnsProposalTopic::Unspecified, 0, "unspecified"),
            (NnsProposalTopic::NeuronManagement, 1, "neuron-management"),
            (NnsProposalTopic::ExchangeRate, 2, "exchange-rate"),
            (NnsProposalTopic::NetworkEconomics, 3, "network-economics"),
            (NnsProposalTopic::Governance, 4, "governance"),
            (NnsProposalTopic::NodeAdmin, 5, "node-admin"),
            (
                NnsProposalTopic::ParticipantManagement,
                6,
                "participant-management",
            ),
            (NnsProposalTopic::SubnetManagement, 7, "subnet-management"),
            (
                NnsProposalTopic::NetworkCanisterManagement,
                8,
                "network-canister-management",
            ),
            (NnsProposalTopic::Kyc, 9, "kyc"),
            (
                NnsProposalTopic::NodeProviderRewards,
                10,
                "node-provider-rewards",
            ),
            (
                NnsProposalTopic::IcOsVersionDeployment,
                12,
                "ic-os-version-deployment",
            ),
            (
                NnsProposalTopic::IcOsVersionElection,
                13,
                "ic-os-version-election",
            ),
            (
                NnsProposalTopic::SnsAndCommunityFund,
                14,
                "sns-and-community-fund",
            ),
            (
                NnsProposalTopic::ApiBoundaryNodeManagement,
                15,
                "api-boundary-node-management",
            ),
            (NnsProposalTopic::SubnetRental, 16, "subnet-rental"),
            (
                NnsProposalTopic::ApplicationCanisterManagement,
                17,
                "application-canister-management",
            ),
            (
                NnsProposalTopic::ProtocolCanisterManagement,
                18,
                "protocol-canister-management",
            ),
        ] {
            assert_eq!(topic.code(), code);
            assert_eq!(NnsProposalTopic::from_code(code), topic);
            assert_json_label(topic, label);
        }
        assert_eq!(
            NnsProposalTopic::from_code(11),
            NnsProposalTopic::Unspecified
        );
        assert_eq!(
            NnsProposalTopic::from_code(99),
            NnsProposalTopic::Unspecified
        );
    }

    fn assert_json_label<T>(value: T, label: &str)
    where
        T: Copy + std::fmt::Debug + Eq + Serialize + serde::de::DeserializeOwned,
    {
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            format!("\"{label}\"")
        );
        assert_eq!(
            serde_json::from_str::<T>(&format!("\"{label}\"")).unwrap(),
            value
        );
    }
}
