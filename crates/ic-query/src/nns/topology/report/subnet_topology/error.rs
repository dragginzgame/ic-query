use crate::{
    HostCacheError,
    ic_registry::RegistryFetchError,
    nns::{
        inventory_source::enforce_mainnet_network_with,
        topology::report::subnet_topology::NnsSubnetTopologyValidationError,
    },
};
use thiserror::Error as ThisError;

///
/// NnsSubnetTopologyHostError
///
/// Live-source, relation-validation, cache, and refresh-lock failures.
///

#[derive(Debug, ThisError)]
pub enum NnsSubnetTopologyHostError {
    /// A caller requested a network other than mainnet.
    #[error("NNS Subnet topology supports only the mainnet `ic` network; requested {network}")]
    UnsupportedNetwork {
        /// Unsupported network name.
        network: String,
    },

    /// Loading or writing the joined topology cache failed.
    #[error(transparent)]
    Cache(#[from] HostCacheError),

    /// A source returned a report for a different network.
    #[error("refreshed Subnet topology network mismatch: requested {requested}, fetched {actual}")]
    RefreshNetworkMismatch {
        /// Network requested by the caller.
        requested: String,
        /// Network recorded in the refreshed report.
        actual: String,
    },

    /// A source returned data attributed to an unexpected Registry canister.
    #[error(
        "refreshed Subnet topology Registry canister mismatch: expected {expected}, fetched {actual}"
    )]
    RegistryCanisterMismatch {
        /// Expected mainnet Registry principal.
        expected: String,
        /// Registry principal recorded in the refreshed report.
        actual: String,
    },

    /// A source returned data attributed to a different endpoint.
    #[error(
        "refreshed Subnet topology source endpoint mismatch: requested {requested}, fetched {actual}"
    )]
    SourceEndpointMismatch {
        /// Endpoint requested by the caller.
        requested: String,
        /// Endpoint recorded in the refreshed report.
        actual: String,
    },

    /// Exact-version Registry collection or relation projection failed.
    #[error(transparent)]
    Registry(#[from] RegistryFetchError),

    /// Refreshed or cached report invariants failed validation.
    #[error(transparent)]
    Validation(#[from] NnsSubnetTopologyValidationError),
}

pub(super) fn enforce_mainnet_network(network: &str) -> Result<(), NnsSubnetTopologyHostError> {
    enforce_mainnet_network_with(network, |network| {
        NnsSubnetTopologyHostError::UnsupportedNetwork { network }
    })
}
