use super::{list::node_list_command, spec::NODE_SPEC};
use crate::{cli::clap::render_help, nns::leaf};
use ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;

#[cfg(test)]
pub(in crate::nns) fn node_usage() -> String {
    render_help(super::node_command())
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
