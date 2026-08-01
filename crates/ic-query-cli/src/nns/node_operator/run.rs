use super::{reports::NnsNodeOperatorReports, spec::NODE_OPERATOR_SPEC};
use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(
        matches,
        network,
        &NODE_OPERATOR_SPEC,
        NnsNodeOperatorReports,
    )
}
