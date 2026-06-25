//! Module: nns::leaf::model
//!
//! Responsibility: shared command and request contracts for generic NNS leaf commands.
//! Does not own: clap construction, report rendering, or cache file IO.
//! Boundary: defines the traits used by data-center, node, operator, and provider commands.

use std::path::{Path, PathBuf};

///
/// NnsLeafCommandSpec
///
/// Static command metadata used to build one generic NNS leaf command family.
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
/// Cache identity contract shared by generic NNS leaf report requests.
///

pub(in crate::nns) trait NnsLeafCacheRequest: Clone {
    fn from_root_network(icp_root: &Path, network: &str) -> Self;
    fn icp_root(&self) -> &Path;
    fn network(&self) -> &str;
}

///
/// NnsLeafListRequest
///
/// Report-builder request contract for generic NNS leaf list commands.
///

pub(in crate::nns) trait NnsLeafListRequest {
    type Cache: NnsLeafCacheRequest;

    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self;
}

///
/// NnsLeafInfoRequest
///
/// Report-builder request contract for generic NNS leaf info commands.
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

///
/// NnsLeafRefreshRequest
///
/// Report-builder request contract for generic NNS leaf refresh commands.
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

    fn cache(&self) -> &Self::Cache;
    fn now_unix_secs(&self) -> u64;
    fn lock_stale_after_seconds(&self) -> u64;
    fn dry_run(&self) -> bool;
    fn output_path(&self) -> Option<&Path>;
}

///
/// NnsLeafReports
///
/// Report construction and rendering callbacks for generic NNS leaf command runners.
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

    fn list_report_text(&self, report: &Self::ListReport) -> String;

    fn list_report_verbose_text(&self, report: &Self::ListReport) -> String;

    fn info_report_text(&self, report: &Self::InfoReport) -> String;

    fn refresh_report_text(&self, report: &Self::RefreshReport) -> String;
}
