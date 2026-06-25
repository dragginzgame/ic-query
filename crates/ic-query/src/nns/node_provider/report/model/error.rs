use crate::{ic_registry::RegistryFetchError, nns::leaf::NnsLeafHostCacheError};
use thiserror::Error as ThisError;

///
/// NnsNodeProviderHostError
///
#[derive(Debug, ThisError)]
pub enum NnsNodeProviderHostError {
    #[error(
        "`icq nns node-provider` supports only the mainnet `ic` network\n\nThe NNS node-provider list is queried from the public Internet Computer mainnet governance canister.\nLocal replica NNS governance discovery is not implemented yet.\n\nTry:\n  icq --network ic nns node-provider list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] NnsLeafHostCacheError),

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
