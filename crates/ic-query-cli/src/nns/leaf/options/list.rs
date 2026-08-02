//! Module: nns::leaf::options::list
//!
//! Responsibility: parse clap options for generic NNS leaf list commands.
//! Does not own: clap command specs, report construction, or rendering.
//! Boundary: converts list command arguments into command-runner options.

use super::NnsCommonOptions;
use crate::{cli::common::OutputFormat, nns::leaf::commands::VERBOSE_ARG};
use clap::ArgMatches;

///
/// NnsLeafListOptions
///
/// Parsed options accepted by generic NNS leaf list command runners.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) verbose: bool,
}

impl NnsLeafListOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            verbose: matches.get_flag(VERBOSE_ARG),
        }
    }
}
