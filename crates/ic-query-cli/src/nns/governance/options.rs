//! Parsed options shared by direct NNS Governance reports.

use crate::nns::{OutputFormat, leaf::NnsCommonOptions};
use clap::ArgMatches;
#[cfg(test)]
use clap::Command as ClapCommand;
#[cfg(test)]
use std::ffi::OsString;

///
/// NnsGovernanceOptions
///
/// Common target and output options for one direct NNS Governance report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsGovernanceOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsGovernanceOptions {
    pub(in crate::nns) fn from_matches(matches: &ArgMatches, network: &str) -> Self {
        let common = NnsCommonOptions::from_matches(matches, network);
        Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        }
    }

    #[cfg(test)]
    pub(in crate::nns) fn parse<I>(
        args: I,
        command: ClapCommand,
    ) -> Result<Self, crate::nns::NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = crate::nns::parse_nns_matches(command, args)?;
        Ok(Self::from_matches(
            &matches,
            ic_query::subnet_catalog::MAINNET_NETWORK,
        ))
    }
}
