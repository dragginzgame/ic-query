//! Module: nns::leaf::options::list
//!
//! Responsibility: parse clap options for generic NNS leaf list commands.
//! Does not own: clap command specs, report construction, or rendering.
//! Boundary: converts list command arguments into command-runner options.

use super::NnsCommonOptions;
use crate::nns::leaf::commands::VERBOSE_ARG;
use clap::ArgMatches;

///
/// NnsLeafListOptions
///
/// Parsed options accepted by generic NNS leaf list command runners.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafListOptions {
    pub(in crate::nns) common: NnsCommonOptions,
    pub(in crate::nns) verbose: bool,
}

impl NnsLeafListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,
            verbose: matches.get_flag(VERBOSE_ARG),
        }
    }
}
