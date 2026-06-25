//! Module: nns::leaf::options::common
//!
//! Responsibility: parse common clap options for generic NNS leaf commands.
//! Does not own: command specs, report requests, or output rendering.
//! Boundary: converts shared clap matches into reusable leaf option values.

use crate::{
    cli::{
        clap::{required_string, required_typed},
        common::{FORMAT_ARG, OutputFormat, SOURCE_ENDPOINT_ARG},
    },
    nns::leaf::commands::NETWORK_ARG,
};
use clap::ArgMatches;

///
/// NnsCommonOptions
///
/// Common parsed options shared by generic NNS leaf command variants.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsCommonOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsCommonOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches) -> Self {
        Self {
            network: required_string(matches, NETWORK_ARG),
            format: required_typed(matches, FORMAT_ARG),
            source_endpoint: required_string(matches, SOURCE_ENDPOINT_ARG),
        }
    }
}
