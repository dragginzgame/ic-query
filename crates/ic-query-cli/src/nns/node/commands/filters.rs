use crate::cli::clap::value_arg;
use ic_query::subnet_catalog::SubnetKind;

pub(in crate::nns::node) const SUBNET_FILTER_ARG: &str = "subnet";
pub(in crate::nns::node) const SUBNET_KIND_FILTER_ARG: &str = "kind";
pub(in crate::nns::node) const DATA_CENTER_FILTER_ARG: &str = "data-center";
pub(in crate::nns::node) const NODE_PROVIDER_FILTER_ARG: &str = "node-provider";
pub(in crate::nns::node) const NODE_OPERATOR_FILTER_ARG: &str = "node-operator";

pub(super) fn subnet_filter_arg() -> clap::Arg {
    value_arg(SUBNET_FILTER_ARG)
        .long(SUBNET_FILTER_ARG)
        .value_name("subnet|subnet-prefix")
        .help("Show only nodes assigned to a subnet principal or prefix")
}

pub(super) fn subnet_kind_filter_arg() -> clap::Arg {
    value_arg(SUBNET_KIND_FILTER_ARG)
        .long(SUBNET_KIND_FILTER_ARG)
        .value_name("application|cloud_engine|system|unknown")
        .value_parser([
            SubnetKind::Application.as_str(),
            SubnetKind::CloudEngine.as_str(),
            SubnetKind::System.as_str(),
            SubnetKind::Unknown.as_str(),
        ])
        .help("Show only nodes assigned to subnets of this kind")
}

pub(super) fn data_center_filter_arg() -> clap::Arg {
    value_arg(DATA_CENTER_FILTER_ARG)
        .long(DATA_CENTER_FILTER_ARG)
        .value_name("data-center|data-center-prefix")
        .help("Show only nodes in a data center id or prefix")
}

pub(super) fn node_provider_filter_arg() -> clap::Arg {
    value_arg(NODE_PROVIDER_FILTER_ARG)
        .long(NODE_PROVIDER_FILTER_ARG)
        .value_name("node-provider|node-provider-prefix")
        .help("Show only nodes owned by a node-provider principal or prefix")
}

pub(super) fn node_operator_filter_arg() -> clap::Arg {
    value_arg(NODE_OPERATOR_FILTER_ARG)
        .long(NODE_OPERATOR_FILTER_ARG)
        .value_name("node-operator|node-operator-prefix")
        .help("Show only nodes owned by a node-operator principal or prefix")
}
