use crate::{HostCacheError, ic_registry::RegistryFetchError};
use thiserror::Error as ThisError;

///
/// NnsNodeHostError
///
/// Errors returned by host-backed NNS node report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsNodeHostError {
    #[error(
        "`icq nns node` supports only the mainnet `ic` network\n\nThe NNS node list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not supported.\n\nTry:\n  icq --network ic nns node list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS node refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    /// A custom source returned evidence that violates the node inventory contract.
    #[error("invalid NNS node source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract failure.
        reason: String,
    },

    #[error("node {input:?} did not match the mainnet NNS node list")]
    NodeNotFound { input: String },

    #[error("node prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodePrefix {
        prefix: String,
        matches: Vec<String>,
    },
}

impl_nns_inventory_host_error!(NnsNodeHostError, "node");
