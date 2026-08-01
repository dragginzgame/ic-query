//! Module: sns::report::live::types
//!
//! Responsibility: group live SNS Candid wire types.
//! Does not own: transport calls, report conversion, cache IO, or rendering.
//! Boundary: re-exports request and response types used by live fetch helpers.

mod canisters;
mod deployed;
mod metrics;
mod neurons;
mod proposals;
mod swap;
mod upgrade;

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
pub(in crate::sns::report::live) use metrics::{
    GetMetricsRequest, GetMetricsResponse, GetMetricsResult, MetricsWire, TreasuryMetricsWire,
    VotingPowerMetricsWire,
};
pub(in crate::sns::report::live) use neurons::{
    ListNeuronsRequest, ListNeuronsResponse, SnsGovernanceDissolveState, SnsGovernanceNeuron,
};
pub(in crate::sns::report::live) use proposals::{
    GetProposalRequest, GetProposalResponse, GetProposalResult, ListProposalsRequest,
    ListProposalsResponse, SnsGovernanceBallot, SnsGovernanceError, SnsGovernanceProposalData,
    SnsProposalId, SnsTopic, SnsTopicSelector,
};
pub(in crate::sns::report::live) use swap::{
    GetDerivedStateResponse, GetLifecycleResponse, GetSaleParametersResponse, SnsSwapParams,
    SnsSwapQueryRequest,
};
pub(in crate::sns::report::live) use upgrade::{
    GetNextSnsVersionRequest, GetNextSnsVersionResponse, GetRunningSnsVersionRequest,
    GetRunningSnsVersionResponse, PendingSnsVersion, SnsVersionWire,
};
