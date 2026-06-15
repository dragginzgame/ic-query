use crate::cli::clap::passthrough_subcommand;
use clap::Command as ClapCommand;

pub(in crate::nns::subnet) fn subnet_command() -> ClapCommand {
    ClapCommand::new("subnet")
        .bin_name("icq nns subnet")
        .about("Inspect and refresh NNS subnet metadata")
        .disable_help_flag(true)
        .subcommand(passthrough_subcommand(
            ClapCommand::new("list").about("List cached mainnet IC subnets"),
        ))
        .subcommand(passthrough_subcommand(ClapCommand::new("info").about(
            "Resolve a subnet, canister, or subnet prefix to cached subnet info",
        )))
        .subcommand(passthrough_subcommand(
            ClapCommand::new("refresh").about("Force-refresh and cache NNS subnet metadata"),
        ))
}
