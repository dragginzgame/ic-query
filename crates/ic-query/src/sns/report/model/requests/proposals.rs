//! Module: sns::report::model::requests::proposals
//!
//! Responsibility: request DTOs for SNS proposal reports.
//! Does not own: command option parsing, live proposal fetches, or rendering.
//! Boundary: carries validated proposal inputs into SNS report builders.

use super::lookup::SnsLookupRequest;
use crate::sns::report::{
    SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
use std::path::PathBuf;

///
/// SnsProposalRequest
///
/// Request accepted by the direct SNS proposal detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
    pub proposal_id: u64,
    pub cache_root: Option<PathBuf>,
    pub verbose: bool,
    pub show_ballots: bool,
}

impl SnsProposalRequest {
    #[must_use]
    pub const fn new(lookup: SnsLookupRequest, proposal_id: u64) -> Self {
        Self {
            lookup,
            proposal_id,
            cache_root: None,
            verbose: false,
            show_ballots: false,
        }
    }

    #[must_use]
    pub fn with_cache_root(mut self, cache_root: impl Into<PathBuf>) -> Self {
        self.cache_root = Some(cache_root.into());
        self
    }

    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    #[must_use]
    pub const fn with_show_ballots(mut self, show_ballots: bool) -> Self {
        self.show_ballots = show_ballots;
        self
    }
}

///
/// SnsProposalsRequest
///
/// Request accepted by the bounded SNS proposal listing report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
    pub limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status: SnsProposalStatusFilter,
    pub topic: SnsProposalTopicFilter,
    pub eligibility: SnsProposalEligibilityFilter,
    pub proposer_neuron_id: Option<String>,
    pub query: Option<String>,
    pub sort: SnsProposalsSort,
    pub sort_direction: SnsProposalSortDirection,
    pub cache_root: Option<PathBuf>,
    pub verbose: bool,
}

impl SnsProposalsRequest {
    #[must_use]
    pub fn new(lookup: SnsLookupRequest, limit: u32) -> Self {
        Self {
            lookup,
            limit,
            before_proposal_id: None,
            status: SnsProposalStatusFilter::default(),
            topic: SnsProposalTopicFilter::default(),
            eligibility: SnsProposalEligibilityFilter::default(),
            proposer_neuron_id: None,
            query: None,
            sort: SnsProposalsSort::default(),
            sort_direction: SnsProposalSortDirection::default(),
            cache_root: None,
            verbose: false,
        }
    }

    #[must_use]
    pub const fn with_before_proposal_id(mut self, before_proposal_id: u64) -> Self {
        self.before_proposal_id = Some(before_proposal_id);
        self
    }

    #[must_use]
    pub const fn with_status(mut self, status: SnsProposalStatusFilter) -> Self {
        self.status = status;
        self
    }

    #[must_use]
    pub const fn with_topic(mut self, topic: SnsProposalTopicFilter) -> Self {
        self.topic = topic;
        self
    }

    #[must_use]
    pub const fn with_eligibility(mut self, eligibility: SnsProposalEligibilityFilter) -> Self {
        self.eligibility = eligibility;
        self
    }

    #[must_use]
    pub fn with_proposer_neuron_id(mut self, proposer_neuron_id: impl Into<String>) -> Self {
        self.proposer_neuron_id = Some(proposer_neuron_id.into());
        self
    }

    #[must_use]
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    #[must_use]
    pub const fn with_sort(mut self, sort: SnsProposalsSort) -> Self {
        self.sort = sort;
        self.sort_direction = sort.default_direction();
        self
    }

    #[must_use]
    pub const fn with_sort_direction(mut self, sort_direction: SnsProposalSortDirection) -> Self {
        self.sort_direction = sort_direction;
        self
    }

    #[must_use]
    pub fn with_cache_root(mut self, cache_root: impl Into<PathBuf>) -> Self {
        self.cache_root = Some(cache_root.into());
        self
    }

    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}
