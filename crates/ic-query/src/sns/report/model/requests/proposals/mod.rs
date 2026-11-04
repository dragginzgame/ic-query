//! Module: sns::report::model::requests::proposals
//!
//! Responsibility: request DTOs for SNS proposal reports.
//! Does not own: command option parsing, live proposal fetches, or rendering.
//! Boundary: carries validated proposal inputs into SNS report builders.

use super::lookup::SnsLookupRequest;
use crate::report::SortDirection;
use crate::sns::report::{
    SnsProposalEligibilityFilter, SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort,
};
use std::path::{Path, PathBuf};
use thiserror::Error as ThisError;

///
/// SnsProposalCollection
///
/// Acquisition route for a bounded live proposal view or a complete-cache view.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnsProposalCollection {
    /// One bounded Governance response.
    Live,
    /// Complete snapshot, refreshing only when absent under the existing policy.
    Cached {
        /// Caller-selected managed cache root.
        cache_root: PathBuf,
    },
}

///
/// SnsProposalViewError
///
/// A caller selected a filter requiring complete cached proposal evidence.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError)]
#[error("`--status decided` requires a complete proposal cache")]
pub struct SnsProposalViewError;

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
}

impl SnsProposalRequest {
    #[must_use]
    pub const fn new(lookup: SnsLookupRequest, proposal_id: u64) -> Self {
        Self {
            lookup,
            proposal_id,
            cache_root: None,
        }
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
    status: SnsProposalStatusFilter,
    pub topic: SnsProposalTopicFilter,
    pub eligibility: SnsProposalEligibilityFilter,
    pub proposer_neuron_id: Option<String>,
    pub query: Option<String>,
    pub sort: SnsProposalsSort,
    pub sort_direction: SortDirection,
    collection: SnsProposalCollection,
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
            sort_direction: SortDirection::default(),
            collection: SnsProposalCollection::Live,
        }
    }

    /// Admit a status filter for the selected acquisition route.
    pub fn with_status(
        mut self,
        status: SnsProposalStatusFilter,
    ) -> Result<Self, SnsProposalViewError> {
        if status == SnsProposalStatusFilter::Decided
            && matches!(self.collection, SnsProposalCollection::Live)
        {
            return Err(SnsProposalViewError);
        }
        self.status = status;
        Ok(self)
    }

    #[must_use]
    pub const fn with_sort(mut self, sort: SnsProposalsSort) -> Self {
        self.sort = sort;
        self.sort_direction = sort.default_direction();
        self
    }

    #[must_use]
    pub fn with_cache_root(mut self, cache_root: impl Into<PathBuf>) -> Self {
        self.collection = SnsProposalCollection::Cached {
            cache_root: cache_root.into(),
        };
        self
    }

    /// Admitted acquisition route.
    #[must_use]
    pub const fn collection(&self) -> &SnsProposalCollection {
        &self.collection
    }

    /// Caller-selected status admitted for this acquisition route.
    #[must_use]
    pub const fn status(&self) -> SnsProposalStatusFilter {
        self.status
    }

    /// Root for complete-cache views, absent for bounded live acquisition.
    #[must_use]
    pub fn cache_root(&self) -> Option<&Path> {
        match &self.collection {
            SnsProposalCollection::Live => None,
            SnsProposalCollection::Cached { cache_root } => Some(cache_root),
        }
    }
}

#[cfg(test)]
mod tests;
