use super::{
    commands::{registry_command, registry_usage_for_error, registry_version_usage_for_error},
    options::RegistryVersionOptions,
};
use crate::{
    cli::{
        clap::parse_required_subcommand_or_usage,
        help::{print_help_or_version, print_help_or_version_flag},
    },
    nns::{
        NnsCommandError, now_unix_secs,
        registry::report::{
            NnsRegistryVersionRequest, build_nns_registry_version_report,
            nns_registry_version_report_text,
        },
        write_text_or_json,
    },
    version_text,
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version_flag(&args, registry_usage_for_error, version_text()) {
        return Ok(());
    }
    let (command, args) =
        parse_required_subcommand_or_usage(registry_command(), args, registry_usage_for_error)
            .map_err(NnsCommandError::Usage)?;

    match command.as_str() {
        "version" => run_registry_version(args),
        _ => unreachable!("nns registry dispatch command only defines known commands"),
    }
}

fn run_registry_version<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, registry_version_usage_for_error, version_text()) {
        return Ok(());
    }
    let options = RegistryVersionOptions::parse(args)?;
    let request = NnsRegistryVersionRequest {
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: now_unix_secs()?,
    };
    let report = build_nns_registry_version_report(&request)?;
    write_text_or_json(options.format, &report, nns_registry_version_report_text)
}
