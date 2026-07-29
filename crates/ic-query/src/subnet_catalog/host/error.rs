use crate::{
    cache_file::{CacheFileError, HostCacheError},
    ic_registry::RegistryFetchError,
    network::enforce_mainnet_network_with,
    subnet_catalog::CatalogError,
};
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

///
/// SubnetCatalogHostError
///
/// Errors returned by host-backed subnet catalog loading and refresh operations.
///

#[derive(Debug, ThisError)]
pub enum SubnetCatalogHostError {
    #[error(
        "`icq nns subnet` supports only the mainnet `ic` network\n\nThe cached NNS subnet data describes the public Internet Computer mainnet.\nLocal replica subnet discovery is not supported.\n\nTry:\n  icq --network ic nns subnet list"
    )]
    UnsupportedNetwork { network: String },

    #[error(
        "subnet catalog cache is missing at {}\n\nRun `icq nns subnet refresh` to fetch the public Internet Computer mainnet catalog, or populate this path with a valid subnet catalog JSON.",
        path.display()
    )]
    MissingCatalog { path: PathBuf },

    #[error("failed to read subnet catalog at {}: {source}", path.display())]
    ReadCatalog { path: PathBuf, source: io::Error },

    #[error(
        "cached subnet catalog network mismatch: path is for {requested}, catalog is for {actual}"
    )]
    NetworkMismatch { requested: String, actual: String },

    #[error(
        "invalid stale duration {value:?}; use positive seconds or a value ending in s, m, h, or d"
    )]
    #[cfg(test)]
    InvalidStaleDuration { value: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS registry refresh failed: {0}")]
    RegistryRefresh(#[from] RegistryFetchError),

    #[error("refreshed subnet catalog network mismatch: requested {requested}, fetched {actual}")]
    RefreshNetworkMismatch { requested: String, actual: String },

    #[error(transparent)]
    Catalog(#[from] CatalogError),
}

pub(super) fn enforce_mainnet_network(network: &str) -> Result<(), SubnetCatalogHostError> {
    enforce_mainnet_network_with(network, |network| {
        SubnetCatalogHostError::UnsupportedNetwork { network }
    })
}

pub(super) fn subnet_cache_error(err: CacheFileError) -> SubnetCatalogHostError {
    HostCacheError::operation("subnet catalog", err).into()
}
