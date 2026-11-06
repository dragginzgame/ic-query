mod filters;
mod list;
mod spec;

pub(super) use filters::NodeFilters;
pub(in crate::nns) use list::node_list_command;
pub(in crate::nns) use spec::NODE_SPEC;

pub(in crate::nns) fn node_command() -> clap::Command {
    crate::nns::leaf::command_with_list(
        &NODE_SPEC,
        ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        node_list_command(),
    )
    .subcommand(crate::nns::operational_status::command(
        crate::nns::operational_status::OperationalStatusSubject::Node,
    ))
}
