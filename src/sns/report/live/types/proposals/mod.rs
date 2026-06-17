//! Module: sns::report::live::types::proposals
//!
//! Responsibility: group SNS governance proposal Candid wire types.
//! Does not own: live transport, report conversion, cache IO, or rendering.
//! Boundary: re-exports proposal request, topic, and response data types.

mod data;
mod request;
mod topic;

pub(in crate::sns::report::live) use data::{
    GetProposalResult, SnsGovernanceBallot, SnsGovernanceProposalData, SnsProposalId,
};
pub(in crate::sns::report::live) use request::{
    GetProposalRequest, GetProposalResponse, ListProposalsRequest, ListProposalsResponse,
};
pub(in crate::sns::report::live) use topic::{SnsTopic, SnsTopicSelector};
