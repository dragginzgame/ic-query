use crate::ic_registry::RegistryFetchError;
use thiserror::Error as ThisError;

///
/// NnsRegistryHostError
///
/// Errors returned by host-backed NNS registry version report operations.
///

#[derive(Debug, ThisError)]
pub enum NnsRegistryHostError {
    #[error(
        "`icq nns registry` supports only the mainnet `ic` network\n\nThe NNS registry inspected by this command is the public Internet Computer mainnet registry canister.\nLocal replica NNS registry discovery is not supported.\n\nTry:\n  icq --network ic nns registry version"
    )]
    UnsupportedNetwork { network: String },

    /// A custom Registry source returned provenance for a different request.
    #[error("NNS Registry source {field} mismatch: expected {expected:?}, got {actual:?}")]
    SourceMismatch {
        /// Provenance field that did not match.
        field: &'static str,
        /// Request-derived value.
        expected: String,
        /// Source-returned value.
        actual: String,
    },

    /// A custom source returned malformed or internally inconsistent evidence.
    #[error("invalid NNS Registry source data: {reason}")]
    InvalidSourceData {
        /// Deterministic source-data validation failure.
        reason: String,
    },

    #[error("live NNS registry query failed: {0}")]
    NnsQuery(#[from] RegistryFetchError),
}
