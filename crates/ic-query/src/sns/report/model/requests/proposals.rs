//! Module: sns::report::model::requests::proposals
//!
//! Responsibility: request DTOs for SNS proposal reports.
//! Does not own: command option parsing, live proposal fetches, or rendering.
//! Boundary: carries validated proposal inputs into SNS report builders.

use crate::sns::report::{
    SnsProposalEligibilityFilter, SnsProposalSortDirection, SnsProposalStatusFilter,
    SnsProposalTopicFilter, SnsProposalsSort,
};
#[cfg(feature = "host")]
use std::path::Path;
use std::path::PathBuf;

///
/// SnsProposalsCacheListRequest
///
/// Request accepted by the local SNS proposal cache list report builder.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsCacheListRequest {
    pub network: String,
    pub icp_root: PathBuf,
}

#[cfg(feature = "host")]
impl SnsProposalsCacheListRequest {
    #[must_use]
    pub fn new(icp_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            network: network.into(),
            icp_root: icp_root.into(),
        }
    }

    #[must_use]
    pub fn icp_root(&self) -> &Path {
        &self.icp_root
    }
}

///
/// SnsProposalsCacheStatusRequest
///
/// Request accepted by the local SNS proposal cache status report builder.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsCacheStatusRequest {
    pub network: String,
    pub icp_root: PathBuf,
    pub input: String,
}

#[cfg(feature = "host")]
impl SnsProposalsCacheStatusRequest {
    #[must_use]
    pub fn new(
        icp_root: impl Into<PathBuf>,
        network: impl Into<String>,
        input: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            icp_root: icp_root.into(),
            input: input.into(),
        }
    }

    #[must_use]
    pub fn icp_root(&self) -> &Path {
        &self.icp_root
    }
}

///
/// SnsProposalRequest
///
/// Request accepted by the direct SNS proposal detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub proposal_id: u64,
    pub icp_root: Option<PathBuf>,
    pub verbose: bool,
    pub show_ballots: bool,
}

impl SnsProposalRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
        proposal_id: u64,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input: input.into(),
            proposal_id,
            icp_root: None,
            verbose: false,
            show_ballots: false,
        }
    }

    #[must_use]
    pub fn with_icp_root(mut self, icp_root: impl Into<PathBuf>) -> Self {
        self.icp_root = Some(icp_root.into());
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
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status: SnsProposalStatusFilter,
    pub topic: SnsProposalTopicFilter,
    pub eligibility: SnsProposalEligibilityFilter,
    pub proposer_neuron_id: Option<String>,
    pub query: Option<String>,
    pub sort: SnsProposalsSort,
    pub sort_direction: SnsProposalSortDirection,
    pub icp_root: Option<PathBuf>,
    pub verbose: bool,
}

impl SnsProposalsRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
        limit: u32,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input: input.into(),
            limit,
            before_proposal_id: None,
            status: SnsProposalStatusFilter::default(),
            topic: SnsProposalTopicFilter::default(),
            eligibility: SnsProposalEligibilityFilter::default(),
            proposer_neuron_id: None,
            query: None,
            sort: SnsProposalsSort::default(),
            sort_direction: SnsProposalSortDirection::default(),
            icp_root: None,
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
    pub fn with_icp_root(mut self, icp_root: impl Into<PathBuf>) -> Self {
        self.icp_root = Some(icp_root.into());
        self
    }

    #[must_use]
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

///
/// SnsProposalsRefreshRequest
///
/// Request accepted by the complete SNS proposal snapshot refresh builder.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsRefreshRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub icp_root: PathBuf,
    pub page_size: u32,
    pub max_pages: Option<u32>,
}

#[cfg(feature = "host")]
impl SnsProposalsRefreshRequest {
    #[must_use]
    pub fn new(
        icp_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
        page_size: u32,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input: input.into(),
            icp_root: icp_root.into(),
            page_size,
            max_pages: None,
        }
    }

    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self
    }
}
