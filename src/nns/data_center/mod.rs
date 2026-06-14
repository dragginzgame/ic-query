pub mod report;

use super::{
    NnsCommandError,
    leaf::{self, NnsLeafCommandSpec},
};
use crate::nns::data_center::report::{
    DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT, NnsDataCenterCacheRequest, NnsDataCenterHostError,
    NnsDataCenterInfoReport, NnsDataCenterInfoRequest, NnsDataCenterListReport,
    NnsDataCenterListRequest, NnsDataCenterRefreshReport, NnsDataCenterRefreshRequest,
    build_nns_data_center_info_report, build_nns_data_center_list_report,
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text, nns_data_center_refresh_report_text,
    refresh_nns_data_center_report,
};
use std::ffi::OsString;

const DATA_CENTER_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns data-center list
  icq nns data-center list --verbose
  icq --network ic nns data-center list --format json

Force-refresh cached native NNS data:
  icq nns data-center refresh";
const DATA_CENTER_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns data-center info <data-center>
  icq nns data-center info <data-center-prefix>
  icq --network ic nns data-center info <data-center> --format json

Force-refresh cached native NNS data:
  icq nns data-center refresh";
const DATA_CENTER_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns data-center refresh
  icq --network ic nns data-center refresh --format json
  icq nns data-center refresh --dry-run --output .icq/data-center/ic/data-centers.preview.json";

const DATA_CENTER_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "data-center",
    bin_name: "icq nns data-center",
    about: "Inspect NNS data-center metadata",
    list_about: "List cached mainnet NNS data centers",
    info_about: "Show one cached mainnet NNS data center",
    refresh_about: "Force-refresh and cache NNS data-center metadata",
    list_help_after: DATA_CENTER_LIST_HELP_AFTER,
    info_help_after: DATA_CENTER_INFO_HELP_AFTER,
    refresh_help_after: DATA_CENTER_REFRESH_HELP_AFTER,
    input_value_name: "data-center|data-center-prefix",
    input_help: "Data-center id or unique data-center id prefix",
    list_source_help: "IC API endpoint used if the data-center cache is missing",
    info_source_help: "IC API endpoint used if the data-center cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS registry queries",
    verbose_help: "Show GPS coordinates and registry metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached data-center report",
    output_help: "Also write the fetched data-center JSON to this path",
};

impl_nns_leaf_reports!(
    NnsDataCenterReports,
    cache = NnsDataCenterCacheRequest,
    list_request = NnsDataCenterListRequest,
    info_request = NnsDataCenterInfoRequest,
    refresh_request = NnsDataCenterRefreshRequest,
    list_report = NnsDataCenterListReport,
    info_report = NnsDataCenterInfoReport,
    refresh_report = NnsDataCenterRefreshReport,
    host_error = NnsDataCenterHostError,
    build_list = build_nns_data_center_list_report,
    build_info = build_nns_data_center_info_report,
    refresh = refresh_nns_data_center_report,
    list_text = nns_data_center_list_report_text,
    list_verbose_text = nns_data_center_list_report_verbose_text,
    info_text = nns_data_center_info_report_text,
    refresh_text = nns_data_center_refresh_report_text,
);

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &DATA_CENTER_SPEC,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        NnsDataCenterReports,
    )
}

impl_cached_leaf_requests!(
    NnsDataCenterCacheRequest,
    NnsDataCenterListRequest,
    NnsDataCenterInfoRequest,
    NnsDataCenterRefreshRequest
);

impl_leaf_test_helpers!(
    data_center_list_options,
    data_center_info_options,
    data_center_refresh_options,
    data_center_usage,
    data_center_list_usage,
    data_center_info_usage,
    data_center_refresh_usage,
    DATA_CENTER_SPEC,
    DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT
);
