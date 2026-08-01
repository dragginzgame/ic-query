//! Module: nns::node_provider
//!
//! Responsibility: assemble node-provider CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-provider arguments to the typed library API.

use crate::nns::leaf;

mod reports;
mod run;
mod spec;
#[cfg(test)]
pub(in crate::nns) mod test_helpers;

pub(super) use run::run;

pub(super) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_PROVIDER_SPEC,
        ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
    )
}
