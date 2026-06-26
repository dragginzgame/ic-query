use super::{
    commands::{registry_command, registry_usage_for_error, registry_version_usage_for_error},
    options::RegistryVersionOptions,
};
use crate::nns::{
    NnsCommandError, command_args, command_flag_args, now_unix_secs, parse_nns_required_subcommand,
    registry::report::{
        NnsRegistryVersionRequest, build_nns_registry_version_report,
        nns_registry_version_report_text,
    },
    write_text_or_json,
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_flag_args(args, registry_usage_for_error) else {
        return Ok(());
    };
    let (command, args) =
        parse_nns_required_subcommand(registry_command(), args, registry_usage_for_error)?;

    match command.as_str() {
        "version" => run_registry_version(args),
        _ => unreachable!("nns registry dispatch command only defines known commands"),
    }
}

fn run_registry_version<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, registry_version_usage_for_error) else {
        return Ok(());
    };
    let options = RegistryVersionOptions::parse(args)?;
    let request =
        NnsRegistryVersionRequest::new(options.network, options.source_endpoint, now_unix_secs()?);
    let report = build_nns_registry_version_report(&request)?;
    write_text_or_json(options.format, &report, nns_registry_version_report_text)
}
