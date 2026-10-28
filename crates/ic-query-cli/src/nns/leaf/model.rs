use ic_query::QueryProgress;
use ic_query::nns::{NnsInventoryInfoRequest, NnsInventoryListRequest, NnsInventoryRefreshRequest};

///
/// NnsLeafCommandSpec
///
/// Static command metadata shared by generic NNS leaf command families.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafCommandSpec {
    pub(in crate::nns) command_name: &'static str,
    pub(in crate::nns) about: &'static str,
    pub(in crate::nns) list_about: &'static str,
    pub(in crate::nns) info_about: &'static str,
    pub(in crate::nns) refresh_about: &'static str,
    pub(in crate::nns) list_help_after: &'static str,
    pub(in crate::nns) info_help_after: &'static str,
    pub(in crate::nns) refresh_help_after: &'static str,
    pub(in crate::nns) input_value_name: &'static str,
    pub(in crate::nns) input_help: &'static str,
    pub(in crate::nns) read_source_help: &'static str,
    pub(in crate::nns) refresh_source_help: &'static str,
    pub(in crate::nns) verbose_help: &'static str,
    pub(in crate::nns) dry_run_help: &'static str,
    pub(in crate::nns) output_help: &'static str,
}

impl NnsLeafCommandSpec {
    pub(super) fn bin_name(&self) -> String {
        format!("icq nns {}", self.command_name)
    }
}

///
/// NnsLeafReports
///
/// Report operations supplied by a concrete NNS leaf command family.
///

pub(in crate::nns) trait NnsLeafReports: Send + Sync {
    type ListReport: serde::Serialize;
    type InfoReport: serde::Serialize;
    type HostError: Into<crate::nns::NnsCommandError>;
    fn build_list_report(
        &self,
        request: &NnsInventoryListRequest,
        progress: &mut dyn QueryProgress,
    ) -> impl std::future::Future<Output = Result<Self::ListReport, Self::HostError>> + Send;
    fn build_info_report(
        &self,
        request: &NnsInventoryInfoRequest,
        progress: &mut dyn QueryProgress,
    ) -> impl std::future::Future<Output = Result<Self::InfoReport, Self::HostError>> + Send;
    fn refresh_report(
        &self,
        request: &NnsInventoryRefreshRequest,
    ) -> impl std::future::Future<
        Output = Result<ic_query::nns::NnsInventoryRefreshReport, Self::HostError>,
    > + Send;
    fn list_report_text(&self, report: &Self::ListReport) -> String;
    fn list_report_verbose_text(&self, report: &Self::ListReport) -> String;
    fn info_report_text(&self, report: &Self::InfoReport) -> String;
}
