mod cache;
mod info;
mod list;
mod refresh;

use super::commands::{subnet_command, subnet_usage};
use crate::{
    cli::{clap::parse_required_subcommand_or_usage, help::print_help_or_version},
    nns::NnsCommandError,
    version_text,
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    if print_help_or_version(&args, subnet_usage, version_text()) {
        return Ok(());
    }
    let (command, args) = parse_required_subcommand_or_usage(subnet_command(), args, subnet_usage)
        .map_err(NnsCommandError::Usage)?;

    match command.as_str() {
        "list" => list::run_catalog_list(args),
        "info" => info::run_catalog_info(args),
        "refresh" => refresh::run_catalog_refresh(args),
        _ => unreachable!("nns subnet dispatch command only defines known commands"),
    }
}
