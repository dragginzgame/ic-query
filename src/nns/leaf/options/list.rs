use super::NnsCommonOptions;
use crate::{
    cli::common::OutputFormat,
    nns::{
        NnsCommandError,
        leaf::{
            commands::{VERBOSE_ARG, list_command, list_usage},
            model::NnsLeafCommandSpec,
        },
        parse_nns_matches,
    },
};
use std::ffi::OsString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafListOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) verbose: bool,
}

impl NnsLeafListOptions {
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(list_command(spec, default_source_endpoint), args, || {
            list_usage(spec, default_source_endpoint)
        })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            verbose: matches.get_flag(VERBOSE_ARG),
        })
    }
}
