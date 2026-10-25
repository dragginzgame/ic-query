//! Module: ic::live
//!
//! Responsibility: shared live HTTP transport for the official IC Dashboard API.
//! Does not own: family wire conversion, report assembly, custom-source validation, or rendering.
//! Boundary: performs one read-only REST lookup for the selected Dashboard capability.

mod canister;
mod cloud_engine_node;
mod cloud_engine_provider;
mod icrc_analytics;
mod icrc_index;
mod metric;
mod network;
mod node_provider_reward;
mod node_status;
mod replica_version;
#[cfg(test)]
mod tests;

use crate::{
    http_endpoint::parse_http_endpoint,
    ic::{IcHostError, MAX_IC_DASHBOARD_RESPONSE_BYTES},
};
use ic_host_http::{ReadFailure, ReadOptions, read_body};
use reqwest::{Client, redirect::Policy as RedirectPolicy};
use serde::de::DeserializeOwned;
use std::time::Duration;
use url::Url;

const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

///
/// LiveIcSource
///
/// Live official IC Dashboard source used by report builders outside tests.
///

#[derive(Default)]
pub struct LiveIcSource {
    client: std::sync::OnceLock<Client>,
}

impl LiveIcSource {
    async fn fetch_live<T>(&self, url: Url) -> Result<T, IcHostError>
    where
        T: DeserializeOwned + Send,
    {
        if self.client.get().is_none() {
            let _ = self.client.set(http_client()?);
        }
        fetch_json_with_limit(
            self.client.get().expect("HTTP client initialized").clone(),
            url,
            MAX_IC_DASHBOARD_RESPONSE_BYTES,
        )
        .await
    }
}

fn http_client() -> Result<Client, IcHostError> {
    Client::builder()
        .user_agent(concat!("ic-query/", env!("CARGO_PKG_VERSION")))
        .timeout(HTTP_TIMEOUT)
        .redirect(RedirectPolicy::none())
        .build()
        .map_err(|error| IcHostError::HttpClientBuild {
            reason: error.to_string(),
        })
}

async fn fetch_json_with_limit<T>(
    client: Client,
    url: Url,
    max_response_bytes: u64,
) -> Result<T, IcHostError>
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

    let max_bytes =
        usize::try_from(max_response_bytes).map_err(|error| IcHostError::HttpResponseBody {
            url: url_text.clone(),
            reason: format!("response budget cannot be represented: {error}"),
        })?;
    let body = read_body(
        response,
        ReadOptions {
            max_bytes,
            reject_declared_oversize: true,
            retain_partial_on_error: false,
        },
    )
    .await
    .map_err(|error| match error.failure {
        ReadFailure::DeclaredLength { declared, .. } => {
            response_too_large(&url_text, max_response_bytes, declared)
        }
        ReadFailure::Limit { .. } => {
            response_too_large(&url_text, max_response_bytes, error.received_bytes)
        }
        ReadFailure::Transport(source) => IcHostError::HttpResponseBody {
            url: url_text.clone(),
            reason: source.to_string(),
        },
        ReadFailure::Allocation(source) => IcHostError::HttpResponseBody {
            url: url_text.clone(),
            reason: source.to_string(),
        },
    })?;

    serde_json::from_slice(&body.bytes).map_err(|error| IcHostError::JsonDecode {
        url: url_text,
        reason: error.to_string(),
    })
}

fn response_too_large(url: &str, max_bytes: u64, observed_bytes: u64) -> IcHostError {
    IcHostError::HttpResponseTooLarge {
        url: url.to_string(),
        max_bytes,
        observed_bytes,
    }
}

fn ledger_resource_url(
    endpoint: &str,
    ledger_canister_id: &str,
    resource_path: &[&str],
) -> Result<Url, IcHostError> {
    let mut url = dashboard_base_url(endpoint)?;
    append_path_segments(endpoint, &mut url, &["ledgers", ledger_canister_id])?;
    append_path_segments(endpoint, &mut url, resource_path)?;
    Ok(url)
}

fn dashboard_base_url(endpoint: &str) -> Result<Url, IcHostError> {
    parse_http_endpoint(endpoint).map_err(|reason| invalid_endpoint(endpoint, reason))
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

fn append_time_window(
    pairs: &mut url::form_urlencoded::Serializer<'_, url::UrlQuery<'_>>,
    start: u64,
    end: u64,
) {
    pairs.append_pair("start", &start.to_string());
    pairs.append_pair("end", &end.to_string());
}
