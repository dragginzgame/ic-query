//! Module: sns::report::model::requests::neurons
//!
//! Responsibility: request DTOs for SNS neuron reports and cache commands.
//! Does not own: command option parsing, cache storage, or live neuron fetches.
//! Boundary: carries validated neuron inputs into SNS report builders.

use crate::sns::report::SnsNeuronsSort;
use std::path::PathBuf;

///
/// SnsNeuronRequest
///
/// Request accepted by the exact SNS neuron detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronRequest {
    /// Requested IC network identity.
    pub network: String,
    /// Explicit IC API endpoint used for live calls.
    pub source_endpoint: String,
    /// Collection timestamp supplied by the caller.
    pub now_unix_secs: u64,
    /// SNS list id or Root canister principal.
    pub input: String,
    /// Exact 32-byte neuron id encoded as 64 lowercase hexadecimal characters.
    pub neuron_id: String,
}

impl SnsNeuronRequest {
    /// Construct one exact SNS neuron detail request.
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        input: impl Into<String>,
        neuron_id: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            input: input.into(),
            neuron_id: neuron_id.into(),
        }
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
    pub cache_root: Option<PathBuf>,
    pub verbose: bool,
}

impl SnsNeuronsRequest {
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
            owner_principal_id: None,
            sort: SnsNeuronsSort::default(),
            cache_root: None,
            verbose: false,
        }
    }

    #[must_use]
    pub fn with_owner_principal_id(mut self, owner_principal_id: impl Into<String>) -> Self {
        self.owner_principal_id = Some(owner_principal_id.into());
        self
    }

    #[must_use]
    pub const fn with_sort(mut self, sort: SnsNeuronsSort) -> Self {
        self.sort = sort;
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
    pub cache_root: PathBuf,
    pub page_size: u32,
    pub max_pages: Option<u32>,
}

impl SnsNeuronsRefreshRequest {
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
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
            cache_root: cache_root.into(),
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
