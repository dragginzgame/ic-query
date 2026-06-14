use super::{SnsListSort, SnsNeuronsSort, SnsProposalStatusFilter};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsListRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub verbose: bool,
    pub sort: SnsListSort,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsLookupRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
}

pub type SnsInfoRequest = SnsLookupRequest;
pub type SnsParamsRequest = SnsLookupRequest;
pub type SnsTokenRequest = SnsLookupRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub proposal_id: u64,
    pub verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsProposalsRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status: SnsProposalStatusFilter,
    pub verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsCacheListRequest {
    pub network: String,
    pub icp_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsCacheStatusRequest {
    pub network: String,
    pub icp_root: PathBuf,
    pub input: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub limit: u32,
    pub owner_principal_id: Option<String>,
    pub sort: SnsNeuronsSort,
    pub icp_root: Option<PathBuf>,
    pub verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsRefreshRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub icp_root: PathBuf,
    pub page_size: u32,
    pub max_pages: Option<u32>,
}
