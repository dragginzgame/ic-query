use super::*;

#[test]
fn rejects_invalid_authorities() {
    for endpoint in [
        "https://:443",
        "https://example.com:not-a-port",
        "https://[invalid]",
        "https://example.com:65536",
        "https://example.com/\npath",
    ] {
        assert!(parse_http_endpoint(endpoint).is_err(), "{endpoint}");
    }
    for endpoint in [
        "https://[::1]:443/api",
        "http://127.0.0.1:8080",
        "https://例え.テスト/api",
    ] {
        assert!(parse_http_endpoint(endpoint).is_ok(), "{endpoint}");
    }
}

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
