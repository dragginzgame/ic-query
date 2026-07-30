//! Module: nns::source
//!
//! Responsibility: identify the built-in live NNS source adapter.
//! Does not own: source capability traits, report projection, or cache policy.
//! Boundary: one adapter implements the capability traits owned by each NNS report family.

use crate::{
    ic_registry::MainnetRegistryFetchRequest, network::enforce_mainnet_network_with,
    subnet_catalog::format_utc_timestamp_secs,
};

///
/// LiveNnsSource
///
/// Built-in live adapter for supported NNS Registry and governance report capabilities.
///

#[derive(Clone, Copy, Debug, Default)]
pub struct LiveNnsSource;

///
/// NnsSourceRequest
///
/// Network and collection provenance shared by NNS source capabilities.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsSourceRequest {
    /// Network to collect.
    pub network: String,
    /// Replica endpoint used for the query.
    pub endpoint: String,
    /// UTC collection timestamp recorded in the report.
    pub fetched_at: String,
    /// Collector identity recorded in the report.
    pub fetched_by: String,
}

impl NnsSourceRequest {
    /// Create source settings for one NNS collection.
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

    /// Create source settings from a Unix collection timestamp.
    #[must_use]
    pub fn from_unix_secs(
        network: impl Into<String>,
        endpoint: impl Into<String>,
        fetched_at_unix_secs: u64,
        fetched_by: impl Into<String>,
    ) -> Self {
        Self::new(
            network,
            endpoint,
            format_utc_timestamp_secs(fetched_at_unix_secs),
            fetched_by,
        )
    }
}

pub fn mainnet_registry_fetch_request<Error>(
    request: &NnsSourceRequest,
    unsupported_network: impl FnOnce(String) -> Error,
) -> Result<MainnetRegistryFetchRequest, Error> {
    enforce_mainnet_network_with(&request.network, unsupported_network)?;
    let mut fetch_request = MainnetRegistryFetchRequest::new(request.fetched_at.clone());
    fetch_request.endpoint.clone_from(&request.endpoint);
    fetch_request.fetched_by.clone_from(&request.fetched_by);
    Ok(fetch_request)
}
