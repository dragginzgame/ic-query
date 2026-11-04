//! Module: ic::live::cloud_engine_provider
//!
//! Responsibility: official Dashboard CloudEngine provider URL and wire handling.
//! Does not own: report filtering, source validation, native CloudEngine queries, or rendering.
//! Boundary: performs one complete provider-resource request or one exact provider request.

use super::{LiveIcSource, dashboard_resource_url};
use crate::{
    cloud_engine::{
        CloudEngineProviderInfoSourceData, CloudEngineProviderListSourceData,
        CloudEngineProviderRow, CloudEngineProviderSource,
    },
    ic::{IcHostError, IcSourceRequest, canonical_request_principal},
};
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl CloudEngineProviderSource for LiveIcSource {
    fn fetch_cloud_engine_provider_list<'a>(
        &'a self,
        request: &'a IcSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<CloudEngineProviderListSourceData, IcHostError>>
    {
        Box::pin(async move {
            let url = provider_list_url(&request.endpoint)?;
            let wire: DashboardNodeProviderList = self.fetch_live(url).await?;
            Ok(CloudEngineProviderListSourceData {
                source: request.clone(),
                providers: wire.node_providers,
            })
        })
    }

    fn fetch_cloud_engine_provider_info<'a>(
        &'a self,
        request: &'a IcSourceRequest,
        node_provider_id: &'a str,
    ) -> crate::report::SourceFuture<'a, Result<CloudEngineProviderInfoSourceData, IcHostError>>
    {
        Box::pin(async move {
            let node_provider_id =
                canonical_request_principal("node_provider_id", node_provider_id)?;
            let url = provider_info_url(&request.endpoint, &node_provider_id)?;
            let provider = self.fetch_live(url).await?;
            Ok(CloudEngineProviderInfoSourceData {
                source: request.clone(),
                provider,
            })
        })
    }
}

fn provider_list_url(endpoint: &str) -> Result<Url, IcHostError> {
    dashboard_resource_url(endpoint, &["node-providers"])
}

fn provider_info_url(endpoint: &str, node_provider_id: &str) -> Result<Url, IcHostError> {
    dashboard_resource_url(endpoint, &["node-providers", node_provider_id])
}

#[derive(SerdeDeserialize)]
struct DashboardNodeProviderList {
    node_providers: Vec<CloudEngineProviderRow>,
}

#[cfg(test)]
mod tests;
