pub mod report;

use super::{
    NnsCommandError,
    leaf::{self, NnsLeafCommandSpec},
};
use crate::nns::node_provider::report::{
    DEFAULT_NNS_SOURCE_ENDPOINT, NnsNodeProviderCacheRequest, NnsNodeProviderHostError,
    NnsNodeProviderInfoReport, NnsNodeProviderInfoRequest, NnsNodeProviderListReport,
    NnsNodeProviderListRequest, NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest,
    build_nns_node_provider_info_report, build_nns_node_provider_list_report,
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text, nns_node_provider_refresh_report_text,
    refresh_nns_node_provider_report,
};
use std::ffi::OsString;

const NODE_PROVIDER_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider list
  icq nns node-provider list --verbose
  icq --network ic nns node-provider list --format json

Force-refresh cached native NNS data:
  icq nns node-provider refresh";
const NODE_PROVIDER_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider info <node-provider>
  icq nns node-provider info <node-provider-prefix>
  icq --network ic nns node-provider info <node-provider> --format json

Force-refresh cached native NNS data:
  icq nns node-provider refresh";
const NODE_PROVIDER_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns node-provider refresh
  icq --network ic nns node-provider refresh --format json
  icq nns node-provider refresh --dry-run --output .icq/node-provider/ic/providers.preview.json";

const NODE_PROVIDER_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "node-provider",
    bin_name: "icq nns node-provider",
    about: "Inspect NNS node-provider metadata",
    list_about: "List cached mainnet NNS node providers",
    info_about: "Show one cached mainnet NNS node provider",
    refresh_about: "Force-refresh and cache NNS node-provider metadata",
    list_help_after: NODE_PROVIDER_LIST_HELP_AFTER,
    info_help_after: NODE_PROVIDER_INFO_HELP_AFTER,
    refresh_help_after: NODE_PROVIDER_REFRESH_HELP_AFTER,
    input_value_name: "node-provider|node-provider-prefix",
    input_help: "Node-provider principal or unique node-provider principal prefix",
    list_source_help: "IC API endpoint used if the node-provider cache is missing",
    info_source_help: "IC API endpoint used if the node-provider cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS governance and registry queries",
    verbose_help: "Show full node-provider principals and reward-account metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached node-provider report",
    output_help: "Also write the fetched node-provider JSON to this path",
};

struct NnsNodeProviderReports;

impl leaf::NnsLeafReports for NnsNodeProviderReports {
    type Cache = NnsNodeProviderCacheRequest;
    type ListRequest = NnsNodeProviderListRequest;
    type InfoRequest = NnsNodeProviderInfoRequest;
    type RefreshRequest = NnsNodeProviderRefreshRequest;
    type ListReport = NnsNodeProviderListReport;
    type InfoReport = NnsNodeProviderInfoReport;
    type RefreshReport = NnsNodeProviderRefreshReport;
    type HostError = NnsNodeProviderHostError;

    fn build_list_report(
        &self,
        request: &Self::ListRequest,
    ) -> Result<Self::ListReport, Self::HostError> {
        build_nns_node_provider_list_report(request)
    }

    fn build_info_report(
        &self,
        request: &Self::InfoRequest,
    ) -> Result<Self::InfoReport, Self::HostError> {
        build_nns_node_provider_info_report(request)
    }

    fn refresh_report(
        &self,
        request: &Self::RefreshRequest,
    ) -> Result<Self::RefreshReport, Self::HostError> {
        refresh_nns_node_provider_report(request)
    }

    fn list_report_text(&self, report: &Self::ListReport) -> String {
        nns_node_provider_list_report_text(report)
    }

    fn list_report_verbose_text(&self, report: &Self::ListReport) -> String {
        nns_node_provider_list_report_verbose_text(report)
    }

    fn info_report_text(&self, report: &Self::InfoReport) -> String {
        nns_node_provider_info_report_text(report)
    }

    fn refresh_report_text(&self, report: &Self::RefreshReport) -> String {
        nns_node_provider_refresh_report_text(report)
    }
}

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &NODE_PROVIDER_SPEC,
        DEFAULT_NNS_SOURCE_ENDPOINT,
        NnsNodeProviderReports,
    )
}

impl_cached_leaf_requests!(
    NnsNodeProviderCacheRequest,
    NnsNodeProviderListRequest,
    NnsNodeProviderInfoRequest,
    NnsNodeProviderRefreshRequest
);

impl_leaf_test_helpers!(
    node_provider_list_options,
    node_provider_info_options,
    node_provider_refresh_options,
    node_provider_usage,
    node_provider_list_usage,
    node_provider_info_usage,
    node_provider_refresh_usage,
    NODE_PROVIDER_SPEC,
    DEFAULT_NNS_SOURCE_ENDPOINT
);
