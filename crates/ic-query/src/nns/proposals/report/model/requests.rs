//! Module: nns::proposals::report::model::requests
//!
//! Responsibility: NNS proposal list and detail request DTOs.
//! Does not own: source transport, serialized output, or view projection.
//! Boundary: captures caller intent before source and report assembly.

use super::selection::{
    NnsProposalListSort, NnsProposalRewardStatusFilter, NnsProposalStatusFilter,
    NnsProposalTopicFilter,
};
use crate::nns::governance::NnsGovernanceRequest;
use crate::report::SortDirection;

///
/// NnsProposalListRequest
///
/// Request accepted by the NNS proposal list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsProposalListRequest {
    /// Shared network, collection time, and source transport request.
    pub governance: NnsGovernanceRequest,
    /// Maximum returned rows; live requests must be between 1 and 100.
    pub limit: u32,
    /// Exclusive upper proposal-id bound; absent on the first page.
    pub before_proposal_id: Option<u64>,
    /// Decision-status filter applied to candidate proposals.
    pub status: NnsProposalStatusFilter,
    /// Reward-settlement filter applied to candidate proposals.
    pub reward_status: NnsProposalRewardStatusFilter,
    /// Topic filter applied to candidate proposals.
    pub topic: NnsProposalTopicFilter,
    /// Optional exact proposer neuron-id filter.
    pub proposer_neuron_id: Option<u64>,
    /// Optional case-insensitive substring search over title, summary, action, and URL.
    pub query: Option<String>,
    /// Requested API order or local proposal sort key.
    pub sort: NnsProposalListSort,
    /// Direction for local sorts; ignored when preserving API order.
    pub sort_direction: SortDirection,
}

impl NnsProposalListRequest {
    /// Create a first-page list request with no filters and API ordering.
    #[must_use]
    pub fn new(governance: NnsGovernanceRequest, limit: u32) -> Self {
        Self {
            governance,
            limit,
            before_proposal_id: None,
            status: NnsProposalStatusFilter::default(),
            reward_status: NnsProposalRewardStatusFilter::default(),
            topic: NnsProposalTopicFilter::default(),
            proposer_neuron_id: None,
            query: None,
            sort: NnsProposalListSort::default(),
            sort_direction: SortDirection::default(),
        }
    }

    /// Select a sort key and reset its direction to the key's default.
    #[must_use]
    pub const fn with_sort(mut self, sort: NnsProposalListSort) -> Self {
        self.sort = sort;
        self.sort_direction = sort.default_direction();
        self
    }
}

///
/// NnsProposalRequest
///
/// Request accepted by the NNS proposal detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsProposalRequest {
    /// Shared network, collection time, and source transport request.
    pub governance: NnsGovernanceRequest,
    /// Exact Governance proposal identifier to fetch.
    pub proposal_id: u64,
}

impl NnsProposalRequest {
    /// Create an exact proposal lookup with compact text and ballots hidden in text output.
    #[must_use]
    pub const fn new(governance: NnsGovernanceRequest, proposal_id: u64) -> Self {
        Self {
            governance,
            proposal_id,
        }
    }
}
