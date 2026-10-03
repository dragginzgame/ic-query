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
    /// Requested network identity; current live adapters support mainnet `ic`.
    pub network: String,
    /// Explicit replica API endpoint used for source calls.
    pub source_endpoint: String,
    /// Caller clock in Unix seconds used for freshness and provenance.
    pub now_unix_secs: u64,
    /// Deployed SNS list id or Root canister principal.
    pub input: String,
    /// Maximum rows in the requested view; does not change snapshot identity.
    pub limit: u32,
    /// Optional principal whose neuron permissions select a targeted live view.
    pub owner_principal_id: Option<String>,
    /// Ordering of displayed neurons; does not change the collected snapshot.
    pub sort: SnsNeuronsSort,
    /// Caller-selected root for confined managed cache files.
    pub cache_root: Option<PathBuf>,
    /// Include additional human-facing neuron detail in text output.
    pub verbose: bool,
}

impl SnsNeuronsRequest {
    /// Construct neuron query settings using the caller clock.
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

    /// Select a targeted owner-permission neuron query.
    #[must_use]
    pub fn with_owner_principal_id(mut self, owner_principal_id: impl Into<String>) -> Self {
        self.owner_principal_id = Some(owner_principal_id.into());
        self
    }

    /// Select view ordering without changing snapshot identity.
    #[must_use]
    pub const fn with_sort(mut self, sort: SnsNeuronsSort) -> Self {
        self.sort = sort;
        self
    }

    /// Enable reading the complete neuron cache beneath this root.
    #[must_use]
    pub fn with_cache_root(mut self, cache_root: impl Into<PathBuf>) -> Self {
        self.cache_root = Some(cache_root.into());
        self
    }

    /// Select detailed text rendering.
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
    /// Requested network identity; current live adapters support mainnet `ic`.
    pub network: String,
    /// Explicit replica API endpoint used for source calls.
    pub source_endpoint: String,
    /// Caller clock in Unix seconds used for freshness and provenance.
    pub now_unix_secs: u64,
    /// Deployed SNS list id or Root canister principal.
    pub input: String,
    /// Caller-selected root for confined managed cache files.
    pub cache_root: PathBuf,
    /// Maximum neurons requested per Governance page.
    pub page_size: u32,
    /// Optional page ceiling; reaching it before exhaustion prevents complete publication.
    pub max_pages: Option<u32>,
}

impl SnsNeuronsRefreshRequest {
    /// Construct neuron query settings using the caller clock.
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

    /// Bound refresh pagination; truncated collections are not published as complete.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self
    }
}
