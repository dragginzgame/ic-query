use ic_query::nns::{
    NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest,
    NnsInventoryRefreshRequest,
};
use std::path::PathBuf;

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
/// NnsLeafReports
///
/// Report operations supplied by a concrete NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafReports {
    type ListReport: serde::Serialize;
    type InfoReport: serde::Serialize;
    type RefreshReport: serde::Serialize;
    type HostError: Into<crate::nns::NnsCommandError>;
    fn build_list_report(
        &self,
        request: &NnsInventoryListRequest,
    ) -> Result<Self::ListReport, Self::HostError>;
    fn build_info_report(
        &self,
        request: &NnsInventoryInfoRequest,
    ) -> Result<Self::InfoReport, Self::HostError>;
    fn refresh_report(
        &self,
        request: &NnsInventoryRefreshRequest,
    ) -> Result<Self::RefreshReport, Self::HostError>;
    fn cache_path(&self, cache: &NnsInventoryCacheRequest) -> PathBuf;
    fn list_report_text(&self, report: &Self::ListReport) -> String;
    fn list_report_verbose_text(&self, report: &Self::ListReport) -> String;
    fn info_report_text(&self, report: &Self::InfoReport) -> String;
    fn refresh_report_text(&self, report: &Self::RefreshReport) -> String;
}
