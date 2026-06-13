pub mod report;

use super::{
    NnsCommandError,
    leaf::{self, NnsLeafCommandSpec},
};
use crate::nns::node_operator::report::{
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, NnsNodeOperatorCacheRequest,
    NnsNodeOperatorInfoRequest, NnsNodeOperatorListRequest, NnsNodeOperatorRefreshRequest,
    build_nns_node_operator_info_report, build_nns_node_operator_list_report,
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text, nns_node_operator_refresh_report_text,
    refresh_nns_node_operator_report,
};
use std::{ffi::OsString, path::Path};

#[cfg(test)]
use super::leaf::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions};

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

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        leaf::NnsLeafReportFns::new(
            build_nns_node_operator_list_report,
            build_nns_node_operator_info_report,
            refresh_nns_node_operator_report,
            nns_node_operator_list_report_text,
            nns_node_operator_list_report_verbose_text,
            nns_node_operator_info_report_text,
            nns_node_operator_refresh_report_text,
        ),
    )
}

impl leaf::NnsLeafCacheRequest for NnsNodeOperatorCacheRequest {
    fn from_root_network(icp_root: &Path, network: &str) -> Self {
        Self {
            icp_root: icp_root.to_path_buf(),
            network: network.to_string(),
        }
    }
}

impl leaf::NnsLeafListRequest for NnsNodeOperatorListRequest {
    type Cache = NnsNodeOperatorCacheRequest;

    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self {
        Self {
            cache,
            source_endpoint,
            now_unix_secs,
        }
    }
}

impl leaf::NnsLeafInfoRequest for NnsNodeOperatorInfoRequest {
    type Cache = NnsNodeOperatorCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        input: String,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            cache,
            source_endpoint,
            input,
            now_unix_secs,
        }
    }
}

impl leaf::NnsLeafRefreshRequest for NnsNodeOperatorRefreshRequest {
    type Cache = NnsNodeOperatorCacheRequest;

    fn from_leaf_parts(
        cache: Self::Cache,
        source_endpoint: String,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
        dry_run: bool,
        output_path: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            cache,
            source_endpoint,
            now_unix_secs,
            lock_stale_after_seconds,
            dry_run,
            output_path,
        }
    }
}

#[cfg(test)]
pub(super) fn node_operator_list_options<I>(args: I) -> Result<NnsLeafListOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafListOptions::parse(
        args,
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

#[cfg(test)]
pub(super) fn node_operator_info_options<I>(args: I) -> Result<NnsLeafInfoOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafInfoOptions::parse(
        args,
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

#[cfg(test)]
pub(super) fn node_operator_refresh_options<I>(
    args: I,
) -> Result<NnsLeafRefreshOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafRefreshOptions::parse(
        args,
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

#[cfg(test)]
pub(super) fn node_operator_usage() -> String {
    leaf::usage(&NODE_OPERATOR_SPEC)
}

#[cfg(test)]
pub(super) fn node_operator_list_usage() -> String {
    leaf::list_usage(
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

#[cfg(test)]
pub(super) fn node_operator_info_usage() -> String {
    leaf::info_usage(
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

#[cfg(test)]
pub(super) fn node_operator_refresh_usage() -> String {
    leaf::refresh_usage(
        &NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}
