//! Module: sns::report::model::requests::neurons
//!
//! Responsibility: request DTOs for SNS neuron reports and cache commands.
//! Does not own: command option parsing, cache storage, or live neuron fetches.
//! Boundary: carries validated neuron inputs into SNS report builders.

use crate::sns::report::SnsNeuronsSort;
use std::path::PathBuf;

///
/// SnsNeuronsCacheListRequest
///
/// Request accepted by the local SNS neuron cache list report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsCacheListRequest {
    pub network: String,
    pub icp_root: PathBuf,
}

///
/// SnsNeuronsCacheStatusRequest
///
/// Request accepted by the local SNS neuron cache status report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsCacheStatusRequest {
    pub network: String,
    pub icp_root: PathBuf,
    pub input: String,
}

///
/// SnsNeuronsRequest
///
/// Request accepted by the SNS neuron listing report builder.
///

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

///
/// SnsNeuronsRefreshRequest
///
/// Request accepted by the complete SNS neuron snapshot refresh builder.
///

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
