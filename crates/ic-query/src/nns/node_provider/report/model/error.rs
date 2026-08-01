use crate::{
    HostCacheError, ic_registry::RegistryFetchError, nns::inventory::NnsInventoryHostError,
};
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// NnsNodeProviderHostError
///
/// Errors returned by host-backed NNS node provider report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsNodeProviderHostError {
    #[error(
        "`icq nns node-provider` supports only the mainnet `ic` network\n\nThe NNS node-provider list is queried from the public Internet Computer mainnet governance canister.\nLocal replica NNS governance discovery is not supported.\n\nTry:\n  icq --network ic nns node-provider list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS node-provider refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("node provider {input:?} did not match the mainnet NNS node-provider list")]
    NodeProviderNotFound { input: String },

    #[error("node-provider prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodeProviderPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}

impl NnsInventoryHostError for NnsNodeProviderHostError {
    fn missing_cache_path(self) -> Result<PathBuf, Self> {
        match self {
            Self::Cache(HostCacheError::MissingCache { path, .. }) => Ok(path),
            error => Err(error),
        }
    }
}
