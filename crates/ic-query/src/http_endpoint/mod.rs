//! Module: http_endpoint
//!
//! Responsibility: parse and validate HTTP(S) endpoints used by live host adapters.
//! Does not own: source-specific URL paths, query policy, errors, or network calls.
//! Boundary: returns one clean base URL after shared authority validation.

use url::Url;

/// Parse a credential-free HTTP(S) base endpoint for a live source adapter.
pub fn parse_http_endpoint(endpoint: &str) -> Result<Url, String> {
    if endpoint
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err("endpoint URL must not include whitespace or control characters".to_string());
    }
    let parsed = Url::parse(endpoint).map_err(|error| error.to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(format!(
            "unsupported URL scheme {:?}; expected http or https",
            parsed.scheme()
        ));
    }
    if parsed.host_str().is_none() {
        return Err("endpoint URL must include a host".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("endpoint URL must not include user information".to_string());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() {
        return Err("endpoint URL must not include a query or fragment".to_string());
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests;
