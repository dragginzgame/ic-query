//! Module: nns::leaf::options::info
//!
//! Responsibility: parse clap options for generic NNS leaf info commands.
//! Does not own: clap command specs, report construction, or rendering.
//! Boundary: converts info command arguments into command-runner options.

use super::NnsCommonOptions;
use crate::{
    cli::{clap::required_string, common::OutputFormat},
    nns::{
        NnsCommandError,
        leaf::{
            commands::{INPUT_ARG, info_command, info_usage},
            model::NnsLeafCommandSpec,
        },
        parse_nns_matches,
    },
};
use std::ffi::OsString;

///
/// NnsLeafInfoOptions
///
/// Parsed options accepted by generic NNS leaf info command runners.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafInfoOptions {
    pub(in crate::nns) input: String,
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
}

impl NnsLeafInfoOptions {
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches = parse_nns_matches(info_command(spec, default_source_endpoint), args, || {
            info_usage(spec, default_source_endpoint)
        })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            input: required_string(&matches, INPUT_ARG),
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
        })
    }
}
