//! Module: nns::source
//!
//! Responsibility: identify the built-in live NNS source adapter.
//! Does not own: source capability traits, report projection, or cache policy.
//! Boundary: one adapter implements the capability traits owned by each NNS report family.

use crate::report::CollectionSourceRequest;
use crate::{ic_registry::MainnetRegistryFetchRequest, network::enforce_mainnet_network_with};

///
/// LiveNnsSource
///
/// Built-in live adapter for supported NNS Registry and governance report capabilities.
///

#[derive(Clone, Debug, Default)]
pub struct LiveNnsSource {
    #[cfg(feature = "nns-host")]
    pub(crate) agent: ic_host_tools::agent::EndpointAgent,
}

pub fn mainnet_registry_fetch_request<Error>(
    request: &CollectionSourceRequest,
    unsupported_network: impl FnOnce(String) -> Error,
) -> Result<MainnetRegistryFetchRequest, Error> {
    enforce_mainnet_network_with(&request.network, unsupported_network)?;
    let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
    fetch_request.endpoint.clone_from(&request.endpoint);
    fetch_request.fetched_by.clone_from(&request.fetched_by);
    Ok(fetch_request)
}
