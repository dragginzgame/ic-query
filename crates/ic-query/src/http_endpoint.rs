//! Module: http_endpoint
//!
//! Responsibility: parse and validate HTTP(S) endpoints used by live host adapters.
//! Does not own: source-specific URL paths, query policy, errors, or network calls.
//! Boundary: returns one clean base URL after shared authority validation.

use url::Url;

/// Parse a credential-free HTTP(S) base endpoint for a live source adapter.
pub fn parse_http_endpoint(endpoint: &str) -> Result<Url, String> {
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
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_endpoints() {
        for (endpoint, parsed) in [
            ("http://localhost:8080", "http://localhost:8080/"),
            ("https://icp-api.io/api/v2", "https://icp-api.io/api/v2"),
        ] {
            assert_eq!(
                parse_http_endpoint(endpoint)
                    .expect("valid HTTP endpoint")
                    .as_str(),
                parsed
            );
        }
    }

    #[test]
    fn rejects_malformed_and_non_http_endpoints() {
        for (endpoint, expected_reason) in [
            (":::", "relative URL without a base"),
            ("ftp://example.com", "expected http or https"),
            (
                "https://reader:secret@example.com/api",
                "must not include user information",
            ),
            (
                "https://example.com/api?limit=1",
                "must not include a query or fragment",
            ),
            (
                "https://example.com/api#section",
                "must not include a query or fragment",
            ),
        ] {
            let reason = parse_http_endpoint(endpoint).expect_err("endpoint must be rejected");
            assert!(
                reason.contains(expected_reason),
                "expected {reason:?} to contain {expected_reason:?}"
            );
        }
    }
}
