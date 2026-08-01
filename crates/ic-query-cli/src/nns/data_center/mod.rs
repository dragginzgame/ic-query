//! Module: nns::data_center
//!
//! Responsibility: assemble data-center CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts data-center arguments to the typed library API.

use crate::nns::leaf;

mod reports;
mod run;
mod spec;
#[cfg(test)]
pub(in crate::nns) mod test_helpers;

pub(super) use run::run;

pub(super) fn command() -> clap::Command {
    leaf::command(
        &spec::DATA_CENTER_SPEC,
        ic_query::nns::data_center::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    )
}
