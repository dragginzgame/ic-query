use crate::ic_registry::RegistryFetchError;
use thiserror::Error as ThisError;

///
/// NnsRegistryHostError
///
#[derive(Debug, ThisError)]
pub enum NnsRegistryHostError {
    #[error(
        "`icq nns registry` supports only the mainnet `ic` network\n\nThe NNS registry inspected by this command is the public Internet Computer mainnet registry canister.\nLocal replica NNS registry discovery is not implemented yet.\n\nTry:\n  icq --network ic nns registry version"
    )]
    UnsupportedNetwork { network: String },

    #[error("live NNS registry query failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),
}
