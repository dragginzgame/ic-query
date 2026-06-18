//! Module: sns::report::model::requests::proposals
//!
//! Responsibility: request DTOs for SNS proposal reports.
//! Does not own: command option parsing, live proposal fetches, or rendering.
//! Boundary: carries validated proposal inputs into SNS report builders.

use crate::sns::report::{SnsProposalStatusFilter, SnsProposalTopicFilter, SnsProposalsSort};
use std::path::PathBuf;

///
/// SnsProposalsCacheListRequest
///
/// Request accepted by the local SNS proposal cache list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsCacheListRequest {
    pub network: String,
    pub icp_root: PathBuf,
}

///
/// SnsProposalsCacheStatusRequest
///
/// Request accepted by the local SNS proposal cache status report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsCacheStatusRequest {
    pub network: String,
    pub icp_root: PathBuf,
    pub input: String,
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
    pub sort: SnsProposalsSort,
    pub icp_root: Option<PathBuf>,
    pub verbose: bool,
}

///
/// SnsProposalsRefreshRequest
///
/// Request accepted by the complete SNS proposal snapshot refresh builder.
///

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
