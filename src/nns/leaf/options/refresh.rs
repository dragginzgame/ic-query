use super::NnsCommonOptions;
use crate::{
    cli::{
        clap::{required_typed, typed_option},
        common::OutputFormat,
    },
    nns::{
        NnsCommandError,
        leaf::{
            commands::{
                DRY_RUN_ARG, LOCK_STALE_AFTER_ARG, OUTPUT_ARG, parse_leaf_matches, refresh_command,
                refresh_usage,
            },
            model::NnsLeafCommandSpec,
        },
    },
};
use std::{ffi::OsString, path::PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::nns) struct NnsLeafRefreshOptions {
    pub(in crate::nns) network: String,
    pub(in crate::nns) format: OutputFormat,
    pub(in crate::nns) source_endpoint: String,
    pub(in crate::nns) lock_stale_after_seconds: u64,
    pub(in crate::nns) dry_run: bool,
    pub(in crate::nns) output_path: Option<PathBuf>,
}

impl NnsLeafRefreshOptions {
    pub(in crate::nns) fn parse<I>(
        args: I,
        spec: &NnsLeafCommandSpec,
        default_source_endpoint: &'static str,
    ) -> Result<Self, NnsCommandError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let matches =
            parse_leaf_matches(refresh_command(spec, default_source_endpoint), args, || {
                refresh_usage(spec, default_source_endpoint)
            })?;
        let common = NnsCommonOptions::from_matches(&matches);
        Ok(Self {
            network: common.network,
            format: common.format,
            source_endpoint: common.source_endpoint,
            lock_stale_after_seconds: required_typed(&matches, LOCK_STALE_AFTER_ARG),
            dry_run: matches.get_flag(DRY_RUN_ARG),
            output_path: typed_option(&matches, OUTPUT_ARG),
        })
    }
}
