//! Module: nns::node_provider
//!
//! Responsibility: assemble node-provider CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-provider arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

mod reports;
mod spec;

pub(super) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_PROVIDER_SPEC,
        ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    )
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(
        matches,
        network,
        &spec::NODE_PROVIDER_SPEC,
        reports::NnsNodeProviderReports,
    )
}

#[cfg(test)]
pub(in crate::nns) mod test_helpers {
    use super::spec::NODE_PROVIDER_SPEC;
    use crate::nns::{NnsCommandError, leaf};
    use ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT;

    impl_leaf_test_helpers!(
        node_provider_list_options,
        node_provider_info_options,
        node_provider_refresh_options,
        node_provider_usage,
        node_provider_list_usage,
        node_provider_info_usage,
        node_provider_refresh_usage,
        NODE_PROVIDER_SPEC,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT
    );
}
