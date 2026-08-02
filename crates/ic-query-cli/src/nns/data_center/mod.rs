//! Module: nns::data_center
//!
//! Responsibility: assemble data-center CLI specification, reports, and dispatch.
//! Does not own: reusable report construction or cache mechanics.
//! Boundary: adapts data-center arguments to the typed library API.

use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

mod reports;
mod spec;

pub(in crate::nns) fn command() -> clap::Command {
    leaf::command(
        &spec::DATA_CENTER_SPEC,
        ic_query::nns::data_center::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    )
}

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(
        matches,
        network,
        &spec::DATA_CENTER_SPEC,
        reports::NnsDataCenterReports,
    )
}

#[cfg(test)]
pub(in crate::nns) use spec::DATA_CENTER_SPEC;
