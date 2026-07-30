//! Module: sns::report::live::types
//!
//! Responsibility: group live SNS Candid wire types.
//! Does not own: transport calls, report conversion, cache IO, or rendering.
//! Boundary: re-exports request and response types used by live fetch helpers.

mod canisters;
mod deployed;
mod neurons;
mod proposals;

pub(in crate::sns::report::live) use canisters::{
    CanisterStatusResult, CanisterStatusType, CanisterSummary, GetSnsCanistersSummaryRequest,
    GetSnsCanistersSummaryResponse, ListSnsCanistersRequest, ListSnsCanistersResponse,
};
#[cfg(test)]
pub(in crate::sns::report::live) use canisters::{DefiniteCanisterSettings, SnsRootExtensions};
pub(in crate::sns::report::live) use deployed::{
    DeployedSns, GetMetadataRequest, GetMetadataResponse, ListDeployedSnsesRequest,
    ListDeployedSnsesResponse,
};
pub(in crate::sns::report::live) use neurons::{
    ListNeuronsRequest, ListNeuronsResponse, SnsGovernanceNeuron,
};
pub(in crate::sns::report::live) use proposals::{
    GetProposalRequest, GetProposalResponse, GetProposalResult, ListProposalsRequest,
    ListProposalsResponse, SnsGovernanceBallot, SnsGovernanceProposalData, SnsProposalId, SnsTopic,
    SnsTopicSelector,
};
