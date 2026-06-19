//! Module: nns::proposals::report
//!
//! Responsibility: build NNS governance proposal reports.
//! Does not own: CLI parsing, SNS proposal reports, cache files, or topology reports.
//! Boundary: maps live NNS governance proposal rows into text and JSON reports.

mod model;
mod source;
mod text;
mod view;
mod wire;

use crate::{
    ic_registry::{DEFAULT_MAINNET_ENDPOINT, MAINNET_GOVERNANCE_CANISTER_ID},
    subnet_catalog::MAINNET_NETWORK,
};
use thiserror::Error as ThisError;

pub(in crate::nns) use model::{
    NNS_PROPOSAL_REWARD_STATUS_ANY_LABEL, NNS_PROPOSAL_SORT_API_LABEL, NNS_PROPOSAL_SORT_ASC_LABEL,
    NNS_PROPOSAL_SORT_DESC_LABEL, NNS_PROPOSAL_STATUS_ANY_LABEL, NNS_PROPOSAL_TOPIC_ANY_LABEL,
    NnsProposalRequest, NnsProposalRewardStatusFilter, NnsProposalSortDirection,
    NnsProposalStatusFilter, NnsProposalTopicFilter, NnsProposalsRequest, NnsProposalsSort,
};
pub(in crate::nns::proposals) use source::{build_nns_proposal_report, build_nns_proposals_report};
pub(in crate::nns::proposals) use text::{nns_proposal_report_text, nns_proposals_report_text};

#[cfg(test)]
pub(in crate::nns) use model::{
    NNS_PROPOSAL_REWARD_STATUS_SETTLED_LABEL, NNS_PROPOSAL_SORT_NONE_LABEL,
    NNS_PROPOSAL_SORT_TITLE_LABEL, NNS_PROPOSAL_STATUS_EXECUTED_LABEL,
    NNS_PROPOSAL_TOPIC_GOVERNANCE_LABEL,
};

#[cfg(test)]
mod tests;

pub(in crate::nns) const DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;

const NNS_PROPOSAL_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSALS_REPORT_SCHEMA_VERSION: u32 = 1;

///
/// NnsProposalHostError
///
/// Error returned while building NNS proposal reports.
///

#[derive(Debug, ThisError)]
pub enum NnsProposalHostError {
    #[error(
        "`icq nns proposal` supports only the mainnet `ic` network\n\nThe NNS proposal list is queried from the public Internet Computer mainnet governance canister.\nLocal replica NNS governance discovery is not implemented yet.\n\nTry:\n  icq --network ic nns proposal list"
    )]
    LocalNetworkUnsupported,

    #[error("failed to build IC agent for {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("NNS governance agent call {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("failed to encode candid {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("failed to decode candid {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error("NNS proposal {proposal_id} was not found")]
    ProposalNotFound { proposal_id: u64 },

    #[error("failed to create Tokio runtime for NNS proposal query: {0}")]
    Runtime(String),
}

fn enforce_mainnet_network(network: &str) -> Result<(), NnsProposalHostError> {
    if network == MAINNET_NETWORK {
        Ok(())
    } else {
        Err(NnsProposalHostError::LocalNetworkUnsupported)
    }
}
