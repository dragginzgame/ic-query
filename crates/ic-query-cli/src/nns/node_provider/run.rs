use super::{reports::NnsNodeProviderReports, spec::NODE_PROVIDER_SPEC};
use crate::nns::{NnsCommandError, leaf};
use clap::ArgMatches;

pub(in crate::nns) fn run(matches: &ArgMatches, network: &str) -> Result<(), NnsCommandError> {
    leaf::run_cached_leaf(
        matches,
        network,
        &NODE_PROVIDER_SPEC,
        NnsNodeProviderReports,
    )
}
