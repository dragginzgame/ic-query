//! Module: sns::commands::options::reward
//!
//! Responsibility: parse SNS reward checkpoint options from Clap matches.
//! Does not own: strict pagination, source calls, or report output.
//! Boundary: combines shared lookup options with an optional diagnostic page cap.

use crate::{
    cli::{
        clap::{required_typed, typed_option},
        common::{OutputFormat, output_format},
    },
    sns::commands::options::lookup::SnsLookupOptions,
};
use clap::ArgMatches;
use std::path::PathBuf;

///
/// SnsRewardCheckpointOptions
///
/// Parsed options accepted by `icq sns reward checkpoint`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsRewardCheckpointOptions {
    pub(in crate::sns::commands) lookup: SnsLookupOptions,
    pub(in crate::sns::commands) max_pages: Option<u32>,
}

impl SnsRewardCheckpointOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            lookup: SnsLookupOptions::from_matches(matches, network),
            max_pages: typed_option(matches, "max-pages"),
        }
    }
}

///
/// SnsRewardDiffOptions
///
/// Parsed local file options accepted by `icq sns reward diff`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::sns::commands) struct SnsRewardDiffOptions {
    pub(in crate::sns::commands) before_checkpoint: PathBuf,
    pub(in crate::sns::commands) after_checkpoint: PathBuf,
    pub(in crate::sns::commands) format: OutputFormat,
}

impl SnsRewardDiffOptions {
    pub(in crate::sns::commands) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            before_checkpoint: required_typed(matches, "before-checkpoint"),
            after_checkpoint: required_typed(matches, "after-checkpoint"),
            format: output_format(matches),
        }
    }
}
