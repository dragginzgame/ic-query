//! Module: nns::node_provider
//!
//! Responsibility: assemble node-provider CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-provider arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

mod reports;
mod spec;

pub(in crate::nns) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_PROVIDER_SPEC,
        ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    )
    .subcommand(crate::nns::operational_status::command(
        crate::nns::operational_status::OperationalStatusSubject::NodeProvider,
    ))
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    if let Some(("status", matches)) = matches.subcommand() {
        return crate::nns::operational_status::run(
            crate::nns::operational_status::OperationalStatusSubject::NodeProvider,
            matches,
            network,
        );
    }
    leaf::run_cached_leaf(
        matches,
        network,
        &spec::NODE_PROVIDER_SPEC,
        reports::NnsNodeProviderReports,
    )
}

#[cfg(test)]
pub(in crate::nns) use spec::NODE_PROVIDER_SPEC;
