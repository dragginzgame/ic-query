//! Module: nns::leaf::options::common
//!
//! Responsibility: parse common clap options for generic NNS leaf commands.
//! Does not own: command specs, report requests, or output rendering.
//! Boundary: converts shared clap matches into reusable leaf option values.

use crate::cli::{
    clap::required_string,
    common::{OutputFormat, SOURCE_ENDPOINT_ARG, output_format},
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
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        Self {
            network: network.to_string(),
            format: output_format(matches),
            source_endpoint: required_string(matches, SOURCE_ENDPOINT_ARG),
        }
    }
}
