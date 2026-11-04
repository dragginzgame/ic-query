//! Module: nns::proposals::report::model::selection
//!
//! Responsibility: NNS proposal filter, sort, and protocol vocabulary.
//! Does not own: request DTOs, serialized report DTOs, or report rendering.
//! Boundary: keeps public selection types and their canonical labels/codes together.

use crate::report::SortDirection;
use serde::{Deserialize, Serialize};

macro_rules! governance_code_enum {
    (
        $(#[$enum_attr:meta])*
        pub enum $name:ident {
            $(#[$unspecified_attr:meta])* Unspecified = 0 => "unspecified",
            $($(#[$variant_attr:meta])* $variant:ident = $code:literal => $label:literal,)*
        }
        filter $filter:ident;
    ) => {
        $(#[$enum_attr])*
        pub enum $name {
            $(#[$unspecified_attr])* #[serde(rename = "unspecified")] Unspecified,
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
                match self { Self::Unspecified => 0, $(Self::$variant => $code,)* }
            }

            /// Return the stable JSON and text label.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { Self::Unspecified => "unspecified", $(Self::$variant => $label,)* }
            }
        }
        ///
        #[doc = stringify!($filter)]
        ///
        /// Selection of one known proposal classification or all proposals.
        ///

        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub enum $filter {
            /// Do not restrict this classification.
            #[default]
            Any,
            $($(#[$variant_attr])* $variant,)*
        }

        impl $filter {
            /// All supported filter choices in presentation order.
            pub const ALL: &'static [Self] = &[Self::Any, $(Self::$variant,)*];

            /// Return the canonical filter label.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self.classification() { None => "any", Some(value) => value.as_str() }
            }

            const fn classification(self) -> Option<$name> {
                match self { Self::Any => None, $(Self::$variant => Some($name::$variant),)* }
            }
        }
    };
}

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
    filter NnsProposalStatusFilter;
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
    filter NnsProposalRewardStatusFilter;
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
    filter NnsProposalTopicFilter;
}

macro_rules! proposal_sort {
    ($($(#[$attr:meta])* $variant:ident => $label:literal,)*) => {
        ///
        /// NnsProposalListSort
        ///
        /// Report-model sort selector for NNS proposal listings.
        ///

        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub enum NnsProposalListSort { $($(#[$attr])* $variant,)* }

        impl NnsProposalListSort {
            /// All supported sort choices in presentation order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// Return the canonical sort label.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $label,)* }
            }
        }
    };
}

proposal_sort! {
    /// Preserve the descending proposal-id order returned by Governance.
    #[default]
    Api => "api",
    /// Sort by proposal id.
    Id => "id",
    /// Sort by the raw decision-status code.
    Status => "status",
    /// Sort by the raw reward-settlement code.
    RewardStatus => "reward-status",
    /// Sort by the raw topic code.
    Topic => "topic",
    /// Sort by proposer neuron id.
    Proposer => "proposer",
    /// Sort by proposal title.
    Title => "title",
    /// Sort by the human-readable action classification.
    Action => "action",
    /// Sort by affirmative voting power in the latest tally.
    Yes => "yes",
    /// Sort by negative voting power in the latest tally.
    No => "no",
    /// Sort by total voting power in the latest tally.
    TotalVotes => "total-votes",
    /// Sort by the latest tally timestamp.
    TallyTime => "tally-time",
    /// Sort by total potential voting power.
    VotingPower => "voting-power",
    /// Sort by ballot count returned by Governance.
    Ballots => "ballots",
    /// Sort by reject cost in ICP e8s.
    RejectCost => "reject-cost",
    /// Sort by reward event round.
    RewardRound => "reward-round",
    /// Sort by proposal creation time.
    Proposed => "proposed",
    /// Sort by voting deadline.
    Deadline => "deadline",
    /// Sort by decision time.
    Decided => "decided",
    /// Sort by execution time.
    Executed => "executed",
    /// Sort by execution failure time.
    Failed => "failed",
}

impl NnsProposalListSort {
    /// Return the default direction for this sort key.
    #[must_use]
    pub const fn default_direction(self) -> SortDirection {
        match self {
            Self::Status
            | Self::RewardStatus
            | Self::Topic
            | Self::Proposer
            | Self::Title
            | Self::Action => SortDirection::Asc,
            _ => SortDirection::Desc,
        }
    }

    /// Return whether this key applies a local sort direction.
    #[must_use]
    pub const fn uses_local_direction(self) -> bool {
        !matches!(self, Self::Api)
    }

    /// Return `none` for API order or the selected local direction label.
    #[must_use]
    pub const fn direction_label(self, direction: SortDirection) -> &'static str {
        match self {
            Self::Api => "none",
            _ => direction.as_str(),
        }
    }
}

impl NnsProposalStatusFilter {
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
}

impl NnsProposalRewardStatusFilter {
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
}

impl NnsProposalTopicFilter {
    pub(in crate::nns) const fn topic_code(self) -> Option<i32> {
        match self.classification() {
            None => None,
            Some(value) => Some(value.code()),
        }
    }
}

#[cfg(test)]
mod tests;
