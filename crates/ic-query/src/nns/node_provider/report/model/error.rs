use crate::{HostCacheError, ic_registry::RegistryFetchError};
use thiserror::Error as ThisError;

///
/// NnsNodeProviderHostError
///
/// Errors returned by host-backed NNS node provider report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsNodeProviderHostError {
    #[error(
        "NNS node-provider reports support only the mainnet `ic` network (requested {network:?})"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS node-provider refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    /// A custom source returned evidence that violates the node-provider inventory contract.
    #[error("invalid NNS node-provider source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract failure.
        reason: String,
    },

    #[error("node provider {input:?} did not match the mainnet NNS node-provider list")]
    NodeProviderNotFound { input: String },

    #[error("node-provider prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodeProviderPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}

impl_nns_inventory_host_error!(NnsNodeProviderHostError, "node-provider");
