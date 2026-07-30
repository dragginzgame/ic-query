//! Module: ic::live
//!
//! Responsibility: live HTTP transport and wire conversion for the official IC Dashboard API.
//! Does not own: report assembly, custom-source validation, command parsing, or rendering.
//! Boundary: performs one read-only REST lookup and retains request provenance.

use crate::ic::{
    IcCanisterSource, IcCanisterSourceData, IcCanisterUpgrade, IcHostError, IcSourceRequest,
};
use crate::runtime::block_on_current_thread;
use reqwest::Client;
use serde::Deserialize as SerdeDeserialize;
use std::time::Duration;
use url::Url;

const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

///
/// LiveIcSource
///
/// Live official IC Dashboard source used by report builders outside tests.
///

pub struct LiveIcSource;

impl IcCanisterSource for LiveIcSource {
    fn fetch_canister(
        &self,
        request: &IcSourceRequest,
        canister_id: &str,
    ) -> Result<IcCanisterSourceData, IcHostError> {
        let canister_id = super::source::canonical_canister_id(canister_id)?;
        let url = canister_url(&request.endpoint, &canister_id)?;
        let url_text = url.to_string();
        let client = Client::builder()
            .user_agent(concat!("ic-query/", env!("CARGO_PKG_VERSION")))
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|error| IcHostError::HttpClientBuild {
                reason: error.to_string(),
            })?;
        block_on_current_thread(fetch_canister(client, url, url_text, request))?
    }
}

async fn fetch_canister(
    client: Client,
    url: Url,
    url_text: String,
    request: &IcSourceRequest,
) -> Result<IcCanisterSourceData, IcHostError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| IcHostError::HttpRequest {
            url: url_text.clone(),
            reason: error.to_string(),
        })?;
    let status = response.status();
    if !status.is_success() {
        return Err(IcHostError::HttpStatus {
            url: url_text,
            status: status.as_u16(),
        });
    }
    let wire = response
        .json::<DashboardCanister>()
        .await
        .map_err(|error| IcHostError::JsonDecode {
            url: url_text,
            reason: error.to_string(),
        })?;
    Ok(wire.into_source_data(request))
}

fn canister_url(endpoint: &str, canister_id: &str) -> Result<Url, IcHostError> {
    let mut url = Url::parse(endpoint).map_err(|error| invalid_endpoint(endpoint, error))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(IcHostError::InvalidEndpoint {
            endpoint: endpoint.to_string(),
            reason: format!(
                "unsupported URL scheme {:?}; expected http or https",
                url.scheme()
            ),
        });
    }
    if url.host_str().is_none() {
        return Err(IcHostError::InvalidEndpoint {
            endpoint: endpoint.to_string(),
            reason: "endpoint URL must include a host".to_string(),
        });
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(IcHostError::InvalidEndpoint {
            endpoint: endpoint.to_string(),
            reason: "base endpoint must not include a query or fragment".to_string(),
        });
    }

    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| IcHostError::InvalidEndpoint {
                endpoint: endpoint.to_string(),
                reason: "base endpoint cannot accept path segments".to_string(),
            })?;
        segments.pop_if_empty();
        segments.push("canisters");
        segments.push(canister_id);
    }
    Ok(url)
}

fn invalid_endpoint(endpoint: &str, error: url::ParseError) -> IcHostError {
    IcHostError::InvalidEndpoint {
        endpoint: endpoint.to_string(),
        reason: error.to_string(),
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanister {
    canister_id: String,
    canister_type: Option<String>,
    controllers: Vec<String>,
    id: u64,
    language: String,
    module_hash: String,
    name: String,
    subnet_id: String,
    updated_at: String,
    upgrades: Option<Vec<DashboardCanisterUpgrade>>,
}

impl DashboardCanister {
    fn into_source_data(self, request: &IcSourceRequest) -> IcCanisterSourceData {
        IcCanisterSourceData {
            source_endpoint: request.endpoint.clone(),
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            canister_id: self.canister_id,
            dashboard_id: self.id,
            canister_type: self.canister_type,
            name: self.name,
            subnet_id: self.subnet_id,
            controllers: self.controllers,
            language: self.language,
            module_hash: self.module_hash,
            dashboard_updated_at: self.updated_at,
            upgrades: self.upgrades.map(|upgrades| {
                upgrades
                    .into_iter()
                    .map(DashboardCanisterUpgrade::into_public)
                    .collect()
            }),
        }
    }
}

#[derive(SerdeDeserialize)]
struct DashboardCanisterUpgrade {
    executed_timestamp_seconds: u64,
    module_hash: String,
    proposal_id: u64,
}

impl DashboardCanisterUpgrade {
    fn into_public(self) -> IcCanisterUpgrade {
        IcCanisterUpgrade {
            executed_timestamp_seconds: self.executed_timestamp_seconds,
            module_hash: self.module_hash,
            proposal_id: self.proposal_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canister_url_appends_path_to_endpoints_with_or_without_trailing_slash() {
        for endpoint in [
            "https://ic-api.internetcomputer.org/api/v3",
            "https://ic-api.internetcomputer.org/api/v3/",
        ] {
            assert_eq!(
                canister_url(endpoint, "ryjl3-tyaaa-aaaaa-aaaba-cai")
                    .expect("canister URL")
                    .as_str(),
                "https://ic-api.internetcomputer.org/api/v3/canisters/ryjl3-tyaaa-aaaaa-aaaba-cai"
            );
        }
    }

    #[test]
    fn wire_decoder_preserves_null_upgrade_history_and_ignores_additive_fields() {
        let wire: DashboardCanister = serde_json::from_str(
            r#"{
                "canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
                "canister_type": null,
                "controllers": [],
                "id": 1,
                "language": "",
                "module_hash": "",
                "name": "",
                "subnet_id": "tdb26-jop6k-aogll-7ltgs-eruif-6kk7m-qpktf-gdiqx-mxtrf-vb5e6-eqe",
                "updated_at": "2026-07-30T17:47:41.745647",
                "upgrades": null,
                "future_field": true
            }"#,
        )
        .expect("current Dashboard payload");

        assert_eq!(wire.canister_type, None);
        assert!(wire.upgrades.is_none());
    }
}
