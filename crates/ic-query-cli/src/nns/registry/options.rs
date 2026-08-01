#[cfg(test)]
use super::commands::registry_version_command;
use crate::nns::{OutputFormat, leaf::NnsCommonOptions};
use clap::ArgMatches;
#[cfg(test)]
use std::ffi::OsString;

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

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, crate::nns::NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(registry_version_command(), args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
