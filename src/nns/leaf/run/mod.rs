mod cached;

use super::{
    commands::{command, usage},
    model::NnsLeafCommandSpec,
};
use crate::{
    cli::{clap::parse_required_subcommand_or_usage, help::print_help_or_version},
    nns::NnsCommandError,
    version_text,
};
use std::ffi::OsString;

pub(in crate::nns) use cached::run_cached_leaf;

pub(in crate::nns) fn run_leaf<I>(
    args: I,
    spec: &NnsLeafCommandSpec,
    run_list: fn(Vec<OsString>) -> Result<(), NnsCommandError>,
    run_info: fn(Vec<OsString>) -> Result<(), NnsCommandError>,
    run_refresh: fn(Vec<OsString>) -> Result<(), NnsCommandError>,
) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, || usage(spec), version_text()) {
        return Ok(());
    }
    let (command_name, args) =
        parse_required_subcommand_or_usage(command(spec), args, || usage(spec))
            .map_err(NnsCommandError::Usage)?;

    match command_name.as_str() {
        "list" => run_list(args),
        "info" => run_info(args),
        "refresh" => run_refresh(args),
        _ => unreachable!("nns leaf dispatch command only defines known commands"),
    }
}
