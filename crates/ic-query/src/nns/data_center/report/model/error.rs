use crate::{
    HostCacheError, ic_registry::RegistryFetchError, nns::inventory::NnsInventoryHostError,
};
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// NnsDataCenterHostError
///
/// Errors returned by host-backed NNS data center report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsDataCenterHostError {
    #[error(
        "`icq nns data-center` supports only the mainnet `ic` network\n\nThe NNS data-center list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not supported.\n\nTry:\n  icq --network ic nns data-center list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS data-center refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("data center {input:?} did not match the mainnet NNS data-center list")]
    DataCenterNotFound { input: String },

    #[error("data-center prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousDataCenterPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}

impl NnsInventoryHostError for NnsDataCenterHostError {
    fn missing_cache_path(self) -> Result<PathBuf, Self> {
        match self {
            Self::Cache(HostCacheError::MissingCache { path, .. }) => Ok(path),
            error => Err(error),
        }
    }
}
