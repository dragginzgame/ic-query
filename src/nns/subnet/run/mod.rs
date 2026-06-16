mod cache;
mod info;
mod list;
mod refresh;

use super::commands::{subnet_command, subnet_usage};
use crate::{
    cli::clap::parse_required_subcommand_or_usage,
    nns::{NnsCommandError, command_args},
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_args(args, subnet_usage) else {
        return Ok(());
    };
    let (command, args) = parse_required_subcommand_or_usage(subnet_command(), args, subnet_usage)
        .map_err(NnsCommandError::Usage)?;

    match command.as_str() {
        "list" => list::run_catalog_list(args),
        "info" => info::run_catalog_info(args),
        "refresh" => refresh::run_catalog_refresh(args),
        _ => unreachable!("nns subnet dispatch command only defines known commands"),
    }
}
