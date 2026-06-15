use super::{
    filters::{
        data_center_filter_arg, node_operator_filter_arg, node_provider_filter_arg,
        subnet_filter_arg, subnet_kind_filter_arg,
    },
    spec::NODE_SPEC,
};
use crate::nns::{leaf, node::report::DEFAULT_NNS_NODE_SOURCE_ENDPOINT};

pub(in crate::nns::node) fn node_list_command() -> clap::Command {
    leaf::list_command(&NODE_SPEC, DEFAULT_NNS_NODE_SOURCE_ENDPOINT)
        .arg(subnet_filter_arg())
        .arg(subnet_kind_filter_arg())
        .arg(data_center_filter_arg())
        .arg(node_provider_filter_arg())
        .arg(node_operator_filter_arg())
}
