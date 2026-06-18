//! Module: sns::commands::run::neurons
//!
//! Responsibility: run SNS neuron list, refresh, and cache subcommands.
//! Does not own: neuron cache mechanics, live governance calls, or rendering.
//! Boundary: chooses live versus cache-capable request setup from CLI options.

mod cache;
mod refresh;

use crate::{
    cli::{clap::OptionalSubcommand, common::write_text_or_json},
    sns::{
        commands::{
            SnsCommandError,
            options::SnsNeuronsOptions,
            run::common::{
                command_args, command_icp_root, lookup_command_parts, parse_optional_command,
            },
            spec::{SnsNeuronsSortArg, sns_neurons_dispatch_command, sns_neurons_usage},
        },
        report::{
            SnsNeuronsRequest, SnsNeuronsSort, build_sns_neurons_report, sns_neurons_report_text,
        },
    },
};
use std::{ffi::OsString, path::PathBuf};

pub(super) fn run_sns_neurons<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neurons_usage) else {
        return Ok(());
    };
    let args =
        match parse_optional_command(sns_neurons_dispatch_command(), args, sns_neurons_usage)? {
            OptionalSubcommand::Matched { name, args } => {
                return match name.as_str() {
                    "refresh" => refresh::run_sns_neurons_refresh(args),
                    "cache" => cache::run_sns_neurons_cache(args),
                    _ => unreachable!("sns neurons dispatch command only defines known commands"),
                };
            }
            OptionalSubcommand::Passthrough(args) => args,
        };
    let options = SnsNeuronsOptions::parse(args)?;
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let icp_root = cache_root_for_sort(options.sort)?;
    let request = SnsNeuronsRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        limit: options.limit,
        owner_principal_id: options.owner_principal_id,
        sort: options.sort.into(),
        icp_root,
        verbose: options.verbose,
    };
    let report = build_sns_neurons_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}

fn cache_root_for_sort(sort: SnsNeuronsSortArg) -> Result<Option<PathBuf>, SnsCommandError> {
    if SnsNeuronsSort::from(sort).uses_cache() {
        return Ok(Some(command_icp_root()?));
    }
    Ok(None)
}
