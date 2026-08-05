#[cfg(feature = "nns-host")]
use std::path::Path;
use std::path::PathBuf;

///
/// TopologyRequestParts
///
/// Shared read settings required by host-backed topology source requests.
///

#[cfg(feature = "nns-host")]
pub(in crate::nns::topology::report) trait TopologyRequestParts {
    fn cache_root(&self) -> &Path;
    fn network(&self) -> &str;
    fn source_endpoint(&self) -> &str;
    fn now_unix_secs(&self) -> u64;
}

///
/// TopologyRefreshParts
///
/// Refresh-specific settings added to a host-backed topology request.
///

#[cfg(feature = "nns-host")]
pub(in crate::nns::topology::report) trait TopologyRefreshParts:
    TopologyRequestParts
{
    fn lock_stale_after_seconds(&self) -> u64;
    fn dry_run(&self) -> bool;
}

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
impl TopologyRequestParts for NnsTopologyReadRequest {
    fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    fn network(&self) -> &str {
        &self.network
    }

    fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

///
/// NnsTopologyRefreshRequest
///
/// Request accepted by the complete NNS topology refresh builder.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshRequest {
    pub cache_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
}

impl NnsTopologyRefreshRequest {
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            lock_stale_after_seconds,
            dry_run: false,
        }
    }

    #[must_use]
    pub const fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

#[cfg(feature = "nns-host")]
impl TopologyRequestParts for NnsTopologyRefreshRequest {
    fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    fn network(&self) -> &str {
        &self.network
    }

    fn source_endpoint(&self) -> &str {
        &self.source_endpoint
    }

    fn now_unix_secs(&self) -> u64 {
        self.now_unix_secs
    }
}

#[cfg(feature = "nns-host")]
impl TopologyRefreshParts for NnsTopologyRefreshRequest {
    fn lock_stale_after_seconds(&self) -> u64 {
        self.lock_stale_after_seconds
    }

    fn dry_run(&self) -> bool {
        self.dry_run
    }
}

#[cfg(feature = "nns-host")]
pub(in crate::nns::topology::report) fn summary_request_from(
    request: &impl TopologyRequestParts,
) -> NnsTopologyReadRequest {
    NnsTopologyReadRequest::new(
        request.cache_root(),
        request.network(),
        request.source_endpoint(),
        request.now_unix_secs(),
    )
}
