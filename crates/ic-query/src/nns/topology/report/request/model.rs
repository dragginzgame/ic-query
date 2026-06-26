#[cfg(feature = "host")]
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "host")]
pub(in crate::nns::topology::report) trait TopologyRequestParts {
    fn icp_root(&self) -> &Path;
    fn network(&self) -> &str;
    fn source_endpoint(&self) -> &str;
    fn now_unix_secs(&self) -> u64;
}

#[cfg(feature = "host")]
pub(in crate::nns::topology::report) trait TopologyRefreshParts:
    TopologyRequestParts
{
    fn lock_stale_after_seconds(&self) -> u64;
    fn dry_run(&self) -> bool;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyReadRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
}

pub type NnsTopologySummaryRequest = NnsTopologyReadRequest;
pub type NnsTopologyCoverageRequest = NnsTopologyReadRequest;
pub type NnsTopologyVersionsRequest = NnsTopologyReadRequest;
pub type NnsTopologyHealthRequest = NnsTopologyReadRequest;
pub type NnsTopologyGapsRequest = NnsTopologyReadRequest;
pub type NnsTopologyCapacityRequest = NnsTopologyReadRequest;
pub type NnsTopologyRegionsRequest = NnsTopologyReadRequest;
pub type NnsTopologyProvidersRequest = NnsTopologyReadRequest;

#[cfg(feature = "host")]
impl TopologyRequestParts for NnsTopologyReadRequest {
    fn icp_root(&self) -> &Path {
        &self.icp_root
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsTopologyRefreshRequest {
    pub icp_root: PathBuf,
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
    pub dry_run: bool,
}

#[cfg(feature = "host")]
impl TopologyRequestParts for NnsTopologyRefreshRequest {
    fn icp_root(&self) -> &Path {
        &self.icp_root
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

#[cfg(feature = "host")]
impl TopologyRefreshParts for NnsTopologyRefreshRequest {
    fn lock_stale_after_seconds(&self) -> u64 {
        self.lock_stale_after_seconds
    }

    fn dry_run(&self) -> bool {
        self.dry_run
    }
}

#[cfg(feature = "host")]
pub(in crate::nns::topology::report) fn summary_request_from(
    request: &impl TopologyRequestParts,
) -> NnsTopologySummaryRequest {
    NnsTopologySummaryRequest {
        icp_root: request.icp_root().to_path_buf(),
        network: request.network().to_string(),
        source_endpoint: request.source_endpoint().to_string(),
        now_unix_secs: request.now_unix_secs(),
    }
}
