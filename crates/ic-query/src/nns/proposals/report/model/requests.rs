//! Module: nns::proposals::report::model::requests
//!
//! Responsibility: NNS proposal list and detail request DTOs.
//! Does not own: source transport, serialized output, or view projection.
//! Boundary: captures caller intent before source and report assembly.

use super::selection::{
    NnsProposalListSort, NnsProposalRewardStatusFilter, NnsProposalSortDirection,
    NnsProposalStatusFilter, NnsProposalTopicFilter,
};
use crate::nns::governance::NnsGovernanceRequest;

///
/// NnsProposalListRequest
///
/// Request accepted by the NNS proposal list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsProposalListRequest {
    /// Shared network, collection time, and source transport request.
    pub governance: NnsGovernanceRequest,
    pub limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status: NnsProposalStatusFilter,
    pub reward_status: NnsProposalRewardStatusFilter,
    pub topic: NnsProposalTopicFilter,
    pub proposer_neuron_id: Option<u64>,
    pub query: Option<String>,
    pub sort: NnsProposalListSort,
    pub sort_direction: NnsProposalSortDirection,
    pub verbose: bool,
}

impl NnsProposalListRequest {
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
            sort_direction: NnsProposalSortDirection::default(),
            verbose: false,
        }
    }

    #[must_use]
    pub const fn with_before_proposal_id(mut self, before_proposal_id: u64) -> Self {
        self.before_proposal_id = Some(before_proposal_id);
        self
    }

    #[must_use]
    pub const fn with_status(mut self, status: NnsProposalStatusFilter) -> Self {
        self.status = status;
        self
    }

    #[must_use]
    pub const fn with_reward_status(
        mut self,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> Self {
        self.reward_status = reward_status;
        self
    }

    #[must_use]
    pub const fn with_topic(mut self, topic: NnsProposalTopicFilter) -> Self {
        self.topic = topic;
        self
    }

    #[must_use]
    pub const fn with_proposer_neuron_id(mut self, proposer_neuron_id: u64) -> Self {
        self.proposer_neuron_id = Some(proposer_neuron_id);
        self
    }

    #[must_use]
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    #[must_use]
    pub const fn with_sort(mut self, sort: NnsProposalListSort) -> Self {
        self.sort = sort;
        self.sort_direction = sort.default_direction();
        self
    }

    #[must_use]
    pub const fn with_sort_direction(mut self, sort_direction: NnsProposalSortDirection) -> Self {
        self.sort_direction = sort_direction;
        self
    }

    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
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
    pub proposal_id: u64,
    pub show_ballots: bool,
    pub verbose: bool,
}

impl NnsProposalRequest {
    #[must_use]
    pub const fn new(governance: NnsGovernanceRequest, proposal_id: u64) -> Self {
        Self {
            governance,
            proposal_id,
            show_ballots: false,
            verbose: false,
        }
    }

    #[must_use]
    pub const fn with_show_ballots(mut self, show_ballots: bool) -> Self {
        self.show_ballots = show_ballots;
        self
    }

    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}
