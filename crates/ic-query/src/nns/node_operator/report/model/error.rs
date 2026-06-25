use crate::{ic_registry::RegistryFetchError, nns::leaf::NnsLeafHostCacheError};
use thiserror::Error as ThisError;

///
/// NnsNodeOperatorHostError
///
#[derive(Debug, ThisError)]
pub enum NnsNodeOperatorHostError {
    #[error(
        "`icq nns node-operator` supports only the mainnet `ic` network\n\nThe NNS node-operator list is derived from public Internet Computer mainnet registry records.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns node-operator list"
    )]
    UnsupportedNetwork { network: String },

    #[error(transparent)]
    Cache(#[from] NnsLeafHostCacheError),

    #[error("live NNS node-operator refresh failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),

    #[error("node operator {input:?} did not match the mainnet NNS node-operator list")]
    NodeOperatorNotFound { input: String },

    #[error("node-operator prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousNodeOperatorPrefix {
        prefix: String,
        matches: Vec<String>,
    },
}
