//! Module: ic::live
//!
//! Responsibility: shared live HTTP transport for the official IC Dashboard API.
//! Does not own: family wire conversion, report assembly, custom-source validation, or rendering.
//! Boundary: performs one read-only REST lookup for the selected Dashboard capability.

mod canister;
mod icrc_analytics;
mod metric;
mod network;

use crate::{
    http_endpoint::parse_http_endpoint, ic::IcHostError, runtime::block_on_current_thread,
};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;
use url::Url;

const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

///
/// LiveIcSource
///
/// Live official IC Dashboard source used by report builders outside tests.
///

pub struct LiveIcSource;

fn fetch_live<T>(url: Url) -> Result<T, IcHostError>
where
    T: DeserializeOwned + Send,
{
    block_on_current_thread(fetch_json(http_client()?, url))?
}

fn http_client() -> Result<Client, IcHostError> {
    Client::builder()
        .user_agent(concat!("ic-query/", env!("CARGO_PKG_VERSION")))
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|error| IcHostError::HttpClientBuild {
            reason: error.to_string(),
        })
}

async fn fetch_json<T>(client: Client, url: Url) -> Result<T, IcHostError>
where
    T: DeserializeOwned,
{
    let url_text = url.to_string();
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
    response
        .json::<T>()
        .await
        .map_err(|error| IcHostError::JsonDecode {
            url: url_text,
            reason: error.to_string(),
        })
}

fn dashboard_base_url(endpoint: &str) -> Result<Url, IcHostError> {
    let url = parse_http_endpoint(endpoint).map_err(|reason| invalid_endpoint(endpoint, reason))?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(invalid_endpoint(
            endpoint,
            "base endpoint must not include a query or fragment",
        ));
    }

    Ok(url)
}

fn append_path_segments(endpoint: &str, url: &mut Url, path: &[&str]) -> Result<(), IcHostError> {
    let mut segments = url
        .path_segments_mut()
        .map_err(|()| invalid_endpoint(endpoint, "base endpoint cannot accept path segments"))?;
    segments.pop_if_empty();
    segments.extend(path.iter().copied());
    Ok(())
}

fn invalid_endpoint(endpoint: &str, reason: impl Into<String>) -> IcHostError {
    IcHostError::InvalidEndpoint {
        endpoint: endpoint.to_string(),
        reason: reason.into(),
    }
}
