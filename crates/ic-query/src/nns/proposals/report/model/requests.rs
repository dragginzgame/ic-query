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
    pub sort_direction: NnsProposalSortDirection,
    /// Whether the text renderer includes expanded proposal fields.
    pub verbose: bool,
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
            sort_direction: NnsProposalSortDirection::default(),
            verbose: false,
        }
    }

    /// Start strictly before the selected proposal id.
    #[must_use]
    pub const fn with_before_proposal_id(mut self, before_proposal_id: u64) -> Self {
        self.before_proposal_id = Some(before_proposal_id);
        self
    }

    /// Select the required decision status.
    #[must_use]
    pub const fn with_status(mut self, status: NnsProposalStatusFilter) -> Self {
        self.status = status;
        self
    }

    /// Select the required reward-settlement status.
    #[must_use]
    pub const fn with_reward_status(
        mut self,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> Self {
        self.reward_status = reward_status;
        self
    }

    /// Select the required proposal topic.
    #[must_use]
    pub const fn with_topic(mut self, topic: NnsProposalTopicFilter) -> Self {
        self.topic = topic;
        self
    }

    /// Require the exact proposer neuron id.
    #[must_use]
    pub const fn with_proposer_neuron_id(mut self, proposer_neuron_id: u64) -> Self {
        self.proposer_neuron_id = Some(proposer_neuron_id);
        self
    }

    /// Search title, summary, action, and URL using case-insensitive substring matching.
    #[must_use]
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Select a sort key and reset its direction to the key's default.
    #[must_use]
    pub const fn with_sort(mut self, sort: NnsProposalListSort) -> Self {
        self.sort = sort;
        self.sort_direction = sort.default_direction();
        self
    }

    /// Override the direction used by local sorting.
    #[must_use]
    pub const fn with_sort_direction(mut self, sort_direction: NnsProposalSortDirection) -> Self {
        self.sort_direction = sort_direction;
        self
    }

    /// Select compact or expanded human-facing text output.
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
    /// Exact Governance proposal identifier to fetch.
    pub proposal_id: u64,
    /// Whether the human-facing detail renderer displays ballot rows.
    pub show_ballots: bool,
    /// Whether the text renderer includes expanded proposal fields.
    pub verbose: bool,
}

impl NnsProposalRequest {
    /// Create an exact proposal lookup with compact text and ballots hidden in text output.
    #[must_use]
    pub const fn new(governance: NnsGovernanceRequest, proposal_id: u64) -> Self {
        Self {
            governance,
            proposal_id,
            show_ballots: false,
            verbose: false,
        }
    }

    /// Include or omit ballots in human-facing detail output.
    #[must_use]
    pub const fn with_show_ballots(mut self, show_ballots: bool) -> Self {
        self.show_ballots = show_ballots;
        self
    }

    /// Select compact or expanded human-facing text output.
    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}
