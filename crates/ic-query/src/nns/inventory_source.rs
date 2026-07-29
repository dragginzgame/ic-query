//! Module: nns::inventory_source
//!
//! Responsibility: define shared source settings for mainnet Registry inventories.
//! Does not own: report projection, source traits, or cache policy.
//! Boundary: enforces the network contract before constructing live fetch requests.

use crate::{ic_registry::MainnetRegistryFetchRequest, subnet_catalog::MAINNET_NETWORK};

///
/// NnsInventorySourceRequest
///
/// Source settings shared by NNS Registry inventory and topology adapters.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsInventorySourceRequest {
    /// Network to collect.
    pub network: String,
    /// Replica endpoint used for Registry queries.
    pub endpoint: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Collector identity recorded in the report.
    pub fetched_by: String,
}

impl NnsInventorySourceRequest {
    #[must_use]
    pub fn new(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at: impl Into<String>,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self {
            network: network.into(),
            endpoint: endpoint.into(),
            fetched_at: fetched_at.into(),
            fetched_by: fetched_by.into(),
        }
    }
}

pub(in crate::nns) fn mainnet_registry_fetch_request<Error>(
    request: &NnsInventorySourceRequest,
    unsupported_network: impl FnOnce(String) -> Error,
) -> Result<MainnetRegistryFetchRequest, Error> {
    enforce_mainnet_network_with(&request.network, unsupported_network)?;
    let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
    fetch_request.endpoint.clone_from(&request.endpoint);
    fetch_request.fetched_by.clone_from(&request.fetched_by);
    Ok(fetch_request)
}

pub(in crate::nns) fn enforce_mainnet_network_with<Error>(
    network: &str,
    unsupported_network: impl FnOnce(String) -> Error,
) -> Result<(), Error> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(unsupported_network(network.to_string()))
}
