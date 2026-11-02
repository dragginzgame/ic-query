use crate::{HostCacheError, ic_registry::RegistryFetchError};
use thiserror::Error as ThisError;

///
/// NnsNodeOperatorHostError
///
/// Errors returned by host-backed NNS node operator report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsNodeOperatorHostError {
    #[error(
        "NNS node-operator reports support only the mainnet `ic` network (requested {network:?})"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS node-operator refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    /// A custom source returned evidence that violates the node-operator inventory contract.
    #[error("invalid NNS node-operator source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract failure.
        reason: String,
    },

    #[error("node operator {input:?} did not match the mainnet NNS node-operator list")]
    NodeOperatorNotFound { input: String },

    #[error("node-operator prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodeOperatorPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}

impl_nns_inventory_host_error!(NnsNodeOperatorHostError, "node-operator");
