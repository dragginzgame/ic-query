use crate::{ic_registry::RegistryFetchError, nns::leaf::NnsLeafHostCacheError};
use thiserror::Error as ThisError;

///
/// NnsDataCenterHostError
///
#[derive(Debug, ThisError)]
pub enum NnsDataCenterHostError {
    #[error(
        "`icq nns data-center` supports only the mainnet `ic` network\n\nThe NNS data-center list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns data-center list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] NnsLeafHostCacheError),

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
