mod read;
mod refresh;

use super::commands::{topology_command, topology_usage};
use crate::{
    cli::clap::parse_required_subcommand_or_usage,
    nns::{NnsCommandError, command_flag_args},
};
use std::ffi::OsString;

pub(in crate::nns) fn run<I>(args: I) -> Result<(), NnsCommandError>
where
    I: IntoIterator<Item = OsString>,
{
    let Some(args) = command_flag_args(args, topology_usage) else {
        return Ok(());
    };
    let (command, args) =
        parse_required_subcommand_or_usage(topology_command(), args, topology_usage)
            .map_err(NnsCommandError::Usage)?;

    match command.as_str() {
        "summary" => read::run_topology_summary(args),
        "coverage" => read::run_topology_coverage(args),
        "versions" => read::run_topology_versions(args),
        "health" => read::run_topology_health(args),
        "gaps" => read::run_topology_gaps(args),
        "capacity" => read::run_topology_capacity(args),
        "regions" => read::run_topology_regions(args),
        "providers" => read::run_topology_providers(args),
        "refresh" => refresh::run_topology_refresh(args),
        _ => unreachable!("nns topology dispatch command only defines known commands"),
    }
}
