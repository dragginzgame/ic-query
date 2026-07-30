use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsInventoryRefreshRequest,
};
use std::path::{Path, PathBuf};

///
/// NnsLeafCommandSpec
///
/// Static command metadata shared by generic NNS leaf command families.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafCommandSpec {
    pub(in crate::nns) command_name: &'static str,
    pub(in crate::nns) bin_name: &'static str,
    pub(in crate::nns) about: &'static str,
    pub(in crate::nns) list_about: &'static str,
    pub(in crate::nns) info_about: &'static str,
    pub(in crate::nns) refresh_about: &'static str,
    pub(in crate::nns) list_help_after: &'static str,
    pub(in crate::nns) info_help_after: &'static str,
    pub(in crate::nns) refresh_help_after: &'static str,
    pub(in crate::nns) input_value_name: &'static str,
    pub(in crate::nns) input_help: &'static str,
    pub(in crate::nns) list_source_help: &'static str,
    pub(in crate::nns) info_source_help: &'static str,
    pub(in crate::nns) refresh_source_help: &'static str,
    pub(in crate::nns) verbose_help: &'static str,
    pub(in crate::nns) dry_run_help: &'static str,
    pub(in crate::nns) output_help: &'static str,
}

///
/// NnsLeafCacheRequest
///
/// Cache request construction required by a generic NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafCacheRequest: Clone {
    fn from_root_network(cache_root: &Path, network: &str) -> Self;
    fn network(&self) -> &str;
}

impl NnsLeafCacheRequest for NnsInventoryCacheRequest {
    fn from_root_network(cache_root: &Path, network: &str) -> Self {
        Self::new(cache_root, network)
    }

    fn network(&self) -> &str {
        &self.network
    }
}

///
/// NnsLeafListRequest
///
/// List request construction required by a generic NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafListRequest {
    type Cache: NnsLeafCacheRequest;
    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self;
}

impl NnsLeafListRequest for NnsInventoryListRequest {
    type Cache = NnsInventoryCacheRequest;

    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self {
        Self::new(cache, source_endpoint, now_unix_secs)
    }
}

///
/// NnsLeafInfoRequest
///
/// Detail request construction required by a generic NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafInfoRequest {
    type Cache: NnsLeafCacheRequest;
    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        input: String,
        now_unix_secs: u64,
    ) -> Self;
}

impl NnsLeafInfoRequest for NnsInventoryInfoRequest {
    type Cache = NnsInventoryCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        input: String,
        now_unix_secs: u64,
    ) -> Self {
        Self::new(cache, source_endpoint, input, now_unix_secs)
    }
}

///
/// NnsLeafRefreshRequest
///
/// Refresh request construction required by a generic NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafRefreshRequest {
    type Cache: NnsLeafCacheRequest;
    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
        dry_run: bool,
        output_path: Option<PathBuf>,
    ) -> Self;
}

impl NnsLeafRefreshRequest for NnsInventoryRefreshRequest {
    type Cache = NnsInventoryCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
        dry_run: bool,
        output_path: Option<PathBuf>,
    ) -> Self {
        let mut request = Self::new(
            cache,
            source_endpoint,
            now_unix_secs,
            lock_stale_after_seconds,
        )
        .with_dry_run(dry_run);
        if let Some(output_path) = output_path {
            request = request.with_output_path(output_path);
        }
        request
    }
}

///
/// NnsLeafReports
///
/// Report operations supplied by a concrete NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafReports {
    type Cache: NnsLeafCacheRequest;
    type ListRequest: NnsLeafListRequest<Cache = Self::Cache>;
    type InfoRequest: NnsLeafInfoRequest<Cache = Self::Cache>;
    type RefreshRequest: NnsLeafRefreshRequest<Cache = Self::Cache>;
    type ListReport: serde::Serialize;
    type InfoReport: serde::Serialize;
    type RefreshReport: serde::Serialize;
    type HostError: Into<crate::nns::NnsCommandError>;
    fn build_list_report(
        &self,
        request: &Self::ListRequest,
    ) -> Result<Self::ListReport, Self::HostError>;
    fn build_info_report(
        &self,
        request: &Self::InfoRequest,
    ) -> Result<Self::InfoReport, Self::HostError>;
    fn refresh_report(
        &self,
        request: &Self::RefreshRequest,
    ) -> Result<Self::RefreshReport, Self::HostError>;
    fn cache_path(&self, cache: &Self::Cache) -> PathBuf;
    fn list_report_text(&self, report: &Self::ListReport) -> String;
    fn list_report_verbose_text(&self, report: &Self::ListReport) -> String;
    fn info_report_text(&self, report: &Self::InfoReport) -> String;
    fn refresh_report_text(&self, report: &Self::RefreshReport) -> String;
}
