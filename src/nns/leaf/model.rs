use std::path::{Path, PathBuf};

///
/// NnsLeafCommandSpec
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

pub(in crate::nns) trait NnsLeafCacheRequest: Clone {
    fn from_root_network(icp_root: &Path, network: &str) -> Self;
}

pub(in crate::nns) trait NnsLeafListRequest {
    type Cache: NnsLeafCacheRequest;

    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self;
}

pub(in crate::nns) trait NnsLeafInfoRequest {
    type Cache: NnsLeafCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        input: String,
        now_unix_secs: u64,
    ) -> Self;
}

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

    fn list_report_text(&self, report: &Self::ListReport) -> String;

    fn list_report_verbose_text(&self, report: &Self::ListReport) -> String;

    fn info_report_text(&self, report: &Self::InfoReport) -> String;

    fn refresh_report_text(&self, report: &Self::RefreshReport) -> String;
}
