use crate::{ic_registry::RegistryFetchError, nns::leaf::NnsLeafHostCacheError};
use thiserror::Error as ThisError;

///
/// NnsNodeHostError
///
#[derive(Debug, ThisError)]
pub enum NnsNodeHostError {
    #[error(
        "`icq nns node` supports only the mainnet `ic` network\n\nThe NNS node list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns node list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] NnsLeafHostCacheError),

    #[error("live NNS node refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("node {input:?} did not match the mainnet NNS node list")]
    NodeNotFound { input: String },

    #[error("node prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodePrefix {
        prefix: String,
        matches: Vec<String>,
    },
}
