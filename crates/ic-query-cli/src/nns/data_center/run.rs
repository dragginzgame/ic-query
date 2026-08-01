use super::{reports::NnsDataCenterReports, spec::DATA_CENTER_SPEC};
use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(matches, network, &DATA_CENTER_SPEC, NnsDataCenterReports)
}
