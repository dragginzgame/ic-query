use std::path::PathBuf;

///
/// NnsTopologyReadRequest
///
/// Shared request accepted by every read-only NNS topology report builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyReadRequest {
    pub cache_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

impl NnsTopologyReadRequest {
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }
}

#[cfg(feature = "nns-host")]
impl NnsTopologyReadRequest {
    /// Derive caller-clock provenance for a live collection; cached reports retain their original provenance.
    #[must_use]
    pub fn source_request(&self) -> crate::nns::NnsSourceRequest {
        crate::nns::NnsSourceRequest::new(
            &self.network,
            &self.source_endpoint,
            crate::subnet_catalog::format_utc_timestamp_secs(self.now_unix_secs),
            "ic-query",
        )
    }
}

///
/// NnsTopologyRefreshRequest
///
/// Request accepted by the complete NNS topology refresh builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshRequest {
    /// Network, endpoint, clock and cache scope shared with topology reads.
    pub read: NnsTopologyReadRequest,

    pub dry_run: bool,
}

impl NnsTopologyRefreshRequest {
    #[must_use]
    pub const fn new(read: NnsTopologyReadRequest) -> Self {
        Self {
            read,

            dry_run: false,
        }
    }

    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}
