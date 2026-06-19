//! Module: nns::proposals::report::wire
//!
//! Responsibility: Candid wire types for NNS governance proposal queries.
//! Does not own: live transport, report DTOs, or text rendering.
//! Boundary: models only the fields needed by NNS proposal reports.

use candid::{CandidType, Deserialize, Reserved};

///
/// NnsListProposalInfoRequest
///
/// Candid request for bounded NNS governance proposal listings.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsListProposalInfoRequest {
    pub(in crate::nns::proposals::report) include_reward_status: Vec<i32>,
    pub(in crate::nns::proposals::report) omit_large_fields: Option<bool>,
    pub(in crate::nns::proposals::report) before_proposal: Option<NnsProposalId>,
    pub(in crate::nns::proposals::report) limit: u32,
    pub(in crate::nns::proposals::report) exclude_topic: Vec<i32>,
    pub(in crate::nns::proposals::report) include_all_manage_neuron_proposals: Option<bool>,
    pub(in crate::nns::proposals::report) include_status: Vec<i32>,
    pub(in crate::nns::proposals::report) return_self_describing_action: Option<bool>,
}

///
/// NnsListProposalInfoResponse
///
/// Candid response containing bounded NNS governance proposal rows.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsListProposalInfoResponse {
    pub(in crate::nns::proposals::report) proposal_info: Vec<NnsProposalInfo>,
}

///
/// NnsProposalId
///
/// Candid NNS governance proposal identifier.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsProposalId {
    pub(in crate::nns::proposals::report) id: u64,
}

///
/// NnsNeuronId
///
/// Candid NNS governance neuron identifier.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsNeuronId {
    pub(in crate::nns::proposals::report) id: u64,
}

///
/// NnsProposal
///
/// Candid NNS proposal content fields used in reports.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsProposal {
    pub(in crate::nns::proposals::report) url: String,
    pub(in crate::nns::proposals::report) title: Option<String>,
    pub(in crate::nns::proposals::report) action: Option<NnsProposalAction>,
    pub(in crate::nns::proposals::report) summary: String,
}

///
/// NnsProposalAction
///
/// Label-only Candid NNS proposal action variant.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) enum NnsProposalAction {
    RegisterKnownNeuron(Reserved),
    DeregisterKnownNeuron(Reserved),
    ManageNeuron(Reserved),
    UpdateCanisterSettings(Reserved),
    InstallCode(Reserved),
    StopOrStartCanister(Reserved),
    CreateServiceNervousSystem(Reserved),
    ExecuteNnsFunction(Reserved),
    RewardNodeProvider(Reserved),
    OpenSnsTokenSwap(Reserved),
    SetSnsTokenSwapOpenTimeWindow(Reserved),
    SetDefaultFollowees(Reserved),
    RewardNodeProviders(Reserved),
    ManageNetworkEconomics(Reserved),
    ApproveGenesisKyc(Reserved),
    AddOrRemoveNodeProvider(Reserved),
    Motion(Reserved),
    FulfillSubnetRentalRequest(Reserved),
    BlessAlternativeGuestOsVersion(Reserved),
    TakeCanisterSnapshot(Reserved),
    LoadCanisterSnapshot(Reserved),
    CreateCanisterAndInstallCode(Reserved),
}

impl NnsProposalAction {
    pub(in crate::nns::proposals::report) const fn as_str(&self) -> &'static str {
        match self {
            Self::RegisterKnownNeuron(_) => "register-known-neuron",
            Self::DeregisterKnownNeuron(_) => "deregister-known-neuron",
            Self::ManageNeuron(_) => "manage-neuron",
            Self::UpdateCanisterSettings(_) => "update-canister-settings",
            Self::InstallCode(_) => "install-code",
            Self::StopOrStartCanister(_) => "stop-or-start-canister",
            Self::CreateServiceNervousSystem(_) => "create-service-nervous-system",
            Self::ExecuteNnsFunction(_) => "execute-nns-function",
            Self::RewardNodeProvider(_) => "reward-node-provider",
            Self::OpenSnsTokenSwap(_) => "open-sns-token-swap",
            Self::SetSnsTokenSwapOpenTimeWindow(_) => "set-sns-token-swap-open-time-window",
            Self::SetDefaultFollowees(_) => "set-default-followees",
            Self::RewardNodeProviders(_) => "reward-node-providers",
            Self::ManageNetworkEconomics(_) => "manage-network-economics",
            Self::ApproveGenesisKyc(_) => "approve-genesis-kyc",
            Self::AddOrRemoveNodeProvider(_) => "add-or-remove-node-provider",
            Self::Motion(_) => "motion",
            Self::FulfillSubnetRentalRequest(_) => "fulfill-subnet-rental-request",
            Self::BlessAlternativeGuestOsVersion(_) => "bless-alternative-guest-os-version",
            Self::TakeCanisterSnapshot(_) => "take-canister-snapshot",
            Self::LoadCanisterSnapshot(_) => "load-canister-snapshot",
            Self::CreateCanisterAndInstallCode(_) => "create-canister-and-install-code",
        }
    }
}

///
/// NnsGovernanceBallot
///
/// Candid NNS governance ballot row.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsGovernanceBallot {
    pub(in crate::nns::proposals::report) vote: i32,
    pub(in crate::nns::proposals::report) voting_power: u64,
}

///
/// NnsProposalTallyWire
///
/// Candid NNS proposal vote tally.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsProposalTallyWire {
    pub(in crate::nns::proposals::report) no: u64,
    pub(in crate::nns::proposals::report) yes: u64,
    pub(in crate::nns::proposals::report) total: u64,
    pub(in crate::nns::proposals::report) timestamp_seconds: u64,
}

///
/// NnsProposalInfo
///
/// Candid NNS governance proposal row returned by governance.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::nns::proposals::report) struct NnsProposalInfo {
    pub(in crate::nns::proposals::report) id: Option<NnsProposalId>,
    pub(in crate::nns::proposals::report) status: i32,
    pub(in crate::nns::proposals::report) topic: i32,
    pub(in crate::nns::proposals::report) ballots: Vec<(u64, NnsGovernanceBallot)>,
    pub(in crate::nns::proposals::report) proposal_timestamp_seconds: u64,
    pub(in crate::nns::proposals::report) reward_event_round: u64,
    pub(in crate::nns::proposals::report) deadline_timestamp_seconds: Option<u64>,
    pub(in crate::nns::proposals::report) failed_timestamp_seconds: u64,
    pub(in crate::nns::proposals::report) reject_cost_e8s: u64,
    pub(in crate::nns::proposals::report) latest_tally: Option<NnsProposalTallyWire>,
    pub(in crate::nns::proposals::report) reward_status: i32,
    pub(in crate::nns::proposals::report) decided_timestamp_seconds: u64,
    pub(in crate::nns::proposals::report) proposal: Option<NnsProposal>,
    pub(in crate::nns::proposals::report) proposer: Option<NnsNeuronId>,
    pub(in crate::nns::proposals::report) executed_timestamp_seconds: u64,
    pub(in crate::nns::proposals::report) total_potential_voting_power: Option<u64>,
}
