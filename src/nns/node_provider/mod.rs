pub mod report;

use super::{
    NnsCommandError,
    leaf::{self, NnsLeafCommandSpec},
};
use crate::nns::node_provider::report::{
    DEFAULT_NNS_SOURCE_ENDPOINT, NnsNodeProviderCacheRequest, NnsNodeProviderInfoRequest,
    NnsNodeProviderListRequest, NnsNodeProviderRefreshRequest, build_nns_node_provider_info_report,
    build_nns_node_provider_list_report, nns_node_provider_info_report_text,
    nns_node_provider_list_report_text, nns_node_provider_list_report_verbose_text,
    nns_node_provider_refresh_report_text, refresh_nns_node_provider_report,
};
use std::{ffi::OsString, path::Path};

#[cfg(test)]
use super::leaf::{NnsLeafInfoOptions, NnsLeafListOptions, NnsLeafRefreshOptions};

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

pub(super) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    leaf::run_cached_leaf(
        args,
        &NODE_PROVIDER_SPEC,
        DEFAULT_NNS_SOURCE_ENDPOINT,
        leaf::NnsLeafReportFns::new(
            build_nns_node_provider_list_report,
            build_nns_node_provider_info_report,
            refresh_nns_node_provider_report,
            nns_node_provider_list_report_text,
            nns_node_provider_list_report_verbose_text,
            nns_node_provider_info_report_text,
            nns_node_provider_refresh_report_text,
        ),
    )
}

impl leaf::NnsLeafCacheRequest for NnsNodeProviderCacheRequest {
    fn from_root_network(icp_root: &Path, network: &str) -> Self {
        Self {
            icp_root: icp_root.to_path_buf(),
            network: network.to_string(),
        }
    }
}

impl leaf::NnsLeafListRequest for NnsNodeProviderListRequest {
    type Cache = NnsNodeProviderCacheRequest;

    fn from_leaf_parts(cache: Self::Cache, source_endpoint: String, now_unix_secs: u64) -> Self {
        Self {
            cache,
            source_endpoint,
            now_unix_secs,
        }
    }
}

impl leaf::NnsLeafInfoRequest for NnsNodeProviderInfoRequest {
    type Cache = NnsNodeProviderCacheRequest;

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

impl leaf::NnsLeafRefreshRequest for NnsNodeProviderRefreshRequest {
    type Cache = NnsNodeProviderCacheRequest;

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
pub(super) fn node_provider_list_options<I>(args: I) -> Result<NnsLeafListOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafListOptions::parse(args, &NODE_PROVIDER_SPEC, DEFAULT_NNS_SOURCE_ENDPOINT)
}

#[cfg(test)]
pub(super) fn node_provider_info_options<I>(args: I) -> Result<NnsLeafInfoOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafInfoOptions::parse(args, &NODE_PROVIDER_SPEC, DEFAULT_NNS_SOURCE_ENDPOINT)
}

#[cfg(test)]
pub(super) fn node_provider_refresh_options<I>(
    args: I,
) -> Result<NnsLeafRefreshOptions, NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    NnsLeafRefreshOptions::parse(args, &NODE_PROVIDER_SPEC, DEFAULT_NNS_SOURCE_ENDPOINT)
}

#[cfg(test)]
pub(super) fn node_provider_usage() -> String {
    leaf::usage(&NODE_PROVIDER_SPEC)
}

#[cfg(test)]
pub(super) fn node_provider_list_usage() -> String {
    leaf::list_usage(&NODE_PROVIDER_SPEC, DEFAULT_NNS_SOURCE_ENDPOINT)
}

#[cfg(test)]
pub(super) fn node_provider_info_usage() -> String {
    leaf::info_usage(&NODE_PROVIDER_SPEC, DEFAULT_NNS_SOURCE_ENDPOINT)
}

#[cfg(test)]
pub(super) fn node_provider_refresh_usage() -> String {
    leaf::refresh_usage(&NODE_PROVIDER_SPEC, DEFAULT_NNS_SOURCE_ENDPOINT)
}
