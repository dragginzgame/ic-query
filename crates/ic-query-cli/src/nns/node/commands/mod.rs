mod filters;
mod list;
mod spec;
#[cfg(test)]
mod usage;

pub(super) use filters::{
    DATA_CENTER_FILTER_ARG, NODE_OPERATOR_FILTER_ARG, NODE_PROVIDER_FILTER_ARG, SUBNET_FILTER_ARG,
    SUBNET_KIND_FILTER_ARG,
};
pub(super) use list::node_list_command;
pub(super) use spec::NODE_SPEC;
#[cfg(test)]
pub(in crate::nns) use usage::{node_info_usage, node_list_usage, node_refresh_usage, node_usage};

pub(super) fn node_command() -> clap::Command {
    crate::nns::leaf::command_with_list(
        &NODE_SPEC,
        ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        node_list_command(),
    )
}
