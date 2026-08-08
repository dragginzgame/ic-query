//! Module: ic::live::cloud_engine_provider
//!
//! Responsibility: official Dashboard CloudEngine provider URL and wire handling.
//! Does not own: report filtering, source validation, native CloudEngine queries, or rendering.
//! Boundary: performs one complete provider-resource request or one exact provider request.

use super::{LiveIcSource, append_path_segments, dashboard_base_url, fetch_live};
use crate::{
    cloud_engine::{
        CloudEngineProviderInfoSourceData, CloudEngineProviderListSourceData,
        CloudEngineProviderRow, CloudEngineProviderSource,
    },
    ic::{IcHostError, IcSourceRequest},
};
use candid::Principal;
use serde::Deserialize as SerdeDeserialize;
use url::Url;

impl CloudEngineProviderSource for LiveIcSource {
    fn fetch_cloud_engine_provider_list(
        &self,
        request: &IcSourceRequest,
    ) -> Result<CloudEngineProviderListSourceData, IcHostError> {
        let url = provider_list_url(&request.endpoint)?;
        let wire: DashboardNodeProviderList = fetch_live(url)?;
        Ok(CloudEngineProviderListSourceData {
            source: request.clone(),
            providers: wire.node_providers,
        })
    }

    fn fetch_cloud_engine_provider_info(
        &self,
        request: &IcSourceRequest,
        node_provider_id: &str,
    ) -> Result<CloudEngineProviderInfoSourceData, IcHostError> {
        let node_provider_id = canonical_provider_id(node_provider_id)?;
        let url = provider_info_url(&request.endpoint, &node_provider_id)?;
        let provider = fetch_live(url)?;
        Ok(CloudEngineProviderInfoSourceData {
            source: request.clone(),
            provider,
        })
    }
}

fn provider_list_url(endpoint: &str) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["node-providers"])?;
    Ok(url)
}

fn provider_info_url(endpoint: &str, node_provider_id: &str) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["node-providers", node_provider_id])?;
    Ok(url)
}

fn canonical_provider_id(value: &str) -> Result<String, IcHostError> {
    Principal::from_text(value)
        .map(|principal| principal.to_text())
        .map_err(|error| IcHostError::InvalidPrincipal {
            field: "node_provider_id",
            reason: error.to_string(),
        })
}

#[derive(SerdeDeserialize)]
struct DashboardNodeProviderList {
    node_providers: Vec<CloudEngineProviderRow>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROVIDER_ID: &str = "rbn2y-6vfsb-gv35j-4cyvy-pzbdu-e5aum-jzjg6-5b4n5-vuguf-ycubq-zae";

    #[test]
    fn provider_urls_select_complete_and_exact_resources() {
        assert_eq!(
            provider_list_url("https://ic-api.internetcomputer.org/api/v3/")
                .expect("provider list URL")
                .as_str(),
            "https://ic-api.internetcomputer.org/api/v3/node-providers"
        );
        assert_eq!(
            provider_info_url("https://ic-api.internetcomputer.org/api/v3", PROVIDER_ID)
                .expect("provider info URL")
                .as_str(),
            format!("https://ic-api.internetcomputer.org/api/v3/node-providers/{PROVIDER_ID}")
        );
    }

    #[test]
    fn provider_wire_preserves_cloud_engine_and_general_fields() {
        let wire: DashboardNodeProviderList = serde_json::from_str(&format!(
            r#"{{"node_providers":[{{
                "principal_id":"{PROVIDER_ID}",
                "display_name":"Provider",
                "website":"example.com",
                "logo_url":null,
                "location_count":1,
                "locations":[{{"dc_key":"dc1","display_name":"One","latitude":1.5,"longitude":2.5,"owner":"Owner","region":"Region"}}],
                "cloud_engine_location_count":1,
                "cloud_engine_locations":[{{"dc_key":"dc1","display_name":"One","latitude":1.5,"longitude":2.5,"owner":"Owner","region":"Region"}}],
                "total_cloud_engine_nodes":5,
                "total_cloud_engine_unassigned_nodes":4,
                "total_cloud_engines":1,
                "total_node_allowance":7,
                "total_nodes":8,
                "total_rewardable_nodes":6,
                "total_subnets":2,
                "total_unassigned_nodes":3,
                "additive":"ignored"
            }}]}}"#
        ))
        .expect("provider wire");

        let provider = &wire.node_providers[0];
        assert_eq!(provider.total_cloud_engine_nodes, 5);
        assert_eq!(provider.total_nodes, 8);
        assert_eq!(provider.cloud_engine_locations[0].dc_key, "dc1");
    }
}
