//! Module: ic::live
//!
//! Responsibility: shared live HTTP transport for the official IC Dashboard API.
//! Does not own: family wire conversion, report assembly, custom-source validation, or rendering.
//! Boundary: performs one read-only REST lookup for the selected Dashboard capability.

mod canister;
mod icrc_analytics;
mod metric;
mod network;
mod node_status;
#[cfg(test)]
mod tests;

use crate::{
    http_endpoint::parse_http_endpoint,
    ic::{IcHostError, MAX_IC_DASHBOARD_RESPONSE_BYTES},
    runtime::block_on_current_thread,
};
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

pub struct LiveIcSource;

fn fetch_live<T>(url: Url) -> Result<T, IcHostError>
where
    T: DeserializeOwned + Send,
{
    block_on_current_thread(fetch_json_with_limit(
        http_client()?,
        url,
        MAX_IC_DASHBOARD_RESPONSE_BYTES,
    ))?
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
    let mut response = client
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

    let declared_length = response.content_length();
    if let Some(length) = declared_length
        && length > max_response_bytes
    {
        return Err(response_too_large(&url_text, max_response_bytes, length));
    }

    let initial_capacity = declared_length
        .and_then(|length| usize::try_from(length).ok())
        .unwrap_or_default();
    let mut body = Vec::with_capacity(initial_capacity);
    while let Some(chunk) =
        response
            .chunk()
            .await
            .map_err(|error| IcHostError::HttpResponseBody {
                url: url_text.clone(),
                reason: error.to_string(),
            })?
    {
        let observed_bytes = u64::try_from(body.len())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
        if observed_bytes > max_response_bytes {
            return Err(response_too_large(
                &url_text,
                max_response_bytes,
                observed_bytes,
            ));
        }
        body.extend_from_slice(&chunk);
    }

    serde_json::from_slice(&body).map_err(|error| IcHostError::JsonDecode {
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
