mod filters;
mod list;
mod spec;

pub(super) use filters::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, SUBNET_FILTER_ARG,
    SUBNET_KIND_FILTER_ARG,
};
pub(in crate::nns) use list::node_list_command;
pub(in crate::nns) use spec::NODE_SPEC;

pub(in crate::nns) fn node_command() -> clap::Command {
    crate::nns::leaf::command_with_list(
        &NODE_SPEC,
        ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        node_list_command(),
    )
}
