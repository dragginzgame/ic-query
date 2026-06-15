mod cache;
mod refresh;

use super::common::{command_icp_root, command_unix_secs};
use crate::{
    cli::{common::write_text_or_json, help::print_help_or_version},
    sns::{
        commands::{
            SnsCommandError,
            options::SnsNeuronsOptions,
            spec::{SnsNeuronsSortArg, sns_neurons_usage},
        },
        report::{
            SnsNeuronsRequest, SnsNeuronsSort, build_sns_neurons_report, sns_neurons_report_text,
        },
    },
    version_text,
};
use std::{ffi::OsString, path::PathBuf};

pub(super) fn run_sns_neurons<I>(args: I) -> Result<(), SnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, sns_neurons_usage, version_text()) {
        return Ok(());
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("refresh") {
        return refresh::run_sns_neurons_refresh(args.into_iter().skip(1));
    }
    if args.first().and_then(|arg| arg.to_str()) == Some("cache") {
        return cache::run_sns_neurons_cache(args.into_iter().skip(1));
    }
    let options = SnsNeuronsOptions::parse(args)?;
    if options.sort != SnsNeuronsSortArg::Api && options.owner_principal_id.is_some() {
        return Err(SnsCommandError::Usage(
            "`icq sns neurons --sort <id|stake|maturity|created>` reads the complete full-neuron cache and does not support --owner yet; use --sort api for owner-filtered live queries".to_string(),
        ));
    }
    let format = options.lookup.format;
    let icp_root = cache_root_for_sort(options.sort)?;
    let request = SnsNeuronsRequest {
        network: options.lookup.network,
        source_endpoint: options.lookup.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.lookup.input,
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
