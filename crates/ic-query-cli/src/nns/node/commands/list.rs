use super::{filters::NodeFilters, spec::NODE_SPEC};
use crate::cli::clap::with_options;
use crate::nns::leaf;
use ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;

pub(in crate::nns) fn node_list_command() -> clap::Command {
    with_options::<NodeFilters>(leaf::list_command(
        &NODE_SPEC,
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
    ))
}
