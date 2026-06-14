use crate::{
    cli::clap::{render_help, value_arg},
    nns::{
        leaf::{self, NnsLeafCommandSpec},
        node::report::{
            DEFAULT_NNS_NODE_SOURCE_ENDPOINT, NNS_NODE_SUBNET_KIND_APPLICATION,
            NNS_NODE_SUBNET_KIND_CLOUD_ENGINE, NNS_NODE_SUBNET_KIND_SYSTEM,
            NNS_NODE_SUBNET_KIND_UNKNOWN,
        },
    },
};

pub(super) const SUBNET_FILTER_ARG: &str = "subnet";
pub(super) const SUBNET_KIND_FILTER_ARG: &str = "kind";
pub(super) const DATA_CENTER_FILTER_ARG: &str = "data-center";
pub(super) const NODE_PROVIDER_FILTER_ARG: &str = "node-provider";
pub(super) const NODE_OPERATOR_FILTER_ARG: &str = "node-operator";

const NODE_LIST_HELP_AFTER: &str = "\
Examples:
  icq nns node list
  icq nns node list --verbose
  icq --network ic nns node list --format json
  icq nns node list --data-center zh2
  icq nns node list --node-provider 7at4h
  icq nns node list --subnet tdb26 --kind system

Force-refresh cached native NNS data:
  icq nns node refresh";
const NODE_INFO_HELP_AFTER: &str = "\
Examples:
  icq nns node info <node>
  icq nns node info <node-prefix>
  icq --network ic nns node info <node> --format json

Force-refresh cached native NNS data:
  icq nns node refresh";
const NODE_REFRESH_HELP_AFTER: &str = "\
Examples:
  icq nns node refresh
  icq --network ic nns node refresh --format json
  icq nns node refresh --dry-run --output .icq/node/ic/nodes.preview.json";

pub(super) const NODE_SPEC: NnsLeafCommandSpec = NnsLeafCommandSpec {
    command_name: "node",
    bin_name: "icq nns node",
    about: "Inspect NNS node metadata",
    list_about: "List cached mainnet NNS nodes",
    info_about: "Show one cached mainnet NNS node",
    refresh_about: "Force-refresh and cache NNS node metadata",
    list_help_after: NODE_LIST_HELP_AFTER,
    info_help_after: NODE_INFO_HELP_AFTER,
    refresh_help_after: NODE_REFRESH_HELP_AFTER,
    input_value_name: "node|node-prefix",
    input_help: "Node principal or unique node principal prefix",
    list_source_help: "IC API endpoint used if the node cache is missing",
    info_source_help: "IC API endpoint used if the node cache is missing",
    refresh_source_help: "IC API endpoint used for native NNS registry queries",
    verbose_help: "Show full node principals and registry metadata in text output",
    dry_run_help: "Fetch and validate without replacing the cached node report",
    output_help: "Also write the fetched node JSON to this path",
};

#[cfg(test)]
pub(in crate::nns) fn node_usage() -> String {
    leaf::usage(&NODE_SPEC)
}

pub(in crate::nns) fn node_list_usage() -> String {
    render_help(node_list_command())
}

pub(in crate::nns) fn node_info_usage() -> String {
    leaf::info_usage(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
}

pub(in crate::nns) fn node_refresh_usage() -> String {
    leaf::refresh_usage(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
}

pub(super) fn node_list_command() -> clap::Command {
    leaf::list_command(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
        .arg(
            value_arg(SUBNET_FILTER_ARG)
                .long(SUBNET_FILTER_ARG)
                .value_name("subnet|subnet-prefix")
                .help("Show only nodes assigned to a subnet principal or prefix"),
        )
        .arg(
            value_arg(SUBNET_KIND_FILTER_ARG)
                .long(SUBNET_KIND_FILTER_ARG)
                .value_name("application|cloud_engine|system|unknown")
                .value_parser([
                    NNS_NODE_SUBNET_KIND_APPLICATION,
                    NNS_NODE_SUBNET_KIND_CLOUD_ENGINE,
                    NNS_NODE_SUBNET_KIND_SYSTEM,
                    NNS_NODE_SUBNET_KIND_UNKNOWN,
                ])
                .help("Show only nodes assigned to subnets of this kind"),
        )
        .arg(
            value_arg(DATA_CENTER_FILTER_ARG)
                .long(DATA_CENTER_FILTER_ARG)
                .value_name("data-center|data-center-prefix")
                .help("Show only nodes in a data center id or prefix"),
        )
        .arg(
            value_arg(NODE_PROVIDER_FILTER_ARG)
                .long(NODE_PROVIDER_FILTER_ARG)
                .value_name("node-provider|node-provider-prefix")
                .help("Show only nodes owned by a node-provider principal or prefix"),
        )
        .arg(
            value_arg(NODE_OPERATOR_FILTER_ARG)
                .long(NODE_OPERATOR_FILTER_ARG)
                .value_name("node-operator|node-operator-prefix")
                .help("Show only nodes owned by a node-operator principal or prefix"),
        )
}
