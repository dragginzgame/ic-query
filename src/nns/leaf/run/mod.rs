mod cached;

use super::{
    commands::{command, usage},
    model::NnsLeafCommandSpec,
};
use crate::nns::{NnsCommandError, command_args, parse_nns_required_subcommand};
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
    let Some(args) = command_args(args, || usage(spec)) else {
        return Ok(());
    };
    let (command_name, args) = parse_nns_required_subcommand(command(spec), args, || usage(spec))?;

    match command_name.as_str() {
        "list" => run_list(args),
        "info" => run_info(args),
        "refresh" => run_refresh(args),
        _ => unreachable!("nns leaf dispatch command only defines known commands"),
    }
}
