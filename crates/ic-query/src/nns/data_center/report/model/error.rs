use crate::{HostCacheError, ic_registry::RegistryFetchError};
use thiserror::Error as ThisError;

///
/// NnsDataCenterHostError
///
/// Errors returned by host-backed NNS data center report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsDataCenterHostError {
    #[error(
        "NNS data-center reports support only the mainnet `ic` network (requested {network:?})"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] HostCacheError),

    #[error("live NNS data-center refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    /// A custom source returned evidence that violates the data-center inventory contract.
    #[error("invalid NNS data-center source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-contract failure.
        reason: String,
    },

    #[error("data center {input:?} did not match the mainnet NNS data-center list")]
    DataCenterNotFound { input: String },

    #[error("data-center prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousDataCenterPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}

impl_nns_inventory_host_error!(NnsDataCenterHostError, "data-center");
