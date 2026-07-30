//! Parsed options shared by direct NNS Governance reports.

use crate::nns::{NnsCommandError, OutputFormat, leaf::NnsCommonOptions, parse_nns_matches};
use clap::Command as ClapCommand;
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
    pub(in crate::nns) fn parse<I>(
        args: I,
        command: ClapCommand,
        usage: impl FnOnce() -> String,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(command, args, usage)?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}
