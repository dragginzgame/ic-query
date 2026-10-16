//! Explicit endpoint and caller-clock scope for official Dashboard requests.

///
/// IcDashboardRequest
///
/// Shared endpoint and collection clock; each operation selects its own API endpoint.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcDashboardRequest {
    /// Explicit Dashboard API base endpoint.
    pub source_endpoint: String,
    /// Caller-supplied collection time as Unix seconds.
    pub now_unix_secs: u64,
}

impl IcDashboardRequest {
    /// Construct collection scope without selecting an endpoint implicitly.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }

    /// Derive live collection provenance from this request's endpoint and clock.
    #[cfg(feature = "dashboard-host")]
    #[must_use]
    pub fn source_request(&self) -> crate::ic::IcSourceRequest {
        crate::ic::IcSourceRequest::new(
            &self.source_endpoint,
            crate::subnet_catalog::format_utc_timestamp_secs(self.now_unix_secs),
            "ic-query",
        )
    }
}
