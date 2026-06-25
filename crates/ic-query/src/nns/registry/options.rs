use super::commands::{registry_version_command, registry_version_usage_for_error};
use crate::nns::{NnsCommandError, OutputFormat, leaf::NnsCommonOptions, parse_nns_matches};
use std::ffi::OsString;

///
/// RegistryVersionOptions
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct RegistryVersionOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl RegistryVersionOptions {
    pub(in crate::nns) fn parse<I>(args: I) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(
            registry_version_command(),
            args,
            registry_version_usage_for_error,
        )?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}
