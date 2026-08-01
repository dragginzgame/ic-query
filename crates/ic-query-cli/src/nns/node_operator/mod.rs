//! Module: nns::node_operator
//!
//! Responsibility: assemble node-operator CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts node-operator arguments to the typed library API.

use crate::nns::leaf;

mod reports;
mod run;
mod spec;
#[cfg(test)]
pub(in crate::nns) mod test_helpers;

pub(super) use run::run;

pub(super) fn command() -> clap::Command {
    leaf::command(
        &spec::NODE_OPERATOR_SPEC,
        ic_query::nns::node_operator::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
    )
}
