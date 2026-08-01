//! Module: sns::commands::run::neurons
//!
//! Responsibility: run SNS neuron list, refresh, and cache subcommands.
//! Does not own: neuron cache mechanics, live governance calls, or rendering.
//! Boundary: chooses live versus cache-capable request setup from CLI options.

mod cache;
mod refresh;

use crate::{
    cli::common::write_text_or_json,
    sns::commands::{
        SnsCommandError,
        options::SnsNeuronsOptions,
        run::common::{
            command_args, command_cache_root, lookup_command_parts, parse_required_command,
        },
        spec::{SnsNeuronsSortArg, sns_neuron_command, sns_neuron_list_usage, sns_neuron_usage},
    },
};
use ic_query::sns::{
    SnsNeuronsRequest, SnsNeuronsSort, build_sns_neurons_report, sns_neurons_report_text,
};
use std::{ffi::OsString, path::PathBuf};

pub(super) fn run_sns_neuron<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neuron_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_command(sns_neuron_command(), args)?;
    match command.as_str() {
        "list" => run_sns_neuron_list(args),
        "refresh" => refresh::run_sns_neuron_refresh(args),
        "cache" => cache::run_sns_neuron_cache(args),
        _ => unreachable!("sns neuron dispatch command only defines known commands"),
    }
}

fn run_sns_neuron_list<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, sns_neuron_list_usage) else {
        return Ok(());
    };
    let options = SnsNeuronsOptions::parse(args)?;
    let parts = lookup_command_parts(options.lookup)?;
    let format = parts.format;
    let cache_root = cache_root_for_sort(options.sort)?;
    let request = SnsNeuronsRequest {
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        limit: options.limit,
        owner_principal_id: options.owner_principal_id,
        sort: options.sort.into(),
        cache_root,
        verbose: options.verbose,
    };
    let report = build_sns_neurons_report(&request)?;
    write_text_or_json(format, &report, sns_neurons_report_text)
}

fn cache_root_for_sort(sort: SnsNeuronsSortArg) -> Result<Option<PathBuf>, SnsCommandError> {
    if SnsNeuronsSort::from(sort).uses_cache() {
        return Ok(Some(command_cache_root()?));
    }
    Ok(None)
}
