use super::{info::info_command, list::list_command, refresh::refresh_command};
use clap::Command as ClapCommand;

pub(in crate::nns) fn subnet_command() -> ClapCommand {
    ClapCommand::new("subnet")
        .bin_name("icq nns subnet")
        .about("Inspect and refresh NNS subnet metadata")
        .subcommand_required(true)
        .subcommand(list_command())
        .subcommand(info_command())
        .subcommand(refresh_command())
}
