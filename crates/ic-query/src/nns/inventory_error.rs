//! Shared failures for Registry-derived NNS inventory operations.

use crate::{HostCacheError, ic_registry::RegistryFetchError};
use std::{fmt, path::PathBuf};
use thiserror::Error;

///
/// NnsInventoryCollection
///
/// Inventory family retained in a typed operation failure.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NnsInventoryCollection {
    /// Mainnet node inventory.
    Node,
    /// Mainnet node-provider inventory.
    NodeProvider,
    /// Mainnet node-operator inventory.
    NodeOperator,
    /// Mainnet data-center inventory.
    DataCenter,
}

impl NnsInventoryCollection {
    /// Stable command and cache component name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::NodeProvider => "node-provider",
            Self::NodeOperator => "node-operator",
            Self::DataCenter => "data-center",
        }
    }
}

impl fmt::Display for NnsInventoryCollection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// NnsInventoryError
///
/// Shared inventory failures retaining family identity and original source evidence.
///

#[derive(Debug, Error)]
pub enum NnsInventoryError {
    /// The selected inventory is restricted to mainnet.
    #[error(
        "NNS {collection} reports support only the mainnet `ic` network (requested {network:?})"
    )]
    UnsupportedNetwork {
        /// Requested inventory family.
        collection: NnsInventoryCollection,
        /// Original requested network.
        network: String,
    },
    /// Managed-cache refusal, including its original component and path.
    #[error("{source}")]
    Cache {
        /// Requested inventory family.
        collection: NnsInventoryCollection,
        /// Original managed-cache failure.
        #[source]
        source: HostCacheError,
    },
    /// Registry acquisition failed for the selected family.
    #[error("live NNS {collection} refresh failed: {source}")]
    Registry {
        /// Requested inventory family.
        collection: NnsInventoryCollection,
        /// Original Registry failure.
        #[source]
        source: RegistryFetchError,
    },
    /// A custom source violated the inventory contract.
    #[error("invalid NNS {collection} source data: {reason}")]
    InvalidSourceData {
        /// Requested inventory family.
        collection: NnsInventoryCollection,
        /// Original source-contract diagnostic.
        reason: String,
    },
    /// The original caller selector matched no row.
    #[error("{label} {input:?} did not match the mainnet NNS {collection} list", label = .collection.as_str().replace('-', " "))]
    NotFound {
        /// Requested inventory family.
        collection: NnsInventoryCollection,
        /// Original caller selector.
        input: String,
    },
    /// The normalized prefix matched more than one row.
    #[error("{collection} prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousPrefix {
        /// Requested inventory family.
        collection: NnsInventoryCollection,
        /// Normalized prefix.
        prefix: String,
        /// Canonically ordered matching identifiers.
        matches: Vec<String>,
    },
}

impl NnsInventoryError {
    pub(super) fn cache_refresh_reason(
        self,
        expected_path: PathBuf,
    ) -> Result<crate::cache_file::CacheRefreshReason, Self> {
        match self {
            Self::Cache { collection, source } => {
                crate::cache_file::host_cache_refresh_reason(source, &expected_path)
                    .map_err(|source| Self::Cache { collection, source })
            }
            error => Err(error),
        }
    }
}
