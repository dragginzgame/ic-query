pub mod report;

use super::{
    NnsCommandError,
    leaf::{self, NnsLeafCommandSpec},
};
use crate::nns::node_operator::report::{
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, NnsNodeOperatorCacheRequest,
    NnsNodeOperatorHostError, NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest,
    NnsNodeOperatorListReport, NnsNodeOperatorListRequest, NnsNodeOperatorRefreshReport,
    NnsNodeOperatorRefreshRequest, build_nns_node_operator_info_report,
    build_nns_node_operator_list_report, nns_node_operator_info_report_text,
    nns_node_operator_list_report_text, nns_node_operator_list_report_verbose_text,
    nns_node_operator_refresh_report_text, refresh_nns_node_operator_report,
};
use std::ffi::OsString;

const NODE_OPERATOR_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node-operator list
  icq nns node-operator list --verbose
  icq --network ic nns node-operator list --format json

Force-refresh cached native NNS data:
  icq nns node-operator refresh";
const NODE_OPERATOR_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node-operator info <node-operator>
  icq nns node-operator info <node-operator-prefix>
  icq --network ic nns node-operator info <node-operator> --format json

Force-refresh cached native NNS data:
  icq nns node-operator refresh";
const NODE_OPERATOR_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns node-operator refresh
  icq --network ic nns node-operator refresh --format json
  icq nns node-operator refresh --dry-run --output .icq/node-operator/ic/operators.preview.json";

const NODE_OPERATOR_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "node-operator",
    bin_name: "icq nns node-operator",
    about: "Inspect NNS node-operator metadata",
    list_about: "List cached mainnet NNS node operators",
    info_about: "Show one cached mainnet NNS node operator",
    refresh_about: "Force-refresh and cache NNS node-operator metadata",
    list_help_after: NODE_OPERATOR_LIST_HELP_AFTER,
    info_help_after: NODE_OPERATOR_INFO_HELP_AFTER,
    refresh_help_after: NODE_OPERATOR_REFRESH_HELP_AFTER,
    input_value_name: "node-operator|node-operator-prefix",
    input_help: "Node-operator principal or unique node-operator principal prefix",
    list_source_help: "IC API endpoint used if the node-operator cache is missing",
    info_source_help: "IC API endpoint used if the node-operator cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS registry queries",
    verbose_help: "Show full node-operator principals and registry metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached node-operator report",
    output_help: "Also write the fetched node-operator JSON to this path",
};

impl_nns_leaf_reports!(
    NnsNodeOperatorReports,
    cache = NnsNodeOperatorCacheRequest,
    list_request = NnsNodeOperatorListRequest,
    info_request = NnsNodeOperatorInfoRequest,
    refresh_request = NnsNodeOperatorRefreshRequest,
    list_report = NnsNodeOperatorListReport,
    info_report = NnsNodeOperatorInfoReport,
    refresh_report = NnsNodeOperatorRefreshReport,
    host_error = NnsNodeOperatorHostError,
    build_list = build_nns_node_operator_list_report,
    build_info = build_nns_node_operator_info_report,
    refresh = refresh_nns_node_operator_report,
    list_text = nns_node_operator_list_report_text,
    list_verbose_text = nns_node_operator_list_report_verbose_text,
    info_text = nns_node_operator_info_report_text,
    refresh_text = nns_node_operator_refresh_report_text,
);

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        NnsNodeOperatorReports,
    )
}

impl_cached_leaf_requests!(
    NnsNodeOperatorCacheRequest,
    NnsNodeOperatorListRequest,
    NnsNodeOperatorInfoRequest,
    NnsNodeOperatorRefreshRequest
);

impl_leaf_test_helpers!(
    node_operator_list_options,
    node_operator_info_options,
    node_operator_refresh_options,
    node_operator_usage,
    node_operator_list_usage,
    node_operator_info_usage,
    node_operator_refresh_usage,
    NODE_OPERATOR_SPEC,
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT
);
