//! Module: nns::leaf::options::refresh
//!
//! Responsibility: parse clap options for generic NNS leaf refresh commands.
//! Does not own: clap command specs, cache writes, or report construction.
//! Boundary: converts refresh command arguments into command-runner options.

use super::NnsCommonOptions;
use crate::{
    cli::clap::typed_option,
    nns::leaf::commands::{DRY_RUN_ARG, OUTPUT_ARG},
};
use clap::ArgMatches;
use std::path::PathBuf;

///
/// NnsLeafRefreshOptions
///
/// Parsed options accepted by generic NNS leaf refresh command runners.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafRefreshOptions {
    pub(in crate::nns) common: NnsCommonOptions,

    pub(in crate::nns) dry_run: bool,
    pub(in crate::nns) output_path: Option<PathBuf>,
}

impl NnsLeafRefreshOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            common,

            dry_run: matches.get_flag(DRY_RUN_ARG),
            output_path: typed_option(matches, OUTPUT_ARG),
        }
    }
}
