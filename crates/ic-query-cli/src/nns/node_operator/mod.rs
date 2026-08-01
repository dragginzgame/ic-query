//! Module: nns::node_operator
//!
//! Responsibility: assemble node-operator CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-operator arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

mod reports;
mod spec;

pub(super) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_OPERATOR_SPEC,
        ic_query::nns::node_operator::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(
        matches,
        network,
        &spec::NODE_OPERATOR_SPEC,
        reports::NnsNodeOperatorReports,
    )
}

#[cfg(test)]
pub(in crate::nns) mod test_helpers {
    use super::spec::NODE_OPERATOR_SPEC;
    use crate::nns::{NnsCommandError, leaf};
    use ic_query::nns::node_operator::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT;

    impl_leaf_test_helpers!(
        node_operator_list_options,
        node_operator_info_options,
        node_operator_refresh_options,
        node_operator_usage,
        node_operator_list_usage,
        node_operator_info_usage,
        node_operator_refresh_usage,
        NODE_OPERATOR_SPEC,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT
    );
}
