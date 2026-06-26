//! Module: sns::report::model::requests::neurons
//!
//! Responsibility: request DTOs for SNS neuron reports and cache commands.
//! Does not own: command option parsing, cache storage, or live neuron fetches.
//! Boundary: carries validated neuron inputs into SNS report builders.

use crate::sns::report::SnsNeuronsSort;
use std::path::{Path, PathBuf};

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

impl SnsNeuronsCacheListRequest {
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

impl SnsNeuronsCacheStatusRequest {
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

impl SnsNeuronsRefreshRequest {
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
