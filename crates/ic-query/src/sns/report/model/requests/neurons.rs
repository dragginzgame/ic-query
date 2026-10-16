//! Module: sns::report::model::requests::neurons
//!
//! Responsibility: request DTOs for SNS neuron reports and cache commands.
//! Does not own: command option parsing, cache storage, or live neuron fetches.
//! Boundary: carries validated neuron inputs into SNS report builders.

use super::lookup::SnsLookupRequest;
use crate::sns::report::SnsNeuronsSort;
use std::path::PathBuf;

///
/// SnsNeuronRequest
///
/// Request accepted by the exact SNS neuron detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
    /// Exact 32-byte neuron id encoded as 64 lowercase hexadecimal characters.
    pub neuron_id: String,
}

impl SnsNeuronRequest {
    /// Construct one exact SNS neuron detail request.
    #[must_use]
    pub fn new(lookup: SnsLookupRequest, neuron_id: impl Into<String>) -> Self {
        Self {
            lookup,
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
    /// SNS target, explicit endpoint, network and caller clock.
    pub lookup: SnsLookupRequest,
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
    pub fn new(lookup: SnsLookupRequest, limit: u32) -> Self {
        Self {
            lookup,
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
