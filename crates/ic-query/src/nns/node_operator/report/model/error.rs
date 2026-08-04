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
        "`icq nns node-operator` supports only the mainnet `ic` network\n\nThe NNS node-operator list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not supported.\n\nTry:\n  icq --network ic nns node-operator list"
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
