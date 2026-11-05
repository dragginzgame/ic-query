//! Module: sns::report::model::requests::neurons
//!
//! Responsibility: request DTOs for SNS neuron reports and cache commands.
//! Does not own: command option parsing, cache storage, or live neuron fetches.
//! Boundary: carries validated neuron inputs into SNS report builders.

use super::target::SnsNeuronTargetRequest;
use crate::sns::report::SnsNeuronsCacheSort;
use std::path::PathBuf;

///
/// SnsNeuronRequest
///
/// Request accepted by the exact SNS neuron detail report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronRequest {
    /// SNS target, explicit endpoint, network and caller clock.
    pub target: SnsNeuronTargetRequest,
    /// Exact 32-byte neuron id encoded as 64 lowercase hexadecimal characters.
    pub neuron_id: String,
}

impl SnsNeuronRequest {
    /// Construct one exact SNS neuron detail request.
    #[must_use]
    pub fn new(target: SnsNeuronTargetRequest, neuron_id: impl Into<String>) -> Self {
        Self {
            target,
            neuron_id: neuron_id.into(),
        }
    }
}

///
/// SnsNeuronCollection
///
/// Acquisition inputs for a bounded live page or a view over a complete cache.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnsNeuronCollection {
    /// One live API page, optionally scoped to a permission owner.
    Live {
        /// Optional principal whose permissions select the page.
        owner_principal_id: Option<String>,
    },
    /// A whole-collection cache view with local ordering.
    Cached {
        /// Caller-selected managed cache root.
        cache_root: PathBuf,
        /// Ordering applied after loading the complete snapshot.
        sort: SnsNeuronsCacheSort,
    },
}

impl SnsNeuronCollection {
    pub(in crate::sns::report) const fn sort_label(&self) -> &'static str {
        match self {
            Self::Live { .. } => "api",
            Self::Cached { sort, .. } => sort.as_str(),
        }
    }
}

///
/// SnsNeuronsRequest
///
/// A bounded live neuron page or a view over a complete neuron cache.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsRequest {
    /// Explicit target, endpoint, network and caller clock.
    pub target: SnsNeuronTargetRequest,
    /// Maximum displayed rows; does not change snapshot identity.
    pub limit: u32,
    /// Inputs specific to the selected acquisition route.
    pub collection: SnsNeuronCollection,
}

impl SnsNeuronsRequest {
    /// Request one live API page with optional owner filtering.
    #[must_use]
    pub const fn live(
        target: SnsNeuronTargetRequest,
        limit: u32,
        owner_principal_id: Option<String>,
    ) -> Self {
        Self {
            target,
            limit,
            collection: SnsNeuronCollection::Live { owner_principal_id },
        }
    }

    /// Request a locally ordered view of a complete snapshot.
    #[must_use]
    pub fn cached(
        target: SnsNeuronTargetRequest,
        limit: u32,
        cache_root: impl Into<PathBuf>,
        sort: SnsNeuronsCacheSort,
    ) -> Self {
        Self {
            target,
            limit,
            collection: SnsNeuronCollection::Cached {
                cache_root: cache_root.into(),
                sort,
            },
        }
    }
}
