use crate::nns::{OutputFormat, leaf::NnsCommonOptions};
use clap::ArgMatches;

///
/// RegistryVersionOptions
///
/// Parsed options accepted by `icq nns registry version`.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct RegistryVersionOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl RegistryVersionOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        }
    }
}
