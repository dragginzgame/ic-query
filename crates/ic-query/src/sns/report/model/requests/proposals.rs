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
